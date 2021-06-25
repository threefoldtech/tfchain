const {
  listEntities
} = require('../../src/entities')

exports.command = 'list'
exports.desc = 'Lists all entities'
exports.builder = {}
exports.handler = listEntities
