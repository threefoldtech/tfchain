const {
  updateNodeContract
} = require('../../src/contract')

exports.command = 'update <id>'
exports.desc = 'Update a contract data and hash'
exports.builder = function (yargs) {
  yargs.option('data', {
    description: 'Contract data',
    alias: 'd',
    type: 'string'
  })
  yargs.option('hash', {
    description: 'Hash of the deployment',
    alias: 'h',
    type: 'string'
  })
  yargs.demandOption(['d', 'h'])
  return yargs
}
exports.handler = updateNodeContract
