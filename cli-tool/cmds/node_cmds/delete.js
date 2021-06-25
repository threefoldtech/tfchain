const {
  deleteNode
} = require('../../src/nodes')

exports.command = 'delete <id>'
exports.desc = 'Delete a node by id'
exports.builder = {}
exports.handler = deleteNode
