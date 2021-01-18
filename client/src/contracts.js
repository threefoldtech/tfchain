const { getClient } = require('./client')

const testAccount = 'industry dismiss casual gym gap music pave gasp sick owner dumb cost'

async function createEntity (name, countryID, cityID, mnemonic, callback) {
  if (mnemonic === '') {
    mnemonic = testAccount
  }

  const client = await getClient('', mnemonic)

  const entity = await client.createEntity(name, countryID, cityID, callback)
  return entity
}

async function updateEntity (name, countryID, cityID, mnemonic, callback) {
  if (mnemonic === '') {
    mnemonic = testAccount
  }

  const client = await getClient('', mnemonic)

  const update = await client.updateEntity(name, countryID, cityID, callback)
  return update
}

async function getEntity (id) {
  const client = await getClient('', testAccount)

  const entity = await client.getEntityByID(id)
  return entity
}

async function deleteEntity (mnemonic) {
  if (mnemonic === '') {
    mnemonic = testAccount
  }

  const client = await getClient('', mnemonic)

  const res = await client.deleteEntity()
  return res
}

async function createTwin (peerID, mnemonic, callback) {
  if (mnemonic === '') {
    mnemonic = testAccount
  }

  const client = await getClient('', mnemonic)

  const create = await client.createTwin(peerID, callback)
  return create
}

async function getTwin (id) {
  const client = await getClient('', testAccount)

  const twin = await client.getTwinByID(id)
  return twin
}

async function deleteTwin (id, mnemonic, callback) {
  if (mnemonic === '') {
    mnemonic = testAccount
  }

  const client = await getClient('', mnemonic)

  const twin = await client.deleteTwinByID(id, callback)
  return twin
}

async function addTwinEntity (twinID, entityID, signature, mnemonic, callback) {
  if (mnemonic === '') {
    mnemonic = testAccount
  }

  const client = await getClient('', mnemonic)

  const create = await client.addTwinEntity(twinID, entityID, signature, callback)
  return create
}

async function removeTwinEntity (twinID, entityID, mnemonic, callback) {
  if (mnemonic === '') {
    mnemonic = testAccount
  }

  const client = await getClient('', mnemonic)

  const create = await client.removeTwinEntity(twinID, entityID, callback)
  return create
}

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
  deleteEntity,
  createTwin,
  getTwin,
  deleteTwin,
  addTwinEntity,
  removeTwinEntity
  // getFarm,
  // deleteFarm,
}
