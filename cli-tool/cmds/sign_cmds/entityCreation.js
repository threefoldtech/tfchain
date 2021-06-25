const {
  signEntityCreation
} = require('../../src/sign')

exports.command = 'signEntityCreation'
exports.desc = 'Sign the creation of an entity'
exports.builder = function (yargs) {
  yargs.option('name', {
    description: 'Entity name',
    alias: 'n',
    type: 'string'
  })
  yargs.option('countryID', {
    description: 'Entity country ID',
    alias: 'c',
    type: 'string'
  })
  yargs.option('cityID', {
    description: 'Entity city ID',
    alias: 't',
    type: 'string'
  })
  yargs.demandOption(['n', 'c', 't'])
  return yargs
}
exports.handler = signEntityCreation
