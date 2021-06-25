const {
  getFarm
} = require('../../src/farms')

exports.command = 'get <id>'
exports.desc = 'Get a farm'
exports.builder = {}
exports.handler = getFarm
