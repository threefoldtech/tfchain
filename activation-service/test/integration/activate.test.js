const assert = require('node:assert')
const http = require('node:http')
const { after, before, describe, it } = require('node:test')

const { derivePool, sweepPool } = require('./pool')

const { MNEMONIC, URL } = process.env

// Runs against a live chain and needs a funded account, so it is opt-in: the CI job
// only sets these when the funder secret is available.
const enabled = Boolean(MNEMONIC && URL)

describe('POST /activation/activate against a live chain', { skip: enabled ? false : 'MNEMONIC and URL are required' }, () => {
  let client, init, activationAmount, topupAmount, app
  let server, baseUrl, pool

  const balanceOf = async address => (await client.balances.get({ address })).free

  const activate = async substrateAccountID => {
    const res = await fetch(`${baseUrl}/activation/activate`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ substrateAccountID })
    })
    return res
  }

  before(async () => {
    // Required lazily: these modules read process.env when first loaded, and the
    // describe above must be able to skip without them being present.
    ;({ client, init } = require('../../lib/substrate'))
    ;({ activationAmount, topupAmount } = require('../../lib/config'))
    app = require('../../index')

    await init()

    // Sweep on the way in as well as out, so a previous run that died before its
    // cleanup does not leave the pool funded and this run's assertions intact.
    const reclaimed = await sweepPool(client, MNEMONIC)
    if (reclaimed.length) console.log(`reclaimed pool accounts from a previous run: ${reclaimed}`)

    pool = await derivePool(MNEMONIC)

    server = http.createServer(app)
    await new Promise(resolve => server.listen(0, resolve))
    baseUrl = `http://127.0.0.1:${server.address().port}`
  })

  after(async () => {
    if (server) await new Promise(resolve => server.close(resolve))
    if (client && client.api) {
      await sweepPool(client, MNEMONIC)
      await client.disconnect()
    }
  })

  // These two cases run in order on purpose: the first leaves the account funded,
  // which is precisely the state the second one needs.
  it('funds an empty account with the configured amount', async () => {
    const { address } = pool[0]
    assert.strictEqual(await balanceOf(address), 0, 'pool account should start empty')

    const res = await activate(address)

    assert.strictEqual(res.status, 200)
    assert.strictEqual(await balanceOf(address), activationAmount())
  })

  it('does not transfer again once the account is above the floor', async () => {
    const { address } = pool[0]
    const before = await balanceOf(address)
    assert.ok(before >= topupAmount, 'previous case should have funded above the floor')

    const res = await activate(address)

    assert.strictEqual(res.status, 200)
    assert.strictEqual(await balanceOf(address), before, 'balance must be unchanged')
  })

  it('rejects a malformed account id with 400', async () => {
    // Before this migration an invalid id produced a 500, because the constructed
    // httpError(400) was never thrown.
    const res = await activate('not-an-address')

    assert.strictEqual(res.status, 400)
  })

  it('rejects a short hex payload rather than funding a derived address', async () => {
    const res = await activate('0x1234')

    assert.strictEqual(res.status, 400)
  })
})
