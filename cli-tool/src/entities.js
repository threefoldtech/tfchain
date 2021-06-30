const { getClient } = require('./client')
const { callback } = require('./util')

async function createEntity (args) {
  const { a: url, m: mnemonic } = args
  const client = await getClient(url, mnemonic)

  let { f: target, s: sig, name, c: countryID, t: cityID } = args

  if (!target && !sig) {
    sig = await client.signEntityCreation(name, countryID, cityID)
    target = client.address
  }

  return client.createEntity(target, name, countryID, cityID, sig, callback)
}

async function updateEntity (args) {
  const { a: url, m: mnemonic } = args
  const client = await getClient(url, mnemonic)

  const { name, c: countryID, t: cityID } = args

  await client.updateEntity(name, countryID, cityID, callback)

  console.log('entity updated')
  process.exit(0)
}

async function getEntity (args) {
  const { a: url, id } = args

  const client = await getClient(url, '')
  const entity = await client.getEntityByID(id)

  console.log(entity)
  process.exit(0)
}

async function listEntities (args) {
  const { a: url } = args

  const client = await getClient(url, '')
  const entities = await client.listEntities()

  console.log(entities)
  process.exit(0)
}

async function deleteEntity (args) {
  const { a: url, m: mnemonic } = args

  const client = await getClient(url, mnemonic)

  try {
    await client.deleteEntity()
  } catch (error) {
    console.log(error)
    process.exit(1)
  }

  console.log('entity deleted')
  process.exit(0)
}

module.exports = {
  createEntity,
  updateEntity,
  getEntity,
  listEntities,
  deleteEntity
}
