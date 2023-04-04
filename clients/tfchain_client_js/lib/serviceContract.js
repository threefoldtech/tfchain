const { validateID } = require('./util')

async function createServiceContract (self, serviceAccount, consumerAccount, callback) {
  const call = self.api.tx.smartContractModule.serviceContractCreate(serviceAccount, consumerAccount)
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return call.signAndSend(self.key, { nonce }, callback)
}

async function setServiceContractMetadata (self, serviceContractId, metadata, callback) {
  const call = self.api.tx.smartContractModule.serviceContractSetMetadata(serviceContractId, metadata)
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return call.signAndSend(self.key, { nonce }, callback)
}

async function setServiceContractFees (self, serviceContractId, baseFee, variableFee, callback) {
  const call = self.api.tx.smartContractModule.serviceContractSetFees(serviceContractId, baseFee, variableFee)
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return call.signAndSend(self.key, { nonce }, callback)
}

async function serviceContractApprove (self, serviceContractId, approve, callback) {
  let call
  if (approve) {
    call = self.api.tx.smartContractModule.serviceContractApprove(serviceContractId)
  } else {
    call = self.api.tx.smartContractModule.serviceContractReject(serviceContractId)
  }
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return call.signAndSend(self.key, { nonce }, callback)
}

async function serviceContractCancel (self, serviceContractId, callback) {
  const call = self.api.tx.smartContractModule.serviceContractCancel(serviceContractId)
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return call.signAndSend(self.key, { nonce }, callback)
}

async function serviceContractBill (self, serviceContractId, variableAmount, metadata, callback) {
  const call = self.api.tx.smartContractModule.serviceContractBill(serviceContractId, variableAmount, metadata)
  const nonce = await self.api.rpc.system.accountNextIndex(self.address)

  return call.signAndSend(self.key, { nonce }, callback)
}

// getServiceContract gets an contract by id
async function getServiceContract (self, serviceContractId) {
  validateID(serviceContractId)

  const contract = await self.api.query.smartContractModule.serviceContracts(serviceContractId)

  const res = contract.toJSON()

  return res
}

module.exports = {
  createServiceContract,
  setServiceContractMetadata,
  setServiceContractFees,
  serviceContractApprove,
  serviceContractCancel,
  serviceContractBill,
  getServiceContract
}
