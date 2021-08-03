exports.command = 'keys <command>'
exports.desc = 'Manage keys'
exports.builder = function (yargs) {
  return yargs.commandDir('key_cmds')
}
exports.handler = function (argv) {}
