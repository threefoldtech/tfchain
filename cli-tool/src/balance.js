const { getClient } = require('./client')

async function getBalance (args) {
  const { a: url, m: mnemonic, address } = args
  const client = await getClient(url, mnemonic)

  const target = address || client.key.address

  const balance = await client.getBalanceOf(target)

  console.log(`\nAddress ${target} has ${balance.free / 1e7} TFT`)
  process.exit(0)
}

module.exports = {
  getBalance
}
