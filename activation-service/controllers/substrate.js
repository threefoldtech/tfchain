const { client } = require('../lib/substrate')
const httpError = require('http-errors')
const { first } = require('lodash')
const SUBSTRATE_ERRORS = require('../lib/errors')
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

async function createEntity (body, res, next) {
  const { target, name, signature, countryID, cityID } = body

  let keyring
  try {
    keyring = client.keyring.addFromAddress(target)
  } catch (error) {
    res.write(error.toString())
    return res.end()
  }

  const entityByName = await client.getEntityIDByName(name)
  if (entityByName !== 0) {
    res.write('conflict')
    return res.end()
    // throw httpError(409)
  }

  const entityByPubkey = await client.getEntityIDByPubkey(keyring.address)
  if (entityByPubkey !== 0) {
    res.write('conflict')
    return res.end()
    // throw httpError(409)
  }

  try {
    await client.createEntity(keyring.address, name, countryID, cityID, signature, result => {
      if (result instanceof Error) {
        console.log(result)
        return
      }
      const { events = [], status } = result
      console.log(`Current status is ${status.type}`)
      res.write(status.type)
      if (status.type === 'Invalid') {
        res.end()
      }
      if (status.isFinalized) {
        events.forEach(({ phase, event: { data, method, section } }) => {
          if (section === 'system' && method === 'ExtrinsicFailed') {
            console.log(`\t' ${phase}: ${section}.${method}:: ${data}`)
            const module = first(data).asModule
            const errIndex = first(module.error.words)
            res.write(SUBSTRATE_ERRORS[errIndex])
            res.end()
          } else if (section === 'system' && method === 'ExtrinsicSuccess') {
            res.write('Success')
            res.end()
          }
        })
      }
    })
  } catch (error) {
    throw httpError(error.toString())
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
  activate,
  createEntity
}
