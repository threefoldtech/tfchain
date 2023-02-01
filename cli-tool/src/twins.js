const { getClient } = require('./client')
const { callback } = require('./util')

async function createTwin(args) {
  const { ip, a: url, m: mnemonic } = args
  const client = await getClient(url, mnemonic)

  return client.createTwin(ip, callback)
}

async function getTwin(args) {
  const { a: url, id } = args

  const client = await getClient(url, '')
  const twin = await client.getTwinByID(id)

  console.log(twin)
  process.exit(0)
}

async function listTwins(args) {
  const { a: url } = args
  const client = await getClient(url, '')

  const twins = await client.listTwins()

  console.log(twins)
  process.exit(0)
}

async function createTwinEntity(twinID, entityID, signature, mnemonic, url, callback) {
  const client = await getClient(url, mnemonic)

  const create = await client.addTwinEntity(twinID, entityID, signature, callback)
  return create
}

async function deleteTwinEntity(twinID, entityID, mnemonic, url, callback) {
  const client = await getClient(url, mnemonic)

  const create = await client.removeTwinEntity(twinID, entityID, callback)
  return create
}

module.exports = {
  createTwin,
  getTwin,
  listTwins,
  createTwinEntity,
  deleteTwinEntity
}
