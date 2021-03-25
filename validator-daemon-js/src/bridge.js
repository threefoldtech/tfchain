const StellarSdk = require('stellar-sdk')
const { getClient } = require('./subclient')
const server = new StellarSdk.Server('https://horizon.stellar.org')
const stellarbase = require('stellar-base')
const chalk = require('chalk')

const paymentHandler = async function (paymentResponse, client, accountToMonitor) {
  console.log(chalk.blue.bold('\nreceived transaction'))
  // const tx = await paymentResponse.transaction()
  // const { memo } = tx

  const { from, to, amount, transaction_hash: txHash } = paymentResponse

  // only check when destination is the account to monitor
  if (to === accountToMonitor) {
    console.log(chalk.blue.bold(`from: ${from} to: ${to}`))
    console.log(chalk.blue.bold(`amount: ${amount} \n`))

    const targetAccount = client.keyring.encodeAddress(stellarbase.StrKey.decodeEd25519PublicKey(from))

    console.log(chalk.green.bold('trying to propose transaction...'))
    await client.proposeTransaction(txHash, targetAccount, amount, res => callback(res, client, txHash))
  }
}

async function monitorBridge (mnemonic, apiurl, accountToMonitor) {
  const client = await getClient(apiurl, mnemonic)

  const knownValidators = await client.listBridgeValidators()
  if (!knownValidators.includes(client.address)) {
    console.log(chalk.red.bold('❌ You are no validator yet, please contact an admin to add your account as a validator first.'))
    process.exit(0)
  }

  console.log(chalk.green.bold('✓ starting validator daemon...'))
  console.log(chalk.blue.bold(`✓ streaming transactions for account ${accountToMonitor} now...`))

  server.payments()
    .forAccount(accountToMonitor)
    .cursor('now')
    .stream({
      onmessage: message => paymentHandler(message, client, accountToMonitor)
    })
}

async function callback (res, client, txHash) {
  if (res instanceof Error) {
    console.log(chalk.red.bold(res))
  }

  const { events = [], status } = res

  if (status.isFinalized) {
    let callbackVote = false
    events.forEach(({ phase, event: { data, method, section } }) => {
      if (method.toString() === 'ExtrinsicFailed') {
        const module = data[0].asModule
        const errid = module.error.words[0]

        // err with id 4 is transaction exists, in this case we want to vote for the transaction because
        // another validator already proposed it
        if (errid === 4) {
          callbackVote = true
          // error with 5 is transaction not exists, which means it is most likely processed already
        } else if (errid === 5) {
          console.log(chalk.blue.bold('\ntransaction already submitted, nothing to do here.'))
        }
      } else if (method.toString() === 'ExtrinsicSuccess') {
        console.log(chalk.green.bold('Transaction submitted successfully.'))
      }
    })

    if (callbackVote) {
      console.log(chalk.blue.bold('\nfailed to propose transaction, voting for validity now...'))
      await client.voteTransaction(txHash, res => callback(res, client, txHash)).catch(() => {
        client.voteTransaction(txHash, callback)
      })
    }
  }
}

module.exports = {
  monitorBridge
}
