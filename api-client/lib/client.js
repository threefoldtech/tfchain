const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api')
const types = require('../types.json')
const { getEntity, deleteEntity, createEntity, updateEntity } = require('./entity')
const bip39 = require('bip39')

class Client {
  constructor (url, words) {
    this.url = url
    this.words = words
    this.key = undefined
  }

  async init () {
    const api = await getPolkaAPI()

    if (!bip39.validateMnemonic(this.words)) {
      throw Error('Invalid mnemonic! Must be bip39 compliant')
    }

    const keyring = new Keyring({ type: 'ed25519' })
    const key = keyring.addFromMnemonic(this.words)

    this.key = key

    console.log(key.address)

    this.api = api
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

  async deleteEntity () {
    return deleteEntity(this)
  }
}

async function getPolkaAPI () {
  if (!this.url || this.url === '') {
    this.url = 'ws://localhost:9944'
  }

  const provider = new WsProvider(this.url)
  return ApiPromise.create({ provider, types })
}

module.exports = { Client }
