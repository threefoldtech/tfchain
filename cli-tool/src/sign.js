const { getClient } = require('./client')

async function signTwinEntityRelation (args) {
  const { entityID, twinID, a: url, m: mnemonic } = args
  const client = await getClient(url, mnemonic)

  const sig = await client.sign(entityID, twinID)

  console.log(sig)
  process.exit(0)
}

async function signEntityCreation (args) {
  const { name, countryID, cityID, a: url, m: mnemonic } = args
  const client = await getClient(url, mnemonic)

  const sig = await client.signEntityCreation(name, countryID, cityID)

  console.log(sig)
  process.exit(0)
}

module.exports = {
  signTwinEntityRelation,
  signEntityCreation
}
