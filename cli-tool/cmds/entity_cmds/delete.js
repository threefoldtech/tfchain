const {
  deleteEntity
} = require('../../src/entities')

exports.command = 'delete'
exports.desc = 'Delete your entity'
exports.builder = {}
exports.handler = deleteEntity
