const { getClient } = require('./client')

async function getBalance (args) {
  const { a: url, m: mnemonic } = args
  const client = await getClient(url, mnemonic)

  const balance = await client.getBalance()

  console.log(balance)
  process.exit(0)
}

module.exports = {
  getBalance
}
