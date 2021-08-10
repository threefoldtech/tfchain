const {
  deletePublicIP
} = require('../../src/farms')

exports.command = 'delete_ip <id>'
exports.desc = 'Delete a public IP from a farm'
exports.builder = function (yargs) {
  yargs.option('ip', {
    description: 'Public IPV4',
    alias: 'i',
    type: 'string'
  })
  yargs.demandOption(['i'])
  return yargs
}
exports.handler = deletePublicIP
