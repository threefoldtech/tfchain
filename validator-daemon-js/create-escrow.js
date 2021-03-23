const StellarSdk = require('stellar-sdk')

const rootKeypair = StellarSdk.Keypair.fromSecret('SAZ2Q6Q2PDKEJLV7DPG75S7N7SNY6DPSJY2ILN5O6WALJZEPJ5FGCJW2')
const sourcePublicKey = rootKeypair.publicKey()
console.log(sourcePublicKey)

const v1 = 'GDNMP6Q6LR5LYARNCOIQUWJABX37F26NP7ULD6D4BZGWW4WZSNICZNMB'
const v2 = 'GDP4XCSCDHDZLTAI3MUVTRU3EO4WBHLERUZT75AUM6O2LCSDQ4JNFNTR'
const v3 = 'GCHBTKV5VKKV6TUORHLASVDTJ336QTT24YOXAEK433SZN5VXJK5NRLAF'
const v4 = 'GBJNADYGQKFRLZZLKJUTD2ECTX65TV35B7LTMKLC6KWH5QSQL3ZKLBEW'
const v5 = 'GBX2GPH27S32GQRDTIKGCGLEMEIW4YLJCQSRVM5NX7WEFANS3UUBYY3M'

async function main () {
  const server = new StellarSdk.Server('https://horizon-testnet.stellar.org')

  const account = await server.loadAccount(sourcePublicKey)

  const transaction = new StellarSdk.TransactionBuilder(account, {
    fee: StellarSdk.BASE_FEE,
    networkPassphrase: StellarSdk.Networks.TESTNET
  })
    .addOperation(StellarSdk.Operation.setOptions({
      signer: {
        ed25519PublicKey: v1,
        weight: 1
      }
    }))
    .addOperation(StellarSdk.Operation.setOptions({
      signer: {
        ed25519PublicKey: v2,
        weight: 1
      }
    }))
    .addOperation(StellarSdk.Operation.setOptions({
      signer: {
        ed25519PublicKey: v3,
        weight: 1
      }
    }))
    .addOperation(StellarSdk.Operation.setOptions({
      signer: {
        ed25519PublicKey: v4,
        weight: 1
      }
    }))
    .addOperation(StellarSdk.Operation.setOptions({
      signer: {
        ed25519PublicKey: v5,
        weight: 1
      }
    }))
    .addOperation(StellarSdk.Operation.setOptions({
      masterWeight: 5, // set master key weight
      lowThreshold: 8,
      medThreshold: 8, // a payment is medium threshold
      highThreshold: 8 // make sure to have enough weight to add up to the high threshold!
    }))
    .addOperation(StellarSdk.Operation.changeTrust({
      asset: new StellarSdk.Asset('TFT', 'GA47YZA3PKFUZMPLQ3B5F2E3CJIB57TGGU7SPCQT2WAEYKN766PWIMB3'),
      limit: '922337203685.4775807'
    }))
    .setTimeout(30)
    .build()

  transaction.sign(rootKeypair) // only need to sign with the root signer as the 2nd signer won't be added to the account till after this transaction completes

  try {
    const transactionResult = await server.submitTransaction(transaction)
    console.log(JSON.stringify(transactionResult, null, 2))
    console.log('\nSuccess! View the transaction at: ')
    console.log(transactionResult._links.transaction.href)
  } catch (e) {
    console.log('An error has occured:')
    console.log(e.response.data.extras)
  }
}

main()
