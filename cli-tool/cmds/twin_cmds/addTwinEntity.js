const {
  createTwinEntity
} = require('../../src/twins')

exports.command = 'createTwinEntity <id>'
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
  yargs.option('signature', {
    description: 'Signature for twin-entity relation creation, can be produced by signing the entityID and twinID',
    alias: 's',
    type: 'string'
  })
  yargs.demandOption(['t', 'e', 's'])
  return yargs
}
exports.handler = createTwinEntity
