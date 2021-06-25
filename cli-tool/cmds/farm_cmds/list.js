const {
  listFarms
} = require('../../src/farms')

exports.command = 'list'
exports.desc = 'List all farms'
exports.builder = {}
exports.handler = listFarms
