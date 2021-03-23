const StellarSdk = require('stellar-sdk')

const rootKeypair = StellarSdk.Keypair.fromSecret('SAZ2Q6Q2PDKEJLV7DPG75S7N7SNY6DPSJY2ILN5O6WALJZEPJ5FGCJW2')
const sourcePublicKey = rootKeypair.publicKey()

async function main () {
  const server = new StellarSdk.Server('https://horizon-testnet.stellar.org')

  const account = await server.loadAccount(sourcePublicKey)

  const transaction = new StellarSdk.TransactionBuilder(account, {
    fee: StellarSdk.BASE_FEE,
    networkPassphrase: StellarSdk.Networks.TESTNET
  })
    .addOperation(StellarSdk.Operation.payment({
      destination: 'GCGBK2U4AP6CTBYDSI6SIVSSPAGS6GSVAR6KUEYRIQTQURFNOYI2B7FU',
      asset: new StellarSdk.Asset('TFT', 'GA47YZA3PKFUZMPLQ3B5F2E3CJIB57TGGU7SPCQT2WAEYKN766PWIMB3'),
      amount: '1250'
    }))
    .setTimeout(6000)
    .build()

  // now we need to sign the transaction with both the root and the secondaryAddress
  transaction.sign(rootKeypair)

  console.log(transaction.toXDR())
}

main()
