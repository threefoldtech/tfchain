const { getApiClient } = require('./api')
const { Keyring } = require('@polkadot/api')

async function createEntity (name, countryID, cityID, callback) {
  const api = await getApiClient()
  const keyring = new Keyring({ type: 'sr25519' })
  const BOB = keyring.addFromUri('//Bob', { name: 'Bob default' })

  return api.tx.templateModule
    .createEntity(name, countryID, cityID)
    .signAndSend(BOB, callback)
}

async function updateEntity (name, countryID, cityID, callback) {
  const api = await getApiClient()
  const keyring = new Keyring({ type: 'sr25519' })
  const BOB = keyring.addFromUri('//Bob', { name: 'Bob default' })

  return api.tx.templateModule
    .updateEntity(name, countryID, cityID)
    .signAndSend(BOB, callback)
}

async function getEntity (id) {
  const api = await getApiClient()
  const entity = await api.query.templateModule.entities(id)

  const res = entity.toJSON()
  res.name = hex2a(res.name)
  return res
}

async function deleteEntity (callback) {
  const api = await getApiClient()
  const keyring = new Keyring({ type: 'sr25519' })
  const BOB = keyring.addFromUri('//Bob', { name: 'Bob default' })

  return api.tx.templateModule
    .deleteEntity()
    .signAndSend(BOB, callback)
}

async function createTwin (callback) {
  const api = await getApiClient()
  const keyring = new Keyring({ type: 'sr25519' })
  const BOB = keyring.addFromUri('//Bob', { name: 'Bob default' })

  return api.tx.templateModule
    .createTwin()
    .signAndSend(BOB, callback)
}

async function getTwin (id) {
  const api = await getApiClient()
  const twin = await api.query.templateModule.twins(id)

  return twin.toJSON()
}

async function deleteTwin (callback) {
  const api = await getApiClient()
  const keyring = new Keyring({ type: 'sr25519' })
  const BOB = keyring.addFromUri('//Bob', { name: 'Bob default' })

  return api.tx.templateModule
    .deleteTwin()
    .signAndSend(BOB, callback)
}

async function createFarm (name, entityID, twinID, pricingPolicyID, certificationType, countryID, cityID, callback) {
  const api = await getApiClient()
  const keyring = new Keyring({ type: 'sr25519' })
  const BOB = keyring.addFromUri('//Bob', { name: 'Bob default' })

  certificationType = api.createType('CertificationType', certificationType)

  return api.tx.templateModule
    .createFarm(
      name,
      entityID,
      twinID,
      pricingPolicyID,
      certificationType,
      countryID,
      cityID
    )
    .signAndSend(BOB, callback)
}

async function getFarm (id) {
  const api = await getApiClient()
  const farm = await api.query.templateModule.farms(id)

  return farm.toJSON()
}

async function deleteFarm (farmID, callback) {
  const api = await getApiClient()
  const keyring = new Keyring({ type: 'sr25519' })
  const BOB = keyring.addFromUri('//Bob', { name: 'Bob default' })

  return api.tx.templateModule
    .deleteFarm(farmID)
    .signAndSend(BOB, callback)
}

function hex2a (hex) {
  var str = ''
  for (var i = 0; i < hex.length; i += 2) {
    var v = parseInt(hex.substr(i, 2), 16)
    if (v) str += String.fromCharCode(v)
  }
  return str
}

module.exports = {
  createEntity,
  updateEntity,
  getEntity,
  deleteEntity,
  createTwin,
  getTwin,
  deleteTwin,
  createFarm,
  getFarm,
  deleteFarm
}
