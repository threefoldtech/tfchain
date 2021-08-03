const {
  createMnemonic
} = require('../../src/keys')

exports.command = 'create'
exports.desc = 'Create a mnemonic'
exports.builder = {}
exports.handler = createMnemonic
