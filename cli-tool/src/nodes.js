const { getClient } = require('./client')
const { callback } = require('./util')

async function createNode (args) {
  const { a: url, m: mnemonic, farmID, twinID } = args
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

  return await client.createNode(node, callback)
}

async function getNode (args) {
  const { id, a: url } = args
  const client = await getClient(url, '')

  const node = await client.getNodeByID(id)

  console.log(node)
  process.exit(0)
}

async function listNodes (args) {
  const { a: url } = args
  const client = await getClient(url, '')

  const nodes = await client.listNodes()

  console.log(nodes)
  process.exit(0)
}

async function deleteNode (args) {
  const { a: url, m: mnemonic, id } = args
  const client = await getClient(url, mnemonic)

  await client.deleteNode(id, callback)

  console.log('node deleted')
  process.exit(0)
}

module.exports = {
  createNode,
  getNode,
  listNodes,
  deleteNode
}
