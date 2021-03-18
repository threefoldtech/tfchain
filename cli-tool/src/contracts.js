const { getClient } = require('./client')

async function createEntity (target, name, countryID, cityID, sig, mnemonic, url, callback) {
  const client = await getClient(url, mnemonic)

  if (!target && !sig) {
    sig = await client.signEntityCreation(name, countryID, cityID)
    target = client.address
  }

  const entity = await client.createEntity(target, name, countryID, cityID, sig, callback)
  return entity
}

async function updateEntity (name, countryID, cityID, mnemonic, url, callback) {
  const client = await getClient(url, mnemonic)

  const update = await client.updateEntity(name, countryID, cityID, callback)
  return update
}

async function getEntity (id, url) {
  const client = await getClient(url, '')

  const entity = await client.getEntityByID(id)
  return entity
}

async function listEntities (url) {
  const client = await getClient(url, '')

  const entities = await client.listEntities()
  return entities
}

async function deleteEntity (mnemonic, url) {
  const client = await getClient(url, mnemonic)

  return await client.deleteEntity()
}

async function createTwin (ip, mnemonic, url, callback) {
  const client = await getClient(url, mnemonic)

  const create = await client.createTwin(ip, callback)
  return create
}

async function getTwin (id, url) {
  const client = await getClient(url, '')

  const twin = await client.getTwinByID(id)
  return twin
}

async function listTwins (url) {
  const client = await getClient(url, '')

  return await client.listTwins()
}

async function deleteTwin (id, mnemonic, url, callback) {
  const client = await getClient(url, mnemonic)

  const twin = await client.deleteTwinByID(id, callback)
  return twin
}

async function addTwinEntity (twinID, entityID, signature, mnemonic, url, callback) {
  const client = await getClient(url, mnemonic)

  const create = await client.addTwinEntity(twinID, entityID, signature, callback)
  return create
}

async function removeTwinEntity (twinID, entityID, mnemonic, url, callback) {
  const client = await getClient(url, mnemonic)

  const create = await client.removeTwinEntity(twinID, entityID, callback)
  return create
}

async function createFarm (name, entityID, twinID, pricingPolicyID, certificationType, countryID, cityID, mnemonic, url, callback) {
  // const { name, entityID, twinID } = farm
  // const { pricingPolicyID, certificationType, countryID, cityID } = farm

  const client = await getClient(url, mnemonic)

  certificationType = client.api.createType('CertificationType', certificationType)
  const farm = {
    id: 0,
    name,
    entity_id: entityID,
    twin_id: twinID,
    pricingPolicyID,
    certificationType,
    countryID,
    cityID
  }

  const create = await client.createFarm(farm, callback)
  return create
}

async function getFarm (id, url) {
  const client = await getClient(url, '')

  const farm = await client.getFarmByID(id)
  return farm
}

async function listFarms (url) {
  const client = await getClient(url, '')

  return await client.listFarms()
}

async function deleteFarm (id, mnemonic, url, callback) {
  const client = await getClient(url, mnemonic)

  const farm = await client.deleteFarmByID(id, callback)
  return farm
}

async function createNode (farmID, twinID, countryID, cityID, mnemonic, url, callback) {
  const client = await getClient(url, mnemonic)

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

  const role = client.api.createType('Role', 'Node')

  const node = {
    id: 0,
    farm_id: farmID,
    twin_id: twinID,
    resources,
    location,
    country_id: 0,
    city_id: 0,
    role
  }

  const create = await client.createNode(node, callback)
  return create
}

async function getNode (id, url) {
  const client = await getClient(url, '')

  const node = await client.getNodeByID(id)
  return node
}

async function listNodes (url) {
  const client = await getClient(url, '')

  return await client.listNodes()
}

async function deleteNode (id, mnemonic, url, callback) {
  const client = await getClient(url, mnemonic)

  const node = await client.deleteNode(id, callback)
  return node
}

async function sign (entityID, twinID, mnemonic, url) {
  const client = await getClient(url, mnemonic)

  return client.sign(entityID, twinID)
}

async function signEntityCreation (name, countryID, cityID, url, mnemonic) {
  const client = await getClient(url, mnemonic)

  const sig = await client.signEntityCreation(name, countryID, cityID)
  return sig
}

async function getPrice (mnemonic, url) {
  const client = await getClient(url, mnemonic)

  return client.getPrice()
}

async function getAvgPrice (mnemonic, url) {
  const client = await getClient(url, mnemonic)

  return client.getAveragePrice()
}

async function vestedTransfer (locked, perBlock, startingBlock, tftPrice, mnemonic, url, callback) {
  const client = await getClient(url, mnemonic)

  const vest = await client.vest(locked, perBlock, startingBlock, tftPrice, callback)
  return vest
}

async function getBalance (mnemonic, url) {
  const client = await getClient(url, mnemonic)

  return await client.getBalance()
}

module.exports = {
  createEntity,
  updateEntity,
  getEntity,
  listEntities,
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
  deleteNode,
  sign,
  getPrice,
  getAvgPrice,
  listTwins,
  listFarms,
  listNodes,
  vestedTransfer,
  getBalance,
  signEntityCreation
}
