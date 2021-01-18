const { getClient } = require('./client')

const testAccount = 'industry dismiss casual gym gap music pave gasp sick owner dumb cost'

async function createEntity (name, countryID, cityID, callback) {
  const client = await getClient('', testAccount)

  const entity = await client.createEntity(name, countryID, cityID, callback)
  return entity
}

async function updateEntity (name, countryID, cityID, callback) {
  const client = await getClient('', testAccount)

  const update = await client.updateEntity(name, countryID, cityID, callback)
  return update
}

async function getEntity (id) {
  const client = await getClient('', testAccount)

  const entity = await client.getEntityByID(id)
  return entity
}

async function deleteEntity () {
  const client = await getClient('', testAccount)

  const res = await client.deleteEntity()
  return res
}

// async function createTwin (peerID, callback) {
//   const api = await getApiClient()
//   const keyring = new Keyring({ type: 'sr25519' })
//   const Alice = keyring.addFromUri('//Alice', { name: 'Alice default' })

//   return api.tx.templateModule
//     .createTwin(peerID)
//     .signAndSend(Alice, callback)
// }

// async function addTwinEntity (twinID, entityID, signature, callback) {
//   const api = await getApiClient()
//   const keyring = new Keyring({ type: 'sr25519' })
//   const Alice = keyring.addFromUri('//Alice', { name: 'Alice default' })

//   return api.tx.templateModule
//     .addTwinEntity(twinID, entityID, signature)
//     .signAndSend(Alice, callback)
// }

// async function removeTwinEntity (twinID, entityID, callback) {
//   const api = await getApiClient()
//   const keyring = new Keyring({ type: 'sr25519' })
//   const Alice = keyring.addFromUri('//Alice', { name: 'Alice default' })

//   return api.tx.templateModule
//     .deleteTwinEntity(twinID, entityID)
//     .signAndSend(Alice, callback)
// }

// async function getTwin (id) {
//   const api = await getApiClient()
//   const twin = await api.query.templateModule.twins(id)

//   const res = twin.toJSON()
//   res.peer_id = hex2a(res.peer_id)
//   return res
// }

// async function deleteTwin (twinID, callback) {
//   const api = await getApiClient()
//   const keyring = new Keyring({ type: 'sr25519' })
//   const BOB = keyring.addFromUri('//Alice', { name: 'Bob default' })

//   return api.tx.templateModule
//     .deleteTwin(twinID)
//     .signAndSend(BOB, callback)
// }

// async function createFarm (name, entityID, twinID, pricingPolicyID, certificationType, countryID, cityID, callback) {
//   const api = await getApiClient()
//   const keyring = new Keyring({ type: 'sr25519' })
//   const BOB = keyring.addFromUri('//Bob', { name: 'Bob default' })

//   certificationType = api.createType('CertificationType', certificationType)

//   return api.tx.templateModule
//     .createFarm(
//       name,
//       entityID,
//       twinID,
//       pricingPolicyID,
//       certificationType,
//       countryID,
//       cityID
//     )
//     .signAndSend(BOB, callback)
// }

// async function getFarm (id) {
//   const api = await getApiClient()
//   const farm = await api.query.templateModule.farms(id)

//   return farm.toJSON()
// }

// async function deleteFarm (farmID, callback) {
//   const api = await getApiClient()
//   const keyring = new Keyring({ type: 'sr25519' })
//   const BOB = keyring.addFromUri('//Bob', { name: 'Bob default' })

//   return api.tx.templateModule
//     .deleteFarm(farmID)
//     .signAndSend(BOB, callback)
// }

module.exports = {
  createEntity,
  updateEntity,
  getEntity,
  deleteEntity
  // createTwin,
  // getTwin,
  // deleteTwin,
  // createFarm,
  // getFarm,
  // deleteFarm,
  // addTwinEntity,
  // removeTwinEntity
}
