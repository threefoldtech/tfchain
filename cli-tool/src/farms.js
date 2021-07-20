const { getClient } = require('./client')
const { callback } = require('./util')

async function createFarm (args) {
  const { name, c: countryID, g: cityID, certificationType, pricingPolicyID, a: url, m: mnemonic } = args

  const client = await getClient(url, mnemonic)

  const certificationTypeParsed = client.api.createType('CertificationType', certificationType)

  const publicIP = client.api.createType('PublicIP', {
    ip: '1.1.1.1',
    gateway: '1.1.1.1',
    contract_id: 0
  })

  return client.createFarm(name, pricingPolicyID, certificationTypeParsed, countryID, cityID, [publicIP], callback)
}

async function getFarm (args) {
  const { id, a: url } = args
  const client = await getClient(url, '')

  const farm = await client.getFarmByID(id)

  console.log(farm)
  process.exit(0)
}

async function listFarms (args) {
  const { a: url } = args
  const client = await getClient(url, '')

  const farms = await client.listFarms()

  console.log(farms)
  process.exit(0)
}

async function deleteFarm (args) {
  const { id, a: url, m: mnemonic } = args
  const client = await getClient(url, mnemonic)

  await client.deleteFarmByID(id, callback)

  console.log('farm deleted')
  process.exit(0)
}

module.exports = {
  createFarm,
  getFarm,
  listFarms,
  deleteFarm
}
