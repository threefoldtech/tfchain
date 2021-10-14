const {
  createEntity
} = require('../../src/entities')

exports.command = 'create <name>'
exports.desc = 'Create an entity given a name, countryID, cityID and optional signature'
exports.builder = function (yargs) {
  yargs.option('countryID', {
    description: 'Name of the country',
    alias: 'c',
    type: 'string'
  })
  yargs.option('cityID', {
    description: 'Name of the city',
    alias: 't',
    type: 'string'
  })
  yargs.option('signature', {
    description: 'Signature for entity creation',
    alias: 's',
    type: 'string'
  })
  yargs.option('target', {
    description: 'Target address to create an entity for',
    alias: 'f',
    type: 'string'
  })
  yargs.demandOption(['c', 't'])
  return yargs
}
exports.handler = createEntity
