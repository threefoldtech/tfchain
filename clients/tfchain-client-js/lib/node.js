const { getFarm } = require('./farms')
const { hex2a } = require('./util')

// createNode creates a node with given properties
async function createNode (self, farmID, resources, location, countryID, cityID, publicConfig, callback) {
  try {
    await validateNode(self, farmID)
  } catch (error) {
    // callback early with error
    if (callback) {
      return callback(error)
    }
    return error
  }

  const parsedResources = self.api.createType('Resources', resources)
  const parsedLocation = self.api.createType('Location', location)
  const parsedPublicConfig = self.api.createType('PublicConfig', publicConfig)

  const create = self.api.tx.tfgridModule.createNode(farmID, parsedResources, parsedLocation, countryID, cityID, parsedPublicConfig)
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return create.signAndSend(self.key, { nonce }, callback)
}

// updateNode updates a node with given properties
async function updateNode (self, nodeID, farmID, resources, location, countryID, cityID, publicConfig, callback) {
  const storedNode = await getNode(self, nodeID)
  if (parseInt(storedNode.id) !== parseInt(nodeID)) {
    throw Error(`node with id ${nodeID} does not exist`)
  }

  try {
    await validateNode(self, farmID)
  } catch (error) {
    // callback early with error
    if (callback) {
      return callback(error)
    }
    return error
  }

  const parsedResources = self.api.createType('Resources', resources)
  const parsedLocation = self.api.createType('Location', location)
  const parsedPublicConfig = self.api.createType('PublicConfig', publicConfig)

  const create = self.api.tx.tfgridModule.updateNode(farmID, parsedResources, parsedLocation, countryID, cityID, parsedPublicConfig)
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return create.signAndSend(self.key, { nonce }, callback)
}

async function getNode (self, id) {
  try {
    id = parseInt(id)
  } catch (error) {
    throw Error('ID must be an integer')
  }
  if (isNaN(id) || id === 0) {
    throw Error('You must pass a valid ID')
  }

  const node = await self.api.query.tfgridModule.nodes(id)

  const res = node.toJSON()
  if (res.id !== id) {
    throw Error('No such node')
  }

  // Decode location
  const { location } = res
  const { longitude = '', latitude = '' } = location
  location.longitude = hex2a(longitude)
  location.latitude = hex2a(latitude)

  if (res.country) {
    res.country = hex2a(res.country)
  }

  if (res.city) {
    res.city = hex2a(res.city)
  }

  const { public_config: publicConfig } = res
  if (publicConfig) {
    const { ipv4, ipv6, gw4, gw6 } = publicConfig
    publicConfig.ipv4 = hex2a(ipv4)
    publicConfig.ipv6 = hex2a(ipv6)
    publicConfig.gw4 = hex2a(gw4)
    publicConfig.gw6 = hex2a(gw6)
  }

  if (res.serialNumber) {
    res.serialNumber = hex2a(res.serialNumber)
  }

  return res
}

async function getNodeIDByPubkey (self, pubkey) {
  const res = await self.api.query.tfgridModule.nodesByPubkeyID(pubkey)

  return res.toJSON()
}

async function listNodes (self) {
  const nodes = await self.api.query.tfgridModule.nodes.entries()

  const parsedNodes = nodes.map(node => {
    return node[1].toJSON()
  })

  return parsedNodes
}

// deleteNode deletes a node by id
async function deleteNode (self, id, callback) {
  const node = await getNode(self, id)
  if (parseInt(node.id) !== parseInt(id)) {
    throw Error(`node with id ${id} does not exist`)
  }
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return self.api.tx.tfgridModule
    .deleteNode(id)
    .signAndSend(self.key, { nonce }, callback)
}

// optOutOfV3Billing opts a node out of v3 billing (farmer only, one-way)
async function optOutOfV3Billing (self, nodeID, callback) {
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)
  return self.api.tx.tfgridModule
    .optOutOfV3Billing(nodeID)
    .signAndSend(self.key, { nonce }, callback)
}

// addTwinAdmin adds an account to the list of twin admins (council origin)
async function addTwinAdmin (self, account, callback) {
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)
  return self.api.tx.tfgridModule
    .addTwinAdmin(account)
    .signAndSend(self.key, { nonce }, callback)
}

// removeTwinAdmin removes an account from the list of twin admins (council origin)
async function removeTwinAdmin (self, account, callback) {
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)
  return self.api.tx.tfgridModule
    .removeTwinAdmin(account)
    .signAndSend(self.key, { nonce }, callback)
}

// getAllowedTwinAdmins returns the list of accounts allowed to deploy on opted-out nodes
async function getAllowedTwinAdmins (self) {
  const result = await self.api.query.tfgridModule.allowedTwinAdmins()
  return result.toJSON() || []
}

// isNodeOptedOutOfV3Billing returns true if the node has opted out of v3 billing
async function isNodeOptedOutOfV3Billing (self, nodeID) {
  const result = await self.api.query.tfgridModule.nodeV3BillingOptOut(nodeID)
  return !result.isNone
}

async function validateNode (self, farmID) {
  const farm = await getFarm(self, farmID)
  if (farm.id !== farmID) {
    throw Error(`farm with id ${farmID} does not exist`)
  }
}

module.exports = {
  createNode,
  updateNode,
  getNode,
  getNodeIDByPubkey,
  deleteNode,
  listNodes,
  optOutOfV3Billing,
  addTwinAdmin,
  removeTwinAdmin,
  getAllowedTwinAdmins,
  isNodeOptedOutOfV3Billing
}
