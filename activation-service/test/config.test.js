const assert = require('node:assert')
const { describe, it } = require('node:test')

const {
  parseActivationAmount,
  DEFAULT_ACTIVATION_TFT,
  UNITS_PER_TFT
} = require('../lib/config')

describe('parseActivationAmount', () => {
  it('defaults to the amount the service funded before it was configurable', () => {
    const expected = DEFAULT_ACTIVATION_TFT * UNITS_PER_TFT
    assert.strictEqual(parseActivationAmount(undefined), expected)
    assert.strictEqual(parseActivationAmount(''), expected)
    assert.strictEqual(expected, 1000000, '0.1 TFT is 1000000 base units')
  })

  it('reads the value as whole TFT, as readme.md and the chart document it', () => {
    // The chart default is 1, and it means one whole TFT.
    assert.strictEqual(parseActivationAmount('1'), 10000000)
  })

  it('accepts decimals down to 1e-7 TFT', () => {
    assert.strictEqual(parseActivationAmount('0.1'), 1000000)
    assert.strictEqual(parseActivationAmount('0.0000001'), 1)
    assert.strictEqual(parseActivationAmount('2.5'), 25000000)
  })

  for (const bad of ['0', '-1', 'abc', 'NaN', 'Infinity', '1e7x', '']) {
    if (bad === '') continue
    it(`rejects ${JSON.stringify(bad)}`, () => {
      assert.throws(() => parseActivationAmount(bad), /ACTIVATION_AMOUNT/)
    })
  }

  it('rejects an amount finer than the chain can represent', () => {
    assert.throws(
      () => parseActivationAmount('0.00000001'),
      /more than 7 decimals/
    )
  })

  it('leaves the existential deposit check to startup', () => {
    // 0.00000001 TFT would be below the deposit, but parsing cannot know a chain
    // constant. lib/substrate.js checks it after connecting; keep the split so
    // neither side grows a hardcoded deposit value.
    assert.strictEqual(parseActivationAmount('0.0000002'), 2)
  })
})
