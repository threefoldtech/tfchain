const { getClient } = require('./client')

async function getBalance (args) {
  const { a: url, m: mnemonic, address } = args
  const client = await getClient(url, mnemonic)

  const balance = await client.getBalanceOf(address)

  console.log(balance)
  process.exit(0)
}

module.exports = {
  getBalance
}
