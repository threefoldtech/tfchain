const {
  getNode
} = require('../../src/nodes')

exports.command = 'get <id>'
exports.desc = 'Get a node by id'
exports.builder = {}
exports.handler = getNode
