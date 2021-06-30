const {
  updateNode
} = require('../../src/nodes')

exports.command = 'update <id>'
exports.desc = 'Update a node by id given parameters'
exports.builder = function (yargs) {
  yargs.option('twinID', {
    description: 'TwinID for the node',
    alias: 't',
    type: 'number'
  })
  yargs.option('farmID', {
    description: 'FarmID for the node',
    alias: 'f',
    type: 'number'
  })
  yargs.demandOption(['t', 'f'])
  return yargs
}
exports.handler = updateNode
