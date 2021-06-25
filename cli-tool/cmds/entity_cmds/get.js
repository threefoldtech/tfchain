const {
  getEntity
} = require('../../src/entities')

exports.command = 'get <id>'
exports.desc = 'Get an entity'
exports.builder = {}
exports.handler = getEntity
