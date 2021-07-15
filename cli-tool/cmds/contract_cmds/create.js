const {
  createContract
} = require('../../src/contract')

exports.command = 'create'
exports.desc = 'Create a contract given a twinID, nodeID workload and number of ips'
exports.builder = function (yargs) {
  yargs.option('nodeID', {
    description: 'Public key of the node',
    alias: 'n',
    type: 'string'
  })
  yargs.option('data', {
    description: 'Workload data',
    alias: 'd',
    type: 'string'
  })
  yargs.option('hash', {
    description: 'Deployment hash',
    alias: 'h',
    type: 'string'
  })
  yargs.option('publicIPs', {
    description: 'Number of public ips to reserve',
    alias: 'p',
    type: 'number'
  })
  yargs.option('json', {
    description: 'Output only the json',
    alias: 'j',
    type: 'bool'
  })
  yargs.demandOption(['n', 'd', 'h', 'p'])
  return yargs
}
exports.handler = createContract
