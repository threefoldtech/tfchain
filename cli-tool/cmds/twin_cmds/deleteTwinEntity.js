const {
  deleteTwinEntity
} = require('../../src/twins')

exports.command = 'deleteTwinEntity <id>'
exports.desc = 'Delete a twin entity relation'
exports.builder = function (yargs) {
  yargs.option('twinID', {
    description: 'TwinID for the twin-entity relation',
    alias: 't',
    type: 'number'
  })
  yargs.option('entityID', {
    description: 'EntityID for the twin-entity relation',
    alias: 'e',
    type: 'number'
  })
  yargs.demandOption(['t', 'e'])
  return yargs
}
exports.handler = deleteTwinEntity
