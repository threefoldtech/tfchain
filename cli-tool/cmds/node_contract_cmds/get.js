const {
  getContract
} = require('../../src/contract')

exports.command = 'get <id>'
exports.desc = 'Get a contract by id'
exports.builder = {}
exports.handler = getContract
