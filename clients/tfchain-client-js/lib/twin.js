const { hex2a } = require('./util')

// createTwin creates an entity with given name
async function createTwin (self, relay, pk, callback) {
  const create = self.api.tx.tfgridModule.createTwin(relay, pk)
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return create.signAndSend(self.key, { nonce }, callback)
}

// updateTwin creates an entity with given name
async function updateTwin (self, relay, pk, callback) {
  const create = self.api.tx.tfgridModule.updateTwin(relay, pk)
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return create.signAndSend(self.key, { nonce }, callback)
}

// addTwinEntity adds an entity to a twin object
// the signature is a signature provided by the entity that is added.
// the signature is composed of twinID-entityID as bytes signed by the entity's private key
// to proof that he in fact approved to be part of this twin
async function addTwinEntity (self, twinID, entityID, signature, callback) {
  const create = self.api.tx.tfgridModule.addTwinEntity(twinID, entityID, signature)
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return create.signAndSend(self.key, { nonce }, callback)
}

// deleteTwinEntity delets an entity from a twin
async function deleteTwinEntity (self, twinID, entityID, callback) {
  const remove = self.api.tx.tfgridModule.deleteTwinEntity(twinID, entityID)
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return remove.signAndSend(self.key, { nonce }, callback)
}

// getTwin gets a twin by id
async function getTwin (self, id) {
  try {
    id = parseInt(id)
  } catch (error) {
    throw Error('ID must be an integer')
  }
  if (isNaN(id) || id === 0) {
    throw Error('You must pass a valid ID')
  }

  const twin = await self.api.query.tfgridModule.twins(id)

  const res = twin.toJSON()
  res.ip = hex2a(res.ip)
  if (res.id !== id) {
    throw Error('No such twin')
  }
  return res
}

async function getTwinIdByAccountId (self, accountId) {
  const res = await self.api.query.tfgridModule.twinIdByAccountID(accountId)
  const twinId = res.toJSON()
  if (twinId === 0) {
    throw Error(`Couldn't find a twin id for this account id: ${accountId}`)
  }
  return twinId
}

async function listTwins (self) {
  const twins = await self.api.query.tfgridModule.twins.entries()

  const parsedTwins = twins.map(twin => {
    const parsedTwin = twin[1].toJSON()
    parsedTwin.ip = hex2a(parsedTwin.ip)

    return parsedTwin
  })

  return parsedTwins
}

// deleteTwin deletes the twin linked to this signing key
async function deleteTwin (self, id, callback) {
  const twin = await getTwin(self, id)
  if (parseInt(twin.id) !== parseInt(id)) {
    throw Error(`twin with id ${id} does not exist`)
  }

  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return self.api.tx.tfgridModule
    .deleteTwin(id)
    .signAndSend(self.key, { nonce }, callback)
}

module.exports = {
  createTwin,
  updateTwin,
  getTwin,
  getTwinIdByAccountId,
  deleteTwin,
  addTwinEntity,
  deleteTwinEntity,
  listTwins
}
