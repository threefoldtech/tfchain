const {
  addPublicIP
} = require('../../src/farms')

exports.command = 'add_ip <id>'
exports.desc = 'Add a public IP to a farm'
exports.builder = function (yargs) {
  yargs.option('ip', {
    description: 'Public IPV4',
    alias: 'i',
    type: 'string'
  })
  yargs.option('gateway', {
    description: 'Gateway for the public IPV4',
    alias: 'g',
    type: 'string'
  })
  yargs.demandOption(['i', 'g'])
  return yargs
}
exports.handler = addPublicIP
