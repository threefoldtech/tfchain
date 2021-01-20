const { getFarm } = require('./farms')
const { getTwin } = require('./twin')

// createNode creates a node with given properties
async function createNode (self, node, callback) {
  try {
    await validateNode(self, node)
  } catch (error) {
    // callback early with error
    if (callback) {
      return callback(error)
    }
    return error
  }

  node = self.api.createType('Node', node)

  const create = self.api.tx.templateModule.createNode(node)
  return create.signAndSend(self.key, callback)
}

async function getNode (self, id) {
  if (isNaN(id)) {
    throw Error('You must pass an ID')
  }

  const node = await self.api.query.templateModule.nodes(id)

  return node.toJSON()
}

// deleteNode deletes a node by id
async function deleteNode (self, id, callback) {
  if (isNaN(id)) {
    throw Error('You must pass an ID')
  }

  const node = await getNode(self, id)
  if (node.id !== id) {
    throw Error(`node with id ${id} does not exist`)
  }

  return self.api.tx.templateModule
    .deleteNode(id)
    .signAndSend(self.key, callback)
}

async function validateNode (self, node) {
  const { farm_id: farmID, twin_id: twinID } = node

  const farm = await getFarm(self, farmID)
  if (farm.id !== farmID) {
    throw Error(`farm with id ${farmID} does not exist`)
  }

  const twin = await getTwin(self, twinID)
  if (twin.id !== twinID) {
    throw Error(`twin with id ${twinID} does not exist`)
  }
}

module.exports = {
  createNode,
  getNode,
  deleteNode
}
