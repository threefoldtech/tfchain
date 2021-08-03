const { getClient } = require('./client')

async function getBalance (args) {
  const { a: url, m: mnemonic, address } = args
  const client = await getClient(url, mnemonic)

  let balance
  if (address) {
    balance = await client.getBalanceOf(address)
  } else {
    balance = await client.getBalance()
  }

  console.log(balance)
  process.exit(0)
}

module.exports = {
  getBalance
}
