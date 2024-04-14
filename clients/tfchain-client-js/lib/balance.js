async function getBalance (self, address) {
  const { data: balance } = await self.api.query.system.account(address)

  return {
    free: balance.free.toJSON(),
    reserved: balance.reserved.toJSON(),
    frozen: balance.frozen.toJSON(),
  }
}

async function transfer (self, address, amount, callback) {
  if (isNaN(amount) || amount === 0) {
    throw Error('You must pass a valid numeric amount')
  }

  const transfer = self.api.tx.balances.transfer(address, amount)

  const nonce = await self.api.rpc.system.accountNextIndex(self.address)
  return transfer.signAndSend(self.key, { nonce }, callback)
}

module.exports = {
  getBalance,
  transfer
}
