const {
  createEntity
} = require('../../src/entities')

exports.command = 'create <name>'
exports.desc = 'Create an entity given a name, countryID, cityID and optional signature'
exports.builder = function (yargs) {
  yargs.option('countryID', {
    description: 'Id of the country',
    alias: 'c',
    type: 'number'
  })
  yargs.option('cityID', {
    description: 'Id of the city',
    alias: 't',
    type: 'number'
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
