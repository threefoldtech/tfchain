const {
  createFarm
} = require('../../src/farms')

exports.command = 'create <name>'
exports.desc = 'Create a farm given parameters'
exports.builder = function (yargs) {
  yargs.option('entityID', {
    description: 'EntityID for the farm',
    alias: 'e',
    type: 'number'
  })
  yargs.option('twinID', {
    description: 'TwinId for the farm',
    alias: 't',
    type: 'number'
  })
  yargs.option('countryID', {
    description: 'Id of the country',
    alias: 'c',
    type: 'number'
  })
  yargs.option('cityID', {
    description: 'Id of the city',
    alias: 'g',
    type: 'number'
  })
  yargs.option('certificationType', {
    description: 'Farm certification type (bronze, silver, gold)',
    alias: 'y',
    type: 'string'
  })
  yargs.option('pricingPolicyID', {
    description: 'PricingPolicyID for the farm',
    alias: 'p',
    type: 'number'
  })
  yargs.demandOption(['e', 't', 'c', 'g', 'y', 'p'])
  return yargs
}
exports.handler = createFarm
