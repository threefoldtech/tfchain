const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api')
const crypto = require('@polkadot/util-crypto')
const bip39 = require('bip39')
const types = require('../types.json')
const {
  getEntity, deleteEntity,
  createEntity, updateEntity,
  listEntities, getEntityIDByName,
  getEntityIDByPubkey
} = require('./entity')
const {
  createTwin, getTwin, getTwinIdByAccountId, updateTwin,
  deleteTwin, addTwinEntity, deleteTwinEntity, listTwins
} = require('./twin')
const {
  createFarm, getFarm, deleteFarm,
  listFarms, addFarmIP, deleteFarmIP, setNodePower
} = require('./farms')
const {
  createNode, updateNode, getNode,
  getNodeIDByPubkey, deleteNode, listNodes
} = require('./node')
const { signEntityTwinID, signEntityCreation } = require('./sign')
const { getBalance, transfer } = require('./balance')
const { proposeTransaction, voteTransaction, listValidators } = require('./voting')
const {
  createNameContract, createNodeContract,
  updateNodeContract, cancelContract,
  getContract, contractIDByNameRegistration,
  contractIDByNodeIDAndHash, nodeContracts,
  createRentContract, activeRentContractForNode
} = require('./contract')
const { tfStoreSet, tfStoreGet, tfStoreList, tfStoreRemove } = require('./tfkvstore')
const { getBlockHash, getBlockTime } = require('./chain')
const { acceptTermsAndConditions } = require('./terms_and_conditions')
const { tftPrice } = require('./tft_price')
const { getPricingPolicyById } = require('./pricing_policy')
const { batch, batchAll } = require('./utility')
const {
  createServiceContract, getServiceContract,
  serviceContractApprove, serviceContractBill,
  serviceContractCancel, setServiceContractFees, setServiceContractMetadata
} = require('./serviceContract')

const validSchemes = ['ed25519', 'sr25519']

class Client {
  constructor(url, words, scheme) {
    this.url = url
    this.words = words

    if (!validSchemes.includes(scheme)) {
      throw Error(`scheme" ${scheme} is not a valid scheme. Should be either of: ${validSchemes}`)
    }

    this.scheme = scheme || 'sr25519'
    this.key = undefined
    this.address = undefined
  }

  async init() {
    const api = await getPolkaAPI(this.url)
    const keyring = new Keyring({ type: this.scheme })

    if (!this.words) {
      this.words = crypto.mnemonicGenerate()
    }

    let key
    try {
      key = keyring.addFromUri(this.words)
    } catch (error) {
      try {
        if (!bip39.validateMnemonic(this.words)) {
          throw Error('Invalid mnemonic! Must be bip39 compliant')
        }
        key = keyring.addFromMnemonic(this.words)
      } catch (error) {
        throw Error('Invalid mnemonic or secret seed! Check your input.')
      }
    }

    this.key = key
    console.log(`Key with address: ${this.key.address} is loaded.`)

    this.keyring = keyring
    this.address = this.key.address

    this.api = api
  }

  async createMnemonic() {
    return crypto.mnemonicGenerate()
  }

  async sign(entityID, twinID) {
    return signEntityTwinID(this, entityID, twinID)
  }

  async signEntityCreation(name, countryID, cityID) {
    return signEntityCreation(this, name, countryID, cityID)
  }

  async updateEntity(name, countryID, cityID, callback) {
    return updateEntity(this, name, countryID, cityID, callback)
  }

  async createEntity(target, name, countryID, cityID, signature, callback) {
    return createEntity(this, target, name, countryID, cityID, signature, callback)
  }

  async getEntityByID(id) {
    return getEntity(this, id)
  }

  async getEntityIDByName(name) {
    return getEntityIDByName(this, name)
  }

  async getEntityIDByPubkey(pubkey) {
    return getEntityIDByPubkey(this, pubkey)
  }

  async listEntities() {
    return listEntities(this)
  }

  async deleteEntity(callback) {
    return deleteEntity(this, callback)
  }

  async createTwin (relay, pk, callback) {
    return createTwin(this, relay, pk, callback)
  }

  async updateTwin (relay, pk, callback) {
    return updateTwin(this, relay, pk, callback)
  }

  async getTwinByID (id) {
    return getTwin(this, id)
  }

  async getTwinIdByAccountId(accountId) {
    return getTwinIdByAccountId(this, accountId)
  }

  async listTwins() {
    return listTwins(this)
  }

  async deleteTwin(id, callback) {
    return deleteTwin(this, id, callback)
  }

  async addTwinEntity(twinID, entityID, signature, callback) {
    return addTwinEntity(this, twinID, entityID, signature, callback)
  }

  async deleteTwinEntity(twinID, entityID, callback) {
    return deleteTwinEntity(this, twinID, entityID, callback)
  }

  async createFarm(name, certificationType, publicIPs, callback) {
    return createFarm(this, name, certificationType, publicIPs, callback)
  }

  async addFarmIp(id, ip, gateway, callback) {
    return addFarmIP(this, id, ip, gateway, callback)
  }

  async deleteFarmIp(id, ip, callback) {
    return deleteFarmIP(this, id, ip, callback)
  }

