const {
  cancelContract
} = require('../../src/contract')

exports.command = 'cancel <id>'
exports.desc = 'Cancel a contract by id'
exports.builder = {}
exports.handler = cancelContract
