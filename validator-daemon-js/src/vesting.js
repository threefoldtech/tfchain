const { getClient } = require('./subclient')
const StellarSdk = require('stellar-sdk')
const stellarbase = require('stellar-base')
const chalk = require('chalk')
const { difference, first, find } = require('lodash')
const bip39 = require('bip39')
const moment = require('moment')
const { sumBy } = require('lodash')

const server = new StellarSdk.Server('https://horizon-testnet.stellar.org')

async function monitorVesting (mnemonic, url) {
  const client = await getClient(url, mnemonic)

  const knownValidators = await client.listVestingValidators()
  if (!knownValidators.includes(client.address)) {
    console.log(chalk.red.bold('❌ You are no validator yet, please contact an admin to add your account as a validator first.'))
    process.exit(0)
  }

  const seed = await bip39.mnemonicToSeed(mnemonic)

  // extract stellar keypair from secret seed
  const stellarPair = StellarSdk.Keypair.fromRawEd25519Seed(seed.slice(0, 32))

  console.log(chalk.blue.bold(`Substrate address: ${client.address} is loaded.`))
  console.log(chalk.blue.bold(`Stellar address: ${stellarPair.publicKey()} is loaded.`))
  console.log(chalk.green.bold('✓ starting vesting validator daemon...'))

  client.api.query.system.events((events) => {
    events.forEach((record) => {
      const { event } = record

      if (event.section === 'vestingValidatorModule') {
        switch (event.method) {
          case 'TransactionProposed':
            console.log(chalk.blue.bold('transaction proposal found'))
            handleTransactionProposal(record, client, stellarPair)
            break
          case 'TransactionReady':
            console.log(chalk.blue.bold('found a ready to be sumbitted transaction'))
            handleTransactionReady(record, client)
            break
          default:
            console.log(chalk.blue.bold(`unknown event seen ${event.method}, skipping ...`))
            break
        }
      }
    })
  })
}

async function handleTransactionProposal (record, client, stellarPair) {
  const { event } = record

  const [tx, account] = event.data

  const transactionXDR = hex2a(tx.toJSON())
  // parse transaction from xdr string
  const transaction = new StellarSdk.Transaction(transactionXDR, StellarSdk.Networks.TESTNET)

  // parse account from substrate address to stellar public key
  const stellarAccount = stellarbase.StrKey.encodeEd25519PublicKey(client.keyring.decodeAddress(account.toJSON()))

  const avgTftPrice = await client.getAveragePrice()

  // todo: check if the account can indeed send X amount of tokens
  const paymentOperation = find(transaction.operations, ['type', 'payment'])
  if (!paymentOperation) return

  let amountWithdrawable = 0

  const accountVestingData = account.data_attr['tft-vesting']

  try {
    amountWithdrawable = await validateWithdrawal(accountVestingData, stellarAccount, avgTftPrice)
    console.log(`\t amount withdrawable: ${amountWithdrawable}\n`)
    if (parseFloat(paymentOperation.amount) > amountWithdrawable) {
      const error = `client trying to withdraw ${parseFloat(paymentOperation.amount)} whilst there is only ${amountWithdrawable} withdrawable.`
      console.log(chalk.red(error))
      await client.reportFailedTransaction(transactionXDR, error, res => callback(res))
      return
    }
  } catch (error) {
    console.log(error)
    console.log(chalk.red('tx failed, reporting now...'))
    await client.reportFailedTransaction(transactionXDR, error, res => callback(res))
    return
  }

  console.log(chalk.blue.red('withdrawal verified and ready to be signed, signing now...'))
  const signaturesPresent = transaction.signatures.map(sigs => sigs.toXDR().toString('base64'))
  // Sign the transaction and submit it back to storage
  transaction.sign(stellarPair)

  const signatureToAdd = difference(transaction.signatures.map(sigs => sigs.toXDR().toString('base64')), signaturesPresent)

  try {
    await client.addTransactionSignature(transactionXDR, first(signatureToAdd), res => callback(res))
  } catch (error) {
    console.log(error)
  }
}

async function handleTransactionReady (record, client) {
  const { event } = record

  const [tx] = event.data
  const transactionXDR = hex2a(tx.toJSON())

  const transactionFromStorage = await client.getTransaction(transactionXDR)

  // parse transaction from xdr string
  const stellarTransaction = new StellarSdk.Transaction(transactionXDR, StellarSdk.Networks.TESTNET)

  // add all the signatures from storage to this transaction
  const stellarSignatures = transactionFromStorage.signatures.map(sig => StellarSdk.xdr.DecoratedSignature.fromXDR(Buffer.from(hex2a(sig), 'base64')))
  stellarTransaction.signatures.push(...stellarSignatures)

  try {
    const transactionResult = await server.submitTransaction(stellarTransaction)
    console.log(chalk.green.bold(`Success! View the transaction at: ${transactionResult._links.transaction.href}`))
    await client.removeTransaction(transactionXDR, res => callback(res))
  } catch (e) {
    console.log(`A Stellar error has occured: ${e.response.data.extras}`)
    console.log(chalk.blue.bold('tx failed, reporting now...'))
    await client.reportFailedTransaction(transactionXDR, e.response.data.extras, res => callback(res))
  }
}

