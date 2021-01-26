const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api')
const types = require('../types.json')
const bip39 = require('bip39')

const { getEntity, deleteEntity, createEntity, updateEntity } = require('./entity')
const { createTwin, getTwin, deleteTwin, addTwinEntity, deleteTwinEntity } = require('./twin')
const { createFarm, getFarm, deleteFarm } = require('./farms')
const { createNode, getNode, deleteNode } = require('./node')
const { signEntityTwinID } = require('./sign')

class Client {
  constructor (url, words) {
    this.url = url
    this.words = words
    this.key = undefined
    this.address = undefined
  }

  async init () {
    const api = await getPolkaAPI(this.url)

    // if no words provided, use the default alice key
    if (!this.words) {
      const keyring = new Keyring({ type: 'sr25519' })
      const key = keyring.addFromUri('//Alice', { name: 'Alice default' })
      this.key = key
      console.log(`Default account Alice with address: ${this.key.address} is loaded.`)
    } else {
      if (!bip39.validateMnemonic(this.words)) {
        throw Error('Invalid mnemonic! Must be bip39 compliant')
      }
      const keyring = new Keyring({ type: 'ed25519' })
      const key = keyring.addFromMnemonic(this.words)
      this.key = key
      console.log(`Key with address: ${this.key.address} is loaded.`)
    }

    this.address = this.key.address

    this.api = api
  }

  async sign (entityID, twinID) {
    return signEntityTwinID(this, entityID, twinID)
  }

  async updateEntity (name, countryID, cityID, callback) {
    return updateEntity(this, name, countryID, cityID, callback)
  }

  async createEntity (name, countryID, cityID, callback) {
    return createEntity(this, name, countryID, cityID, callback)
  }

  async getEntityByID (id) {
    return getEntity(this, id)
  }

  async deleteEntity (callback) {
    return deleteEntity(this, callback)
  }

  async createTwin (peerID, callback) {
    return createTwin(this, peerID, callback)
  }

  async getTwinByID (id) {
    return getTwin(this, id)
  }

  async deleteTwin (id, callback) {
    return deleteTwin(this, id, callback)
  }

  async addTwinEntity (twinID, entityID, signature, callback) {
    return addTwinEntity(this, twinID, entityID, signature, callback)
  }

  async deleteTwinEntity (twinID, entityID, callback) {
    return deleteTwinEntity(this, twinID, entityID, callback)
  }

  async createFarm (farm, callback) {
    return createFarm(this, farm, callback)
  }

  async getFarmByID (id) {
    return getFarm(this, id)
  }

  async deleteFarmByID (id, callback) {
    return deleteFarm(this, id, callback)
  }

  async createNode (node, callback) {
    return createNode(this, node, callback)
  }

  async getNodeByID (id) {
    return getNode(this, id)
  }

  async deleteNode (id, callback) {
    return deleteNode(this, id, callback)
  }
}

async function getPolkaAPI (url) {
  if (!url || url === '') {
    url = 'ws://localhost:9944'
  }

  const provider = new WsProvider(url)
  return ApiPromise.create({ provider, types })
}

module.exports = { Client }
