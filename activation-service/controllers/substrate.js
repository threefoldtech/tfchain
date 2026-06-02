const { client } = require('../lib/substrate')
const httpError = require('http-errors')
// const whitelist = require('../whitelist.json')
// const { KYC_PUBLIC_KEY } = process.env

const AMOUNT = 1000000

async function activate (body) {
  const { substrateAccountID } = body

  let keyring
  try {
    keyring = client.keyring.addFromAddress(substrateAccountID)
  } catch (error) {
    httpError(400)
  }

  console.log(`amount: ${AMOUNT}`)

  const balance = await client.getBalanceOf(keyring.address)

  if (balance.free === 0) {
    try {
      return await client.transfer(keyring.address, AMOUNT)
    } catch (error) {
      throw httpError(error)
    }
  }

  if (balance.free < 15000) {
    return await client.transfer(keyring.address, 15000)
  }
}

// async function validateActivation (body) {
//   const { kycSignature, data, substrateAccountID } = body

//   // allow whitelisted users to be funded whenever they want
//   if (whitelist.includes(substrateAccountID)) {
//     try {
//       await client.transfer(substrateAccountID, AMOUNT)
//     } catch (error) {
//       throw httpError(error)
//     }
//     return
//   }

//   const { email, name: identifier } = data
//   const originalData = `{ "email": "${email}", "identifier": "${identifier}" }`

//   try {
//     const buff = Buffer.from(kycSignature, 'base64')
//     const sig = take(buff, 64)

//     const valid = await client.verify(originalData, sig, KYC_PUBLIC_KEY)
//     if (!valid) throw httpError('signature is not valid')
//   } catch (error) {
//     throw httpError('failed to verify signature')
//   }
// }

module.exports = {
  activate
}
