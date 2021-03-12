const { Account } = require('stellar-sdk')
const StellarSdk = require('stellar-sdk')
const { parse } = require('yargs')

const rootKeypair = StellarSdk.Keypair.fromSecret('SBQWY3DNPFWGSZTFNV4WQZLBOJ2GQYLTMJSWK3TTMVQXEY3INFXGO52X')
const sourcePublicKey = rootKeypair.publicKey()
console.log(sourcePublicKey)

const secondaryAddress = 'GC6HHHS7SH7KNUAOBKVGT2QZIQLRB5UA7QAGLA3IROWPH4TN65UKNJPK'

async function main () {
  const server = new StellarSdk.Server('https://horizon-testnet.stellar.org')

  const account = await server.loadAccount(sourcePublicKey)

  // let transaction = new StellarSdk.TransactionBuilder(account, {
  //   fee: StellarSdk.BASE_FEE,
  //   networkPassphrase: StellarSdk.Networks.TESTNET
  // })
  //   .addOperation(StellarSdk.Operation.setOptions({
  //     signer: {
  //       ed25519PublicKey: secondaryAddress,
  //       weight: 1
  //     }
  //   }))
  //   .addOperation(StellarSdk.Operation.setOptions({
  //     masterWeight: 1, // set master key weight
  //     lowThreshold: 1,
  //     medThreshold: 2, // a payment is medium threshold
  //     highThreshold: 2 // make sure to have enough weight to add up to the high threshold!
  //   }))
  //   .setTimeout(30)
  //   .build()

  // transaction.sign(rootKeypair) // only need to sign with the root signer as the 2nd signer won't be added to the account till after this transaction completes

  // try {
  //   const transactionResult = await server.submitTransaction(transaction)
  //   console.log(JSON.stringify(transactionResult, null, 2))
  //   console.log('\nSuccess! View the transaction at: ')
  //   console.log(transactionResult._links.transaction.href)
  // } catch (e) {
  //   console.log('An error has occured:')
  //   console.log(e)
  // }

  // now create a payment with the account that has two signers

  const transaction = new StellarSdk.TransactionBuilder(account, {
    fee: StellarSdk.BASE_FEE,
    networkPassphrase: StellarSdk.Networks.TESTNET
  })
    .addOperation(StellarSdk.Operation.payment({
      destination: 'GBTVUCDT5CNSXIHJTDHYSZG3YJFXBAJ6FM4CKS5GKSAWJOLZW6XX7NVC',
      asset: StellarSdk.Asset.native(),
      amount: '2000' // 2000 XLM
    }))
    .setTimeout(6000)
    .build()

  // now we need to sign the transaction with both the root and the secondaryAddress
  transaction.sign(rootKeypair)
  console.log(transaction.sequence)

  console.log(transaction.toXDR())
}

main()
