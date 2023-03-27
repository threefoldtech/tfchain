async function listValidators (self) {
  const validators = await self.api.query.tftBridgeModule.validators()

  return validators.toJSON()
}

async function proposeTransaction (self, transactionID, to, amount, callback) {
  amount = await self.api.createType('Balance', amount * 1e7)

  const create = self.api.tx.tftBridgeModule.proposeTransaction(transactionID, to, amount)
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return create.signAndSend(self.key, { nonce }, callback)
}

async function voteTransaction (self, transactionID, callback) {
  const create = self.api.tx.tftBridgeModule.voteTransaction(transactionID)
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return create.signAndSend(self.key, { nonce }, callback)
}

module.exports = {
  proposeTransaction,
  voteTransaction,
  listValidators
}
