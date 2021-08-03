const Client = require('tfgrid-api-client')

async function createMnemonic () {
  const cli = new Client('', '')

  const mnemonic = await cli.createMnemonic()

  console.log(`mnemonic: ${mnemonic}`)
  process.exit(0)
}

module.exports = {
  createMnemonic
}
