exports.command = 'sign <command>'
exports.desc = 'Sign functionality'
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
  return yargs.commandDir('sign_cmds')
}
exports.handler = function (argv) { }
