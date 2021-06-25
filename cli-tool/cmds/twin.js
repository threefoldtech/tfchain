exports.command = 'twin <command>'
exports.desc = 'Manage twins'
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
  return yargs.commandDir('twin_cmds')
}
exports.handler = function (argv) { }
