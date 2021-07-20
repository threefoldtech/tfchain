const {
  createNode
} = require('../../src/nodes')

exports.command = 'create'
exports.desc = 'Create a node given parameters'
exports.builder = function (yargs) {
  yargs.option('farmID', {
    description: 'FarmID for the node',
    alias: 'f',
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
  yargs.demandOption(['f', 'c', 'g'])
  return yargs
}
exports.handler = createNode
