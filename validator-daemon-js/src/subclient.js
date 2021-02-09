const Client = require('tfgrid-api-client')

async function getClient (url, mnemonic) {
  const cli = new Client(url, mnemonic)

  try {
    await cli.init()
  } catch (err) {
    console.log(err)
    return err
  }

  return cli
}

module.exports = { getClient }
