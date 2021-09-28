const {
  createNameContract
} = require('../../src/contract')

exports.command = 'create <name>'
exports.desc = 'Create a name contract with a unique name'
exports.builder = {}
exports.handler = createNameContract
