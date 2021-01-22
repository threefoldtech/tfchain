const testAccount = 'industry dismiss casual gym gap music pave gasp sick owner dumb cost'
const { exit } = require('yargs')
const { getClient } = require('./src/client')

async function test () {
  const client = await getClient('', testAccount)

  const certificationType = await client.api.createType('CertificationType', 'silver')
  const farm = {
    id: 0,
    name: 'dssdadsds',
    entityID: 0,
    twinID: 0,
    pricingPolicyID: 0,
    certificationType,
    countryID: 0,
    cityID: 0
  }

  // createFarm(farm.name, farm.entityID, farm.twinID, farm.pricingPolicyID, farm.certificationType, farm.countryID, farm.cityID, testAccount, callback)

  // const res = await client.createFarm(farm)
  // console.log(`Included in block with has: ${res.toHex()}`)
  // exit(1)

  await client.createFarm(farm, (res) => {
    if (res instanceof Error) {
      console.log(res)
      exit(1)
    }

    const { events = [], status } = res
    console.log(`Current status is ${status.type}`)

    if (status.isFinalized) {
      console.log(`Transaction included at blockHash ${status.asFinalized}`)

      // Loop through Vec<EventRecord> to display all events
      events.forEach(({ phase, event: { data, method, section } }) => {
        console.log(`\t' ${phase}: ${section}.${method}:: ${data}`)
      })
      exit(1)
    }
  })
}

test()