async function validateWithdrawal (encodedVestingSchedule, accountID, avgTftPrice) {
  const vestingSchedule = Buffer.from(encodedVestingSchedule, 'base64').toString()

  if (!vestingSchedule || vestingSchedule === '') {
    return
  }

  console.log(chalk.blue.bold(`\n\tfetching withdrawal details for: ${accountID}`))

  // const test = 'month1=04/2021,12months,priceunlock=tftvalue>month*0.0015+0.02'
  const [start, duration, priceUnlock] = vestingSchedule.split(',')

  const x = start.split('=')[1]
  const month = parseInt(x.split('/')[0])
  const year = parseInt(x.split('/')[1])
  // clean start date for vestin schedule
  const startDate = moment().month(month - 1).date(1).hour(0).minute(0).second(0).millisecond(0).set('year', year)

  const transactions = await server.transactions()
    .forAccount(accountID)
    .call()

  const operations = transactions.records.map(tx => {
    return tx.operations()
  })

  const res = await Promise.all(operations)

  // Calculate the total amount of TFT available on this account
  const total = sumBy(res, r => {
    return sumBy(r.records, record => {
      // gather all payment before the startdate of the vesting to calculate the amount of tokens that
      // can be withdrawn each month
      if (record.type === 'payment' && record.to === accountID && record.asset_code === 'TFT') {
        return parseFloat(record.amount)
      }
    })
  }) || 0
  console.log(`\t total deposited: ${total}`)

  // if no funds are present on the escrow account, just return
  if (!total) return Error('no funds are present on escrow account!')

  const numberOfMonths = parseInt(duration.slice(0, 2))
  console.log(`\t total months of vesting: ${numberOfMonths}`)

  const monthlyWithdrawable = total / numberOfMonths
  console.log(`\t monthly withdrawable: ${monthlyWithdrawable}`)

  // calculate how much already has been withdrawn from this account
  // in order to have replay protection
  const totalWithdrawn = sumBy(res, r => {
    return sumBy(r.records, record => {
      if (record.type === 'payment' && record.from === accountID) {
        return parseFloat(record.amount)
      }
    })
  }) || 0
  console.log(`\t total withdrawn already: ${totalWithdrawn}`)

  // TODO: remove simulation
  const now = moment().month(6)
  console.log(`\t simulated now date: ${now}`)

  console.log(`\t vesting start date: ${startDate}`)
  const monthsBetweenStartAndNow = now.diff(startDate, 'months')
  console.log(`\t months between start and now: ${monthsBetweenStartAndNow}`)

  if (monthsBetweenStartAndNow <= 0) throw Error('cannot withdraw yet!')

  // tftvalue>month*0.015+0.15
  const parsedPriceUnlockCondition = priceUnlock.split('*')[1]
  const [multiplier, basePrice] = parsedPriceUnlockCondition.split('+')

  const multiplierFloat = parseFloat(multiplier)
  const basePriceFloat = parseFloat(basePrice)

  const unlockCondition = (monthsBetweenStartAndNow * multiplierFloat) + basePriceFloat
  console.log(`\t unlock price condition: ${unlockCondition}`)
  console.log(`\t avg tft price: ${avgTftPrice}`)

  const canUnlock = avgTftPrice > unlockCondition
  console.log(`\t user can withdraw: ${canUnlock}`)

  if (canUnlock) {
    return (monthlyWithdrawable * monthsBetweenStartAndNow) - totalWithdrawn
  } else {
    throw Error(`cannot withdraw funds yet, the unlock price condition is $ ${unlockCondition} whilst the price is still $ ${avgTftPrice}`)
  }
}

function hex2a (hex) {
  let str = ''
  for (let i = 0; i < hex.length; i += 2) {
    const v = parseInt(hex.substr(i, 2), 16)
    if (v) str += String.fromCharCode(v)
  }
  return str
}

async function callback (res) {
  if (res instanceof Error) {
    console.log(chalk.red.bold(res))
  }

  const { events = [], status } = res

  console.log(`Current status is ${status.type}`)
  if (status.isFinalized) {
    events.forEach(({ event: { data, method } }) => {
      if (method.toString() === 'ExtrinsicFailed') {
        const module = data[0].asModule
        const errid = module.error.words[0]

        if (errid === 5) {
          console.log(chalk.blue.bold('\ntransaction already submitted, nothing to do here.'))
        }
      } else if (method.toString() === 'ExtrinsicSuccess') {
        console.log(chalk.green.bold('Transaction submitted successfully.'))
      }
    })
  }
}

module.exports = {
  monitorVesting
}