  async getFarmByID(id) {
    return getFarm(this, id)
  }

  async listFarms() {
    return listFarms(this)
  }

  async deleteFarmByID(id, callback) {
    return deleteFarm(this, id, callback)
  }

  async createNode(farmID, resources, location, countryID, cityID, publicConfig, callback) {
    return createNode(this, farmID, resources, location, countryID, cityID, publicConfig, callback)
  }

  async updateNode(nodeID, farmID, resources, location, countryID, cityID, publicConfig, callback) {
    return updateNode(this, nodeID, farmID, resources, location, countryID, cityID, publicConfig, callback)
  }

  async getNodeByID(id) {
    return getNode(this, id)
  }

  async getNodeByPubkey(id) {
    return getNodeIDByPubkey(this, id)
  }

  async listNodes() {
    return listNodes(this)
  }

  async deleteNode(id, callback) {
    return deleteNode(this, id, callback)
  }

  async setNodePower (nodeId, power, callback) {
    return setNodePower(this, nodeId, power, callback)
  }

  async getBalance () {
    return getBalance(this, this.address)
  }

  async getBalanceOf(address) {
    return getBalance(this, address)
  }

  async transfer(address, amount, callback) {
    return transfer(this, address, amount, callback)
  }

  async proposeTransaction(transactionID, to, amount, callback) {
    return proposeTransaction(this, transactionID, to, amount, callback)
  }

  async voteTransaction(transactionID, callback) {
    return voteTransaction(this, transactionID, callback)
  }

  async listValidators() {
    return listValidators(this)
  }

  async verify(message, signature, pubkey) {
    return this.key.verify(message, signature, pubkey)
  }

  async createNodeContract(nodeID, data, deploymentHash, publicIPS, solutionProviderID, callback) {
    return createNodeContract(this, nodeID, data, deploymentHash, publicIPS, solutionProviderID, callback)
  }

  async updateNodeContract(contractID, data, hash, callback) {
    return updateNodeContract(this, contractID, data, hash, callback)
  }

  async createNameContract(name, callback) {
    return createNameContract(this, name, callback)
  }

  async createRentContract(nodeId, solutionProviderID, callback) {
    return createRentContract(this, nodeId, solutionProviderID, callback)
  }

  async activeRentContractForNode(nodeId, callback) {
    return activeRentContractForNode(this, nodeId, callback)
  }

  async cancelContract(contractID, callback) {
    return cancelContract(this, contractID, callback)
  }

  async getContractByID(id) {
    return getContract(this, id)
  }

  async contractIDByNameRegistration(name) {
    return contractIDByNameRegistration(this, name)
  }

  async contractIDByNodeIDAndHash(nodeID, hash) {
    return contractIDByNodeIDAndHash(this, nodeID, hash)
  }

  async nodeContracts(id, contractState) {
    return nodeContracts(this, id, contractState)
  }

  async tfStoreSet(key, value, callback) {
    return tfStoreSet(this, key, value, callback)
  }

  async tfStoreGet(key) {
    return tfStoreGet(this, key)
  }

  async tfStoreList() {
    return tfStoreList(this)
  }

  async tfStoreRemove(key, callback) {
    return tfStoreRemove(this, key, callback)
  }

  async getBlockHash(blockNumber) {
    return getBlockHash(this, blockNumber)
  }

  async getBlockTime(blockHash) {
    return getBlockTime(this, blockHash)
  }

  async acceptTermsAndConditions(documentLink, documentHash, callback) {
    return acceptTermsAndConditions(this, documentLink, documentHash, callback)
  }

  async tftPrice() {
    return tftPrice(this)
  }

  async getPricingPolicyById(policyId) {
    return getPricingPolicyById(this, policyId)
  }

  async batch(extrinsics, callback) {
    return batch(this, extrinsics, callback)
  }

  async batchAll(extrinsics, callback) {
    return batchAll(this, extrinsics, callback)
  }

  async createServiceContract(serviceAccount, consumerAccount, callback) {
    return createServiceContract(this, serviceAccount, consumerAccount, callback)
  }

  async setServiceContractMetadata(serviceContractId, metadata, callback) {
    return setServiceContractMetadata(this, serviceContractId, metadata, callback)
  }

  async setServiceContractFees(serviceContractId, baseFee, variableFee, callback) {
    return setServiceContractFees(this, serviceContractId, baseFee, variableFee, callback)
  }

  async serviceContractApprove(serviceContractId, approve, callback) {
    return serviceContractApprove(this, serviceContractId, approve, callback)
  }

  async serviceContractCancel(serviceContractId, callback) {
    return serviceContractCancel(this, serviceContractId, callback)
  }

  async serviceContractBill(serviceContractId, variableAmount, metadata, callback) {
    return serviceContractBill(this, serviceContractId, variableAmount, metadata, callback)
  }

  async getServiceContract(serviceContractId) {
    return getServiceContract(this, serviceContractId)
  }
}
async function getPolkaAPI(url) {
  if (!url || url === '') {
    url = 'ws://localhost:9944'
  }

  const provider = new WsProvider(url)
  return ApiPromise.create({ provider, types })
}

module.exports = { Client }
