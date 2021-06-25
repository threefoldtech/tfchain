const {
  getTwin
} = require('../../src/twins')

exports.command = 'get <id>'
exports.desc = 'Get a twin'
exports.builder = {}
exports.handler = getTwin
