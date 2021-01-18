const { hex2a } = require('./util')

// createEntity creates an entity with given name
async function createEntity (self, name, countryID, cityID, callback) {
  const create = self.api.tx.templateModule.createEntity(name, countryID, cityID)
  return create.signAndSend(self.key, callback)
}

// updateEntity updates the entity linked to this signing key
async function updateEntity (self, name, countryID, cityID, callback) {
  const update = self.api.tx.templateModule.updateEntity(name, countryID, cityID)
  return update.signAndSend(self.key, callback)
}

// getEntity gets an entity by id
async function getEntity (self, id) {
  if (!id) {
    throw Error('You must pass and ID')
  }

  const entity = await self.api.query.templateModule.entities(id)

  const res = entity.toJSON()
  res.name = hex2a(res.name)
  return res
}

// deleteEntity deletes the entity linked to this signing key
async function deleteEntity (self, callback) {
  return self.api.tx.templateModule
    .deleteEntity()
    .signAndSend(self.key, callback)
}

module.exports = {
  createEntity,
  updateEntity,
  getEntity,
  deleteEntity
}
