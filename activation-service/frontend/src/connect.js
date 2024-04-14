import env from '@testing-library/jest-dom'
const { ApiPromise, WsProvider } = require('@polkadot/api')

const API_URL = env.API_URL

export async function connect () {
  // Initialise the provider to connect to the local node
  const provider = new WsProvider(API_URL)

  // Create the API and wait until ready
  const api = await ApiPromise.create({ provider })

  // Retrieve the chain & node information information via rpc calls
  const [chain, nodeName, nodeVersion] = await Promise.all([
    api.rpc.system.chain(),
    api.rpc.system.name(),
    api.rpc.system.version()
  ])

  console.log(`You are connected to chain ${chain} using ${nodeName} v${nodeVersion}`)
  return api
}
