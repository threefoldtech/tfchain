const StellarSdk = require('stellar-sdk')
const { getClient } = require('./src/subclient')
const server = new StellarSdk.Server('https://horizon.stellar.org')
const stellarbase = require('stellar-base')
const chalk = require('chalk')

const yargs = require('yargs')

const argv = yargs
  .command('validate', 'Start your validator node', {
    mnemonic: {
      description: 'Mnemonic to sign with',
      alias: 'm',
      type: 'string'
    },
    apiUrl: {
      description: 'Url of the api',
      alias: 'a',
      type: 'string'
    },
    account: {
      description: 'Account to monitor',
      alias: 'c',
      type: 'string',
      default: 'GDIVGRGFOHOEWJKRR5KKW2AJALUYARHO7MW3SPI4N33IJZ4PQ5WJ6TU2'
    }
  })
  .help()
  .alias('help', 'h')
  .argv

if (argv._.includes('validate')) {
  // if (!argv.n || !argv.c || !argv.t) {
  //   console.log(argv)
  //   console.log('Bad Params')
  //   exit(1)
  // }

  main(argv.m, argv.a, argv.c)
}

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

  // .catch(err => {
  //   console.log(err)
  //   console.log('error occurred!!!')
  //   // VOTE
  //   client.voteTransaction(txHash, res => callback(res, client, txHash))
  // })
}

async function main (mnemonic, apiurl, accountToMonitor) {
  const client = await getClient(apiurl, mnemonic)

  const knownValidators = await client.listValidators()
  if (!knownValidators.includes(client.address)) {
    console.log(chalk.red.bold('❌ You are no validator yet, please contact an admin to add your account as a validator first.'))
    process.exit(0)
  }
  // // FROM STELLAR
  // const ss = client.keyring.encodeAddress(stellarbase.StrKey.decodeEd25519PublicKey('GCCNQN4HVJVH5XOV3A2BO7NQ3OJY6MEOW7MSXQJU3VWGRPXU273BN5OB'))
  // console.log(ss)

  // // TO STELLAR
  // const y = stellarbase.StrKey.encodeEd25519PublicKey(client.keyring.decodeAddress(client.address))
  // console.log(y)

  // const accountToMonitor = 'GDIVGRGFOHOEWJKRR5KKW2AJALUYARHO7MW3SPI4N33IJZ4PQ5WJ6TU2'

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
    // console.log(res)
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
      // console.log(`\t' ${phase}: ${section}.${method}:: ${data}`)
    })

    if (callbackVote) {
      console.log(chalk.blue.bold('\nfailed to propose transaction, voting for validity now...'))
      await client.voteTransaction(txHash, res => callback(res, client, txHash)).catch(() => {
        client.voteTransaction(txHash, callback)
      })
    }
  }
}
