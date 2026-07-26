const assert = require('node:assert')
const { describe, it } = require('node:test')

const { Client } = require('@threefold/tfchain_client')

// A mnemonic is required for the client to load a keypair; this one is a well known
// throwaway and is never used to sign anything here.
const MNEMONIC = 'bottom drive obey lake curtain smoke basket hold race lonely fit walk'

describe('connecting to an unreachable chain', () => {
  // The timeout is the point of the test: without it a regression here would hang
  // the run rather than fail it, which is exactly how the old client behaved.
  it('rejects promptly instead of retrying forever', { timeout: 30000 }, async () => {
    // This is the regression the migration exists to fix. bin/www awaits init()
    // before server.listen(), and the previous client's ApiPromise.create() never
    // rejected — the service hung before listening, logged nothing, and never became
    // ready. Nothing is listening on this port, so connect() must reject.
    const client = new Client({
      url: 'ws://127.0.0.1:59999',
      mnemonicOrSecret: MNEMONIC,
      keypairType: 'sr25519'
    })

    const started = Date.now()
    await assert.rejects(client.connect())
    const elapsed = Date.now() - started

    // The client's own connect timeout is 10s; anything near that still proves the
    // property. The point is that it terminates at all.
    assert.ok(elapsed < 20000, `connect() took ${elapsed}ms, expected it to reject`)
  })
})
