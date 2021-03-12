const StellarSdk = require('stellar-sdk')

const rootKeypair = StellarSdk.Keypair.fromSecret('SBQWY3DNPFWGSZTFNV4WQZLBOJ2GQYLTMJSWK3TTMVQXEY3INFXGO52X')
const account = new StellarSdk.Account(rootKeypair.publicKey(), '46316927324160')

const secondaryAddress = 'GC6HHHS7SH7KNUAOBKVGT2QZIQLRB5UA7QAGLA3IROWPH4TN65UKNJPK'

let transaction = new StellarSdk.TransactionBuilder(account, {
  fee: StellarSdk.BASE_FEE,
  networkPassphrase: StellarSdk.Networks.TESTNET
})
  .addOperation(StellarSdk.Operation.setOptions({
    signer: {
      ed25519PublicKey: secondaryAddress,
      weight: 1
    }
  }))
  .addOperation(StellarSdk.Operation.setOptions({
    masterWeight: 1, // set master key weight
    lowThreshold: 1,
    medThreshold: 2, // a payment is medium threshold
    highThreshold: 2 // make sure to have enough weight to add up to the high threshold!
  }))
  .setTimeout(30)
  .build()

transaction.sign(rootKeypair) // only need to sign with the root signer as the 2nd signer won't be added to the account till after this transaction completes

// now create a payment with the account that has two signers

transaction = new StellarSdk.TransactionBuilder(account, {
  fee: StellarSdk.BASE_FEE,
  networkPassphrase: StellarSdk.Networks.TESTNET
})
  .addOperation(StellarSdk.Operation.payment({
    destination: 'GBTVUCDT5CNSXIHJTDHYSZG3YJFXBAJ6FM4CKS5GKSAWJOLZW6XX7NVC',
    asset: StellarSdk.Asset.native(),
    amount: '2000' // 2000 XLM
  }))
  .setTimeout(30)
  .build()

const transaction1 = new StellarSdk.TransactionBuilder(account, {
  fee: StellarSdk.BASE_FEE,
  networkPassphrase: StellarSdk.Networks.TESTNET
})
  .addOperation(StellarSdk.Operation.payment({
    destination: 'GBTVUCDT5CNSXIHJTDHYSZG3YJFXBAJ6FM4CKS5GKSAWJOLZW6XX7NVC',
    asset: StellarSdk.Asset.native(),
    amount: '2000' // 2000 XLM
  }))
  .setTimeout(30)
  .build()

const secondKeypair = StellarSdk.Keypair.fromSecret('SAMZUAAPLRUH62HH3XE7NVD6ZSMTWPWGM6DS4X47HLVRHEBKP4U2H5E7')

// now we need to sign the transaction with both the root and the secondaryAddress
transaction.sign(rootKeypair)
// console.log(transaction.toXDR())
transaction1.sign(secondKeypair)
// console.log(transaction.toXDR())

const transactionx = new StellarSdk.TransactionBuilder(account, {
  fee: StellarSdk.BASE_FEE,
  networkPassphrase: StellarSdk.Networks.TESTNET
})
  .addOperation(StellarSdk.Operation.payment({
    destination: 'GBTVUCDT5CNSXIHJTDHYSZG3YJFXBAJ6FM4CKS5GKSAWJOLZW6XX7NVC',
    asset: StellarSdk.Asset.native(),
    amount: '2000' // 2000 XLM
  }))
  .setTimeout(30)
  .build()

transactionx.signatures.push(...transaction.signatures)
transactionx.signatures.push(...transaction1.signatures)

transactionx.signatures.map(sig => console.log(sig.toXDR().toString('base64')))
