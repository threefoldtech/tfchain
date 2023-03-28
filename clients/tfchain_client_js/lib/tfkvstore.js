const { hex2a } = require('./util')

// set value in Threefold key-value store
async function tfStoreSet (self, key, value, callback) {
  try {
    // const create = self.api.tx.tfkvStore.set(key, value)
    const create = self.api.tx.tfkvStore.set(key, value)
    const nonce = await self.api.rpc.system.accountNextIndex(self.address)
    return create.signAndSend(self.key, { nonce }, callback)
  } catch (e) { console.log(e) }
}
// get a value with a key
async function tfStoreGet (self, key) {
  const value = await self.api.query.tfkvStore.tfkvStore(self.address, key)

  return hex2a(value.toString())
}

// list keys
async function tfStoreList (self) {
  const res = await self.api.query.tfkvStore.tfkvStore.entries(self.address)
  const keys = []
  for (const key of res) {
    keys.push(key[0].toHuman()[1])
  }
  return keys
}

// remove value from Threefold key-value store
async function tfStoreRemove (self, key, callback) {
  const remove = self.api.tx.tfkvStore.delete(key)
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return remove.signAndSend(self.key, { nonce }, callback)
}

module.exports = {
  tfStoreSet,
  tfStoreGet,
  tfStoreList,
  tfStoreRemove
}
