const { getClient } = require('./subclient')
const StellarSdk = require('stellar-sdk')
const stellarbase = require('stellar-base')
const chalk = require('chalk')
const { difference } = require('lodash')

const server = new StellarSdk.Server('https://horizon-testnet.stellar.org')

async function monitorVesting (mnemonic, url) {
  const client = await getClient(url, mnemonic)
  // second signer in this case for the multisig test
  const stellarPair = StellarSdk.Keypair.fromSecret('SAMZUAAPLRUH62HH3XE7NVD6ZSMTWPWGM6DS4X47HLVRHEBKP4U2H5E7')
  console.log(chalk.green.bold('✓ starting vesting validator daemon...'))

  client.api.query.system.events((events) => {
    events.forEach((record) => {
      const { event } = record

      if (event.section === 'vestingValidatorModule') {
        console.log(event.method)

        switch (event.method) {
          case 'TransactionProposed':
            console.log(chalk.blue.bold('transaction proposal found'))
            handleTransactionProposal(record, client, stellarPair)
            break
          case 'TransactionReady':
            console.log(chalk.blue.bold('found a ready to be sumbitted transaction'))
            handleTransactionReady(record, client, stellarPair)
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

  // parse transaction from xdr string
  const transaction = new StellarSdk.Transaction(hex2a(tx.toJSON()), StellarSdk.Networks.TESTNET)

  // parse account from substrate address to stellar public key
  const stellarAccount = stellarbase.StrKey.encodeEd25519PublicKey(client.keyring.decodeAddress(account.toJSON()))
  const accountResponse = await server.loadAccount(stellarAccount)

  // todo add validation
  accountResponse.balances.map(balance => {
    console.log(balance)
  })

  // todo: check if the account can indeed send X amount of tokens
  console.log(transaction.operations)

  const signaturesPresent = transaction.signatures.map(sigs => sigs.toXDR().toString('base64'))
  // Sign the transaction and submit it back to storage
  transaction.sign(stellarPair)

  const signatureToAdd = difference(transaction.signatures.map(sigs => sigs.toXDR().toString('base64')), signaturesPresent)
  console.log(signatureToAdd)

  await client.addTransactionSignature(transaction.toXDR(), signatureToAdd[0], res => callback(res))
}

async function handleTransactionReady (record, client, stellarPair) {
  const { event } = record

  const [tx] = event.data
  const transactionXDR = hex2a(tx.toJSON())

  const transactionFromStorage = await client.getTransaction(transactionXDR)
  console.log(transactionFromStorage)

  // parse transaction from xdr string
  const stellarTransaction = new StellarSdk.Transaction(transactionXDR, StellarSdk.Networks.TESTNET)

  // add all the signatures from storage to this transaction
  const stellarSignatures = transactionFromStorage.signatures.map(sig => {
    const sigBuffer = Buffer.from(sig, 'base64')
    return StellarSdk.xdr.DecoratedSignature.fromXDR(sigBuffer)
  })
  stellarTransaction.signatures.push(...stellarSignatures)

  try {
    const transactionResult = await server.submitTransaction(stellarTransaction)
    console.log(chalk.green.bold(`Success! View the transaction at: ${transactionResult._links.transaction.href}`))
    await client.removeTransaction(stellarTransaction.toXDR(), res => callback(res))
  } catch (e) {
    console.log('An error has occured:')
    console.log(e.response.data.extras)
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

  console.log(status)
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
