const { hex2a } = require('./util')

// createEntity creates an entity with given name
async function createEntity (self, target, name, countryID, cityID, signature, callback) {
  const create = self.api.tx.tfgridModule.createEntity(target, name, countryID, cityID, signature)
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return create.signAndSend(self.key, { nonce }, callback)
}

// updateEntity updates the entity linked to this signing key
async function updateEntity (self, name, countryID, cityID, callback) {
  const update = self.api.tx.tfgridModule.updateEntity(name, countryID, cityID)
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return update.signAndSend(self.key, { nonce }, callback)
}

// getEntity gets an entity by id
async function getEntity (self, id) {
  try {
    id = parseInt(id)
  } catch (error) {
    throw Error('ID must be an integer')
  }
  if (isNaN(id) || id === 0) {
    throw Error('You must pass a valid ID')
  }

  const entity = await self.api.query.tfgridModule.entities(id)

  const res = entity.toJSON()
  if (res.id !== id) {
    throw Error('No such entity')
  }

  res.name = hex2a(res.name)
  return res
}

async function getEntityIDByName (self, name) {
  const entity = await self.api.query.tfgridModule.entitiesByNameID(name)

  return entity.toJSON()
}

async function getEntityIDByPubkey (self, pubkey) {
  const entity = await self.api.query.tfgridModule.entitiesByPubkeyID(pubkey)

  return entity.toJSON()
}

async function listEntities (self) {
  const entities = await self.api.query.tfgridModule.entities.entries()

  const parsedEntities = entities.map(entity => {
    const parsedEntity = entity[1].toJSON()
    parsedEntity.name = hex2a(parsedEntity.name)

    return parsedEntity
  })

  return parsedEntities
}

// deleteEntity deletes the entity linked to this signing key
async function deleteEntity (self, callback) {
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return self.api.tx.tfgridModule
    .deleteEntity()
    .signAndSend(self.key, { nonce }, callback)
}

module.exports = {
  createEntity,
  updateEntity,
  getEntity,
  deleteEntity,
  listEntities,
  getEntityIDByName,
  getEntityIDByPubkey
}
