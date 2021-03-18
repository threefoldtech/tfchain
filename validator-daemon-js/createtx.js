const StellarSdk = require('stellar-sdk')

const rootKeypair = StellarSdk.Keypair.fromSecret('SCDBPKFTGKZMYX3BI6H73RRT6IEP55GX6BBWJGCSQKZXIVE5JMTBQBJU')
const sourcePublicKey = rootKeypair.publicKey()

async function main () {
  const server = new StellarSdk.Server('https://horizon-testnet.stellar.org')

  const account = await server.loadAccount(sourcePublicKey)

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

  console.log(transaction.toXDR())
}

main()
