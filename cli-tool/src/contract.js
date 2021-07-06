const { getClient } = require('./client')
const { callback } = require('./util')

async function createContract (args) {
  const { a: url, m: mnemonic } = args
  const client = await getClient(url, mnemonic)

  const { t: twinID, n: nodeID, w: workload, p: publicIPs } = args

  return client.createContract(twinID, nodeID, workload, publicIPs, callback)
}

async function cancelContract (args) {
  const { a: url, m: mnemonic, id } = args
  const client = await getClient(url, mnemonic)

  return client.cancelContract(id, callback)
}

async function getContract (args) {
  const { a: url, m: mnemonic, id } = args
  const client = await getClient(url, mnemonic)

  const contract = await client.getContractByID(id, callback)
  console.log(contract)
  process.exit(0)
}

module.exports = {
  createContract,
  cancelContract,
  getContract
}
