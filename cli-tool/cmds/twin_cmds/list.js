const {
  listTwins
} = require('../../src/twins')

exports.command = 'list'
exports.desc = 'Lists all twins'
exports.builder = {}
exports.handler = listTwins
