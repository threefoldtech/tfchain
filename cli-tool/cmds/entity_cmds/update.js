const {
  updateEntity
} = require('../../src/entities')

exports.command = 'update <name>'
exports.desc = 'Update an entity'
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
  yargs.demandOption(['c', 't'])
  return yargs
}
exports.handler = updateEntity
