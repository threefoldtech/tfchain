const { getApiClient } = require('./api')
const { Keyring } = require('@polkadot/api')

async function createEntity (name, countryID, cityID, callback) {
  const api = await getApiClient()
  const keyring = new Keyring({ type: 'sr25519' })
  const BOB = keyring.addFromUri('//Bob', { name: 'Bob default' })

  return api.tx.templateModule
    .createEntity(name, countryID, cityID)
    .signAndSend(BOB, callback)
}

async function getEntity (id) {
  const api = await getApiClient()
  const entity = await api.query.templateModule.entities(id)

  const res = entity.toJSON()
  res.name = hexToAscii(res.name)
  return res
}

function hexToAscii (str1) {
  const hex = str1.toString()
  let str = ''
  for (let n = 0; n < hex.length; n += 2) {
    str += String.fromCharCode(parseInt(hex.substr(n, 2), 16))
  }
  return str
}

module.exports = {
  createEntity,
  getEntity
}
