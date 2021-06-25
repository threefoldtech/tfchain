const {
  listNodes
} = require('../../src/nodes')

exports.command = 'list'
exports.desc = 'List all nodes'
exports.builder = {}
exports.handler = listNodes
