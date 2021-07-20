const { getClient } = require('./client')
const { callback } = require('./util')

async function createNode (args) {
  const { a: url, m: mnemonic, farmID, c: countryID, g: cityID } = args
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

  const publicConfig = client.api.createType('PublicConfig', {
    ipv4: '1.1.1.1',
    ipv6: 'fe80::b4cf:8aff:fecc:8f64/64',
    gw4: '1.1.1.1',
    gw6: 'fe80::b4cf:8aff:fecc:8f64/64'
  })

  return await client.createNode(farmID, resources, location, countryID, cityID, publicConfig, callback)
}

async function updateNode (args) {
  const { a: url, m: mnemonic, farmID, c: countryID, g: cityID, id } = args
  const client = await getClient(url, mnemonic)

  const resources = client.api.createType('Resources', {
    hru: 2000,
    sru: 5000,
    cru: 16,
    mru: 64
  })

  const location = client.api.createType('Location', {
    longitude: '5.349970',
    latitude: '51.845080'
  })

  const publicConfig = client.api.createType('PublicConfig', {
    ipv4: '1.1.1.2',
    ipv6: 'fe80::b4cf:8aff:fecc:8f64/64',
    gw4: '1.1.1.2',
    gw6: 'fe80::b4cf:8aff:fecc:8f64/64'
  })

  return await client.updateNode(id, farmID, resources, location, countryID, cityID, publicConfig, callback)
}

async function getNode (args) {
  const { id, a: url } = args
  const client = await getClient(url, '')

  const node = await client.getNodeByID(id)

  console.log(node)
  process.exit(0)
}

async function getNodeByPubkey (args) {
  const { pubkey, a: url } = args
  const client = await getClient(url, '')

  const node = await client.getNodeByPubkey(pubkey)

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

  return await client.deleteNode(id, callback)
}

module.exports = {
  createNode,
  updateNode,
  getNode,
  getNodeByPubkey,
  listNodes,
  deleteNode
}
