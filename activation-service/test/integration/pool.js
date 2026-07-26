const { Keyring } = require('@polkadot/keyring')
const { cryptoWaitReady } = require('@polkadot/util-crypto')

// Activation targets are derived from the funder mnemonic rather than generated
// randomly, so a run that dies before sweeping leaves behind accounts the *next*
// run can still find and reclaim. A random keypair per run would orphan its funds
// in an address nothing can derive again.
const POOL_SIZE = 5
const POOL_PATH = index => `//as-ci//${index}`

/**
 * Derive the fixed pool of activation targets from the funder mnemonic.
 *
 * @returns {Promise<Array<{index: number, address: string, pair: object}>>}
 */
async function derivePool (mnemonic, size = POOL_SIZE) {
  await cryptoWaitReady()
  const keyring = new Keyring({ type: 'sr25519' })

  return Array.from({ length: size }, (_, i) => {
    const index = i + 1
    const pair = keyring.addFromUri(`${mnemonic}${POOL_PATH(index)}`)
    return { index, address: pair.address, pair }
  })
}

/**
 * Return every pool account's funds to the funder and reap the account.
 *
 * Uses `transferAll(..., keepAlive = false)` so the account is removed rather than
 * left holding the existential deposit. @threefold/tfchain_client does not expose
 * transferAll, so this goes through the underlying api.
 *
 * Failures are reported but never thrown: sweeping is best-effort cleanup and must
 * not fail the job that called it.
 */
async function sweepPool (client, mnemonic, { log = console } = {}) {
  const pool = await derivePool(mnemonic)
  const swept = []

  for (const { index, address, pair } of pool) {
    const { data } = await client.api.query.system.account(address)
    if (data.free.toBigInt() === 0n) continue

    try {
      await new Promise((resolve, reject) => {
        client.api.tx.balances
          .transferAll(client.address, false)
          .signAndSend(pair, ({ status, dispatchError }) => {
            if (dispatchError) return reject(new Error(dispatchError.toString()))
            if (status.isInBlock) resolve()
          })
          .catch(reject)
      })
      swept.push(index)
    } catch (error) {
      log.warn?.(`failed to sweep pool account ${index} (${address}): ${error.message}`)
    }
  }

  return swept
}

module.exports = {
  POOL_SIZE,
  derivePool,
  sweepPool
}
