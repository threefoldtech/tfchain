const {
  createContract
} = require('../../src/contract')

exports.command = 'create'
exports.desc = 'Create a contract given a twinID, nodeID workload and number of ips'
exports.builder = function (yargs) {
  yargs.option('twinID', {
    description: 'Id of the twin',
    alias: 't',
    type: 'number'
  })
  yargs.option('nodeID', {
    description: 'Public key of the node',
    alias: 'n',
    type: 'string'
  })
  yargs.option('workload', {
    description: 'Workload data',
    alias: 'w',
    type: 'string'
  })
  yargs.option('publicIPs', {
    description: 'Number of public ips to reserve',
    alias: 'p',
    type: 'number'
  })
  yargs.demandOption(['t', 'n', 'w', 'p'])
  return yargs
}
exports.handler = createContract
