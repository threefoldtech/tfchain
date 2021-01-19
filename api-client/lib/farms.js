const { getEntity } = require('./entity')
const { getTwin } = require('./twin')
const { hex2a } = require('./util')

// createFarm creates a farm with given properties
async function createFarm (self, farm, callback) {
  try {
    await validateFarm(self, farm)
  } catch (error) {
    // callback early with error
    return callback(error)
  }

  const create = self.api.tx.templateModule.createFarm(farm)
  return create.signAndSend(self.key, callback)
}

// getFarm gets a farm by id
async function getFarm (self, id) {
  if (!id) {
    throw Error('You must pass and ID')
  }

  const farm = await self.api.query.templateModule.farms(id)

  const res = farm.toJSON()
  res.name = hex2a(res.name)
  return res
}

async function validateFarm (self, farm) {
  const { name, entityID, twinID } = farm
  // const { pricingPolicyID, certificationType, countryID, cityID } = farm

  if (name === '') {
    throw Error('farm should have a name')
  }

  const entity = await getEntity(self, entityID)
  if (entity.entityID !== entityID) {
    throw Error('entity does not exist')
  }

  const twin = await getTwin(self, twinID)
  console.log(twin)
  if (!twin.twinID !== twinID) {
    throw Error('twin does not exist')
  }
}

module.exports = {
  createFarm,
  getFarm
}
