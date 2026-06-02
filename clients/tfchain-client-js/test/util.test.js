const { test } = require('node:test')
const assert = require('node:assert')
const { hex2a } = require('../lib/util')

test('hex2a decodes a 0x-prefixed hex string to ASCII', () => {
  // "test" = 0x74657374
  assert.strictEqual(hex2a('0x74657374'), 'test')
})

test('hex2a decodes a bare (no-0x) hex string to ASCII', () => {
  assert.strictEqual(hex2a('74657374'), 'test')
})

test('hex2a returns empty string for undefined (absent field on current runtime)', () => {
  // Regression for #1090: twin/node/farm read wrappers call hex2a on fields
  // that no longer exist on the current runtime; this must not throw.
  assert.strictEqual(hex2a(undefined), '')
})

test('hex2a returns empty string for null (Option field decoded as null)', () => {
  assert.strictEqual(hex2a(null), '')
})

test('hex2a returns empty string for empty input', () => {
  assert.strictEqual(hex2a(''), '')
  assert.strictEqual(hex2a('0x'), '')
})
