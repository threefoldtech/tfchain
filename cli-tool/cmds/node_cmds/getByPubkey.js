const {
  getNodeByPubkey
} = require('../../src/nodes')

exports.command = 'getByPubkey <pubkey>'
exports.desc = 'Get a node id by pubkey'
exports.builder = {}
exports.handler = getNodeByPubkey
