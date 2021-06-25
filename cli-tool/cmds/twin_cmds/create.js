const {
  createTwin
} = require('../../src/twins')

exports.command = 'create <ip>'
exports.desc = 'Create a twin with an IP'
exports.builder = {}
exports.handler = createTwin
