exports.command = 'entity <command>'
exports.desc = 'Manage entities'
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
  return yargs.commandDir('entity_cmds')
}
exports.handler = function (argv) {}
