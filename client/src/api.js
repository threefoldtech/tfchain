const { ApiPromise, WsProvider } = require('@polkadot/api')
const types = require('../types.json')

async function getApiClient () {
  const wsProvider = new WsProvider('ws://localhost:9944')
  return ApiPromise.create({
    provider: wsProvider,
    types
  })
}

module.exports = { getApiClient }
