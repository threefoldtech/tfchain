const {
  createNodeContract
} = require('../../src/contract')

exports.command = 'create'
exports.desc = 'Create a contract given a node id, workload and number of ips'
exports.builder = function (yargs) {
  yargs.option('node_id', {
    description: 'Numeric ID of the node',
    alias: 'n',
    type: 'number'
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
exports.handler = createNodeContract
