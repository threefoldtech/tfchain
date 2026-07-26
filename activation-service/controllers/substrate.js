const { decodeAddress, encodeAddress } = require('@polkadot/util-crypto')
const httpError = require('http-errors')

const { client } = require('../lib/substrate')
const { activationAmount, topupAmount } = require('../lib/config')
const log = require('../lib/logger')

// An account id is a 32 byte public key. decodeAddress also accepts hex and shorter
// payloads, so the length has to be checked separately: `0x1234` decodes happily and
// used to be funded as address `25NbUg`, which nobody holds the key to.
const PUBLIC_KEY_LENGTH = 32

// The errors thrown by @threefold/tfchain_client all report `name === 'Error'`, so the
// constructor name is the only discriminator available without depending on the
// client's own transitive @threefold/types package. Anything unmapped is a 500.
const ERROR_STATUS = {
  ValidationError: 400,
  TimeoutError: 503,
  ConnectionError: 503
}

/**
 * Validate an account id and return it in canonical ss58 form.
 *
 * @throws {httpError.HttpError} 400 if the id is not a valid account id
 */
function normalizeAddress (accountID) {
  let publicKey
  try {
    publicKey = decodeAddress(accountID)
  } catch (error) {
    throw httpError(400, 'substrateAccountID is not a valid account id')
  }

  if (publicKey.length !== PUBLIC_KEY_LENGTH) {
    throw httpError(400, 'substrateAccountID is not a valid account id')
  }

  return encodeAddress(publicKey)
}

function toHttpError (error) {
  if (httpError.isHttpError(error)) return error

  const status = ERROR_STATUS[error.constructor.name] || 500

  // The decoded chain error is logged rather than left to the response, which
  // should not be the only record of it. A 4xx here means the caller sent
  // something unusable, which is not a fault on our side, so it is not an error.
  if (status >= 500) {
    log.error({ err: error }, 'chain call failed')
  } else {
    log.warn({ err: error }, 'chain call rejected the request')
  }

  return httpError(status, error.message)
}

/**
 * Submit a transfer and wait for it to be included in a block.
 *
 * apply() rejects on system.ExtrinsicFailed, so unlike the previous client a failed
 * funding transfer surfaces as an error instead of a success.
 */
async function transfer (chainClient, address, amount) {
  log.debug({ address, amount }, 'funding account')

  const extrinsic = await chainClient.balances.transfer({ address, amount })
  return extrinsic.apply()
}

/**
 * Fund an account if it is empty, or top it up if it has fallen below the floor.
 *
 * Takes the client as an argument so it can be exercised without a chain connection;
 * `activate` binds it to the shared client.
 */
async function activateWith (chainClient, body) {
  const address = normalizeAddress(body.substrateAccountID)

  try {
    const balance = await chainClient.balances.get({ address })

    if (balance.free === 0) {
      return await transfer(chainClient, address, activationAmount())
    }

    if (balance.free < topupAmount) {
      return await transfer(chainClient, address, topupAmount)
    }
  } catch (error) {
    throw toHttpError(error)
  }
}

async function activate (body) {
  return activateWith(client, body)
}

module.exports = {
  activate,
  activateWith,
  normalizeAddress
}
