const Client = require('tfgrid-api-client')

const {
  SUBSTRATE_API_URL,
  MNEMONIC
} = process.env

async function getClient (url, mnemonic) {
  if (!url) {
    url = SUBSTRATE_API_URL
  }

  if (!mnemonic) {
    mnemonic = MNEMONIC
  }

  const cli = new Client(url, mnemonic)

  try {
    await cli.init()
  } catch (err) {
    return err
  }

  return cli
}

module.exports = { getClient }
