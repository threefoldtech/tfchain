exports.command = 'contract <command>'
exports.desc = 'Manage contracts'
exports.builder = function (yargs) {
  yargs.option('apiURL', {
    alias: 'a',
    description: 'Substrate API url',
    type: 'string'
  })
  yargs.option('mnemonic', {
    alias: 'm',
    description: 'Mnemonic to sign with',
    type: 'string'
  })
  // yargs.demandOption(['a', 'm'])
  return yargs.commandDir('contract_cmds')
}
exports.handler = function (argv) {}
