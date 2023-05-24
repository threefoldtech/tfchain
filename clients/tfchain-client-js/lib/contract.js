const { validateID } = require('./util')

// createContract creates a contract
async function createNodeContract (self, nodeID, data, hash, numberOfPublicIPs, solutionProviderID, callback) {
  const create = self.api.tx.smartContractModule.createNodeContract(nodeID, hash, data, numberOfPublicIPs, solutionProviderID)
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return create.signAndSend(self.key, { nonce }, callback)
}

async function updateNodeContract (self, contractID, data, hash, callback) {
  const update = self.api.tx.smartContractModule.updateNodeContract(contractID, hash, data)
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return update.signAndSend(self.key, { nonce }, callback)
}

async function createNameContract (self, name, callback) {
  const create = self.api.tx.smartContractModule.createNameContract(name)
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return create.signAndSend(self.key, { nonce }, callback)
}

async function createRentContract (self, nodeId, solutionProviderID, callback) {
  const create = self.api.tx.smartContractModule.createRentContract(nodeId, solutionProviderID)
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return create.signAndSend(self.key, { nonce }, callback)
}

// createContract creates a contract
async function cancelContract (self, contractID, callback) {
  const cancel = self.api.tx.smartContractModule.cancelContract(contractID)
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return cancel.signAndSend(self.key, { nonce }, callback)
}

// getContract gets an contract by id
async function getContract (self, id) {
  validateID(id)

  const contract = await self.api.query.smartContractModule.contracts(id)

  const res = contract.toJSON()

  return res
}

async function activeRentContractForNode (self, id) {
  validateID(id)

  const contract = await self.api.query.smartContractModule.activeRentContractForNode(id)

  const res = contract.toJSON()

  return res
}

async function contractIDByNameRegistration (self, name) {
  const contractID = await self.api.query.smartContractModule.contractIDByNameRegistration(name)

  const c = contractID.toJSON()

  return c
}

async function contractIDByNodeIDAndHash (self, nodeID, hash) {
  validateID(nodeID)

  const contractID = await self.api.query.smartContractModule.contractIDByNodeIDAndHash(nodeID, hash)

  const c = contractID.toJSON()

  return c
}

async function nodeContracts (self, id, contractState) {
  validateID(id)
  if (!['Created', 'Deleted', 'OutOfFunds'].includes(contractState)) {
    throw Error('You must pass a valid contract status')
  }

  const contracts = await self.api.query.smartContractModule.nodeContracts(id, contractState)
  const res = contracts.toJSON()

  return res
}

module.exports = {
  createNodeContract,
  updateNodeContract,
  createNameContract,
  createRentContract,
  cancelContract,
  getContract,
  contractIDByNameRegistration,
  contractIDByNodeIDAndHash,
  nodeContracts,
  activeRentContractForNode
}
