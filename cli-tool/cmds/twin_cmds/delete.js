const {
  deleteTwin
} = require('../../src/twins')

exports.command = 'delete <id>'
exports.desc = 'Delete a twin'
exports.builder = {}
exports.handler = deleteTwin
