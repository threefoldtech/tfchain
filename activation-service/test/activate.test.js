const assert = require('node:assert')
const { describe, it } = require('node:test')

const { activateWith, normalizeAddress } = require('../controllers/substrate')
const { activationAmount, topupAmount } = require('../lib/config')

const ADDRESS = '5GNZHEi9YvT3cFpSysD3HZa1AEbmq7w25dTqu68FJWDhatod'

/**
 * Minimal stand-in for @threefold/tfchain_client. Records the transfers it was asked
 * to make so the funding branches can be asserted without a chain.
 */
function fakeClient ({ free = 0, applyError, getError } = {}) {
  const transfers = []

  return {
    transfers,
    balances: {
      get: async () => {
        if (getError) throw getError
        return { free, reserved: 0, frozen: 0 }
      },
      transfer: async ({ address, amount }) => {
        transfers.push({ address, amount })
        return {
          apply: async () => {
            if (applyError) throw applyError
            return amount
          }
        }
      }
    }
  }
}

function chainError (name, message = 'boom') {
  // The client's errors are distinguished by constructor name, not by `name`,
  // which is always 'Error'. Reproduce that shape exactly.
  const Ctor = { [name]: class extends Error {} }[name]
  return new Ctor(message)
}

describe('normalizeAddress', () => {
  it('accepts an ss58 address and returns it canonicalised', () => {
    assert.strictEqual(normalizeAddress(ADDRESS), ADDRESS)
  })

  it('accepts a 32 byte hex public key', () => {
    const hex = '0x' + '11'.repeat(32)
    assert.strictEqual(typeof normalizeAddress(hex), 'string')
  })

  // '0x1234' is the interesting one: the previous client accepted it and funded the
  // derived address 25NbUg, which nobody holds the key to.
  for (const bad of ['', 'not-an-address', '0x1234', ADDRESS.slice(0, -1)]) {
    it(`rejects ${JSON.stringify(bad)} with a 400`, () => {
      assert.throws(() => normalizeAddress(bad), err => {
        assert.strictEqual(err.status, 400)
        return true
      })
    })
  }
})

describe('activate', () => {
  it('funds the activation amount when the account is empty', async () => {
    const client = fakeClient({ free: 0 })

    await activateWith(client, { substrateAccountID: ADDRESS })

    assert.deepStrictEqual(client.transfers, [{ address: ADDRESS, amount: activationAmount() }])
  })

  it('funds the amount ACTIVATION_AMOUNT asks for, in whole TFT', async () => {
    // The variable was required but ignored before this; 1 means one whole TFT.
    const previous = process.env.ACTIVATION_AMOUNT
    process.env.ACTIVATION_AMOUNT = '1'
    try {
      const client = fakeClient({ free: 0 })

      await activateWith(client, { substrateAccountID: ADDRESS })

      assert.deepStrictEqual(client.transfers, [{ address: ADDRESS, amount: 10000000 }])
    } finally {
      if (previous === undefined) delete process.env.ACTIVATION_AMOUNT
      else process.env.ACTIVATION_AMOUNT = previous
    }
  })

  it('tops up when the balance is below the floor', async () => {
    const client = fakeClient({ free: topupAmount - 1 })

    await activateWith(client, { substrateAccountID: ADDRESS })

    assert.deepStrictEqual(client.transfers, [{ address: ADDRESS, amount: topupAmount }])
  })

  it('does nothing when the balance is at or above the floor', async () => {
    const client = fakeClient({ free: topupAmount })

    await activateWith(client, { substrateAccountID: ADDRESS })

    assert.deepStrictEqual(client.transfers, [])
  })

  it('rejects an invalid address with a 400 before touching the chain', async () => {
    const client = fakeClient({ free: 0 })

    await assert.rejects(
      activateWith(client, { substrateAccountID: 'nonsense' }),
      err => err.status === 400
    )
    assert.deepStrictEqual(client.transfers, [], 'must not transfer for a bad address')
  })

  it('propagates a failed extrinsic instead of reporting success', async () => {
    // The previous client resolved on submission, so a failed funding transfer
    // still returned 200. apply() rejecting must surface as an error.
    const client = fakeClient({ free: 0, applyError: chainError('TFChainError', 'ExtrinsicFailed') })

    await assert.rejects(
      activateWith(client, { substrateAccountID: ADDRESS }),
      err => err.status === 500
    )
  })

  it('maps a client ValidationError to 400', async () => {
    const client = fakeClient({ free: 0, applyError: chainError('ValidationError') })

    await assert.rejects(
      activateWith(client, { substrateAccountID: ADDRESS }),
      err => err.status === 400
    )
  })

  for (const name of ['ConnectionError', 'TimeoutError']) {
    it(`maps a client ${name} to 503`, async () => {
      const client = fakeClient({ free: 0, getError: chainError(name) })

      await assert.rejects(
        activateWith(client, { substrateAccountID: ADDRESS }),
        err => err.status === 503
      )
    })
  }
})
