async function getBlockHash (self, blockNumber) {
  return await self.api.rpc.chain.getBlockHash(blockNumber)
}

async function getBlockTime (self, blockHash) {
  const apiAt = await self.api.at(blockHash)
  return await apiAt.query.timestamp.now()
}

module.exports = {
  getBlockHash,
  getBlockTime
}
