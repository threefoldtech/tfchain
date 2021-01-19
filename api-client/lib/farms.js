const { getEntity } = require('./entity')
const { getTwin } = require('./twin')
const { hex2a } = require('./util')

// createFarm creates a farm with given properties
async function createFarm (self, farm, callback) {
  try {
    await validateFarm(self, farm)
  } catch (error) {
    // callback early with error
    if (callback) {
      return callback(error)
    }
    return error
  }

  const create = self.api.tx.templateModule.createFarm(farm)
  return create.signAndSend(self.key, callback)
}

// getFarm gets a farm by id
async function getFarm (self, id) {
  if (isNaN(id)) {
    throw Error('You must pass an ID')
  }

  const farm = await self.api.query.templateModule.farms(id)

  const res = farm.toJSON()
  res.name = hex2a(res.name)
  return res
}

// deleteEntity deletes the entity linked to this signing key
async function deleteFarm (self, id, callback) {
  if (isNaN(id)) {
    throw Error('You must pass an ID')
  }

  return self.api.tx.templateModule
    .deleteFarm(id)
    .signAndSend(self.key, callback)
}

async function validateFarm (self, farm) {
  const { name, entityID, twinID } = farm
  // const { pricingPolicyID, certificationType, countryID, cityID } = farm

  if (name === '') {
    throw Error('farm should have a name')
  }

  const entity = await getEntity(self, entityID)
  if (entity.entity_id !== entityID) {
    throw Error(`entity with id ${entityID} does not exist`)
  }

  const twin = await getTwin(self, twinID)
  if (twin.twin_id !== twinID) {
    throw Error(`twin with id ${twinID} does not exist`)
  }
}

module.exports = {
  createFarm,
  getFarm,
  deleteFarm
}
