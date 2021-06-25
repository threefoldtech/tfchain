const {
  getBalance
} = require('../../src/balance')

exports.command = 'get'
exports.desc = 'Get your balance'
exports.builder = {}
exports.handler = getBalance
