const { getClient } = require('./client')
const { callback } = require('./util')

async function createFarm (args) {
  const { name, entityID, twinID, c: countryID, t: cityID, certificationType, pricingPolicyID, a: url, m: mnemonic } = args

  const client = await getClient(url, mnemonic)

  const certificationTypeParsed = client.api.createType('CertificationType', certificationType)
  const farm = {
    id: 0,
    name,
    entity_id: entityID,
    twin_id: twinID,
    pricingPolicyID,
    certificationType: certificationTypeParsed,
    countryID,
    cityID
  }

  return client.createFarm(farm, callback)
}

async function getFarm (args) {
  const { id, a: url } = args
  const client = await getClient(url, '')

  const farm = await client.getFarmByID(id)

  console.log(farm)
  process.exit(0)
}

async function listFarms (args) {
  const { a: url } = args
  const client = await getClient(url, '')

  const farms = await client.listFarms()

  console.log(farms)
  process.exit(0)
}

async function deleteFarm (args) {
  const { id, a: url, m: mnemonic } = args
  const client = await getClient(url, mnemonic)

  await client.deleteFarmByID(id, callback)

  console.log('farm deleted')
  process.exit(0)
}

module.exports = {
  createFarm,
  getFarm,
  listFarms,
  deleteFarm
}
