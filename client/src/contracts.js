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

async function createFarm (name, entityID, twinID, pricingPolicyID, certificationType, countryID, cityID, mnemonic, callback) {
  // const { name, entityID, twinID } = farm
  // const { pricingPolicyID, certificationType, countryID, cityID } = farm

  if (mnemonic === '') {
    mnemonic = testAccount
  }

  const client = await getClient('', mnemonic)

  certificationType = client.api.createType('CertificationType', certificationType)
  const farm = {
    id: 0,
    name,
    entityID,
    twinID,
    pricingPolicyID,
    certificationType,
    countryID,
    cityID
  }

  const create = await client.createFarm(farm, callback)
  return create
}

async function getFarm (id) {
  const client = await getClient('', testAccount)

  const farm = await client.getFarmByID(id)
  return farm
}

async function deleteFarm (id, mnemonic, callback) {
  if (mnemonic === '') {
    mnemonic = testAccount
  }

  const client = await getClient('', mnemonic)

  const farm = await client.deleteFarmByID(id, callback)
  return farm
}

async function createNode (farmID, twinID, countryID, cityID, mnemonic, callback) {
  if (mnemonic === '') {
    mnemonic = testAccount
  }

  const client = await getClient('', mnemonic)

  const resources = client.api.createType('Resources', {
    hru: 2000,
    sru: 5000,
    cru: 16,
    mru: 64
  })

  const location = client.api.createType('Location', {
    longitude: '4.349970',
    latitude: '50.845080'
  })

  const node = {
    id: 0,
    farm_id: farmID,
    twin_id: twinID,
    resources,
    location,
    country_id: 0,
    city_id: 0
  }

  const create = await client.createNode(node, callback)
  return create
}

async function getNode (id) {
  const client = await getClient('', testAccount)

  const node = await client.getNodeByID(id)
  return node
}

async function deleteNode (id, mnemonic, callback) {
  if (mnemonic === '') {
    mnemonic = testAccount
  }

  const client = await getClient('', mnemonic)

  const node = await client.deleteNodeByID(id, callback)
  return node
}

module.exports = {
  createEntity,
  updateEntity,
  getEntity,
  deleteEntity,
  createTwin,
  getTwin,
  deleteTwin,
  addTwinEntity,
  removeTwinEntity,
  createFarm,
  getFarm,
  deleteFarm,
  createNode,
  getNode,
  deleteNode
}
