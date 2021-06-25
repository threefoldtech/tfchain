const {
  deleteFarm
} = require('../../src/farms')

exports.command = 'delete <id>'
exports.desc = 'Delete a farm by id'
exports.builder = {}
exports.handler = deleteFarm
