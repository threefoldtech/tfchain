const {
  getBalance
} = require('../../src/balance')

exports.command = 'get [address]'
exports.desc = 'Get balance'
exports.builder = {}
exports.handler = getBalance
