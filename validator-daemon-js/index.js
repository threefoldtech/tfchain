const StellarSdk = require('stellar-sdk')
const { getClient } = require('./src/subclient')
const server = new StellarSdk.Server('https://horizon.stellar.org')
const stellarbase = require('stellar-base')

const yargs = require('yargs')
// const { exit } = require('yargs')

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

  main(argv.m, argv.a)
}

const paymentHandler = async function (paymentResponse, client) {
  console.log('\nreceived transaction')
  // const tx = await paymentResponse.transaction()
  // const { memo } = tx

  const { from, to, amount, transaction_hash: txHash } = paymentResponse
  console.log(`from: ${from} to: ${to}`)
  console.log(`amount: ${amount} \n`)

  const targetAccount = client.keyring.encodeAddress(stellarbase.StrKey.decodeEd25519PublicKey(from))

  console.log('trying to propose transaction...')
  await client.proposeTransaction(txHash, targetAccount, amount, res => callback(res, client, txHash)).catch(err => {
    console.log(err)
    console.log('error occurred!!!')
    // VOTE
    client.voteTransaction(txHash, res => callback(res, client, txHash))
  })
}

async function main (mnemonic, apiurl) {
  const client = await getClient(apiurl, mnemonic)
  // // FROM STELLAR
  // const ss = client.keyring.encodeAddress(stellarbase.StrKey.decodeEd25519PublicKey('GCCNQN4HVJVH5XOV3A2BO7NQ3OJY6MEOW7MSXQJU3VWGRPXU273BN5OB'))
  // console.log(ss)

  // // TO STELLAR
  // const y = stellarbase.StrKey.encodeEd25519PublicKey(client.keyring.decodeAddress(client.address))
  // console.log(y)

  console.log('starting validator daemon...')
  console.log('streaming transaction now...')
  server.payments()
    .forAccount('GDIVGRGFOHOEWJKRR5KKW2AJALUYARHO7MW3SPI4N33IJZ4PQ5WJ6TU2')
    .cursor('now')
    .stream({
      onmessage: message => paymentHandler(message, client)
    })
}

// main()
async function callback (res, client, txHash) {
  if (res instanceof Error) {
    console.log(res)
  }
  const { events = [], status } = res
  console.log(`Current status is ${status.type}`)

  if (status.isFinalized) {
    console.log(`Transaction included at blockHash ${status.asFinalized}`)

    let callbackVote = false
    // Loop through Vec<EventRecord> to display all events
    events.forEach(({ phase, event: { data, method, section } }) => {
      if (method.toString() === 'ExtrinsicFailed') {
        console.log('tx failed, might be needed to vote here ;)')
        // VOTE
        callbackVote = true
      }

      console.log(`\t' ${phase}: ${section}.${method}:: ${data}`)
    })

    if (callbackVote || status.type === 'Invalid') {
      console.log('I need to vote now...')
      await client.voteTransaction(txHash, res => callback(res, client, txHash)).catch(err => {
        console.log(err)
        console.log('error occurred!!!')
        // VOTE
        client.voteTransaction(txHash, callback)
      })
    }
  }
}
