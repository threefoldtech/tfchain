const testAccount = 'industry dismiss casual gym gap music pave gasp sick owner dumb cost'
const { getClient } = require('./src/client')
const { exit } = require('yargs')

async function signExample () {
  const client = await getClient('', testAccount)

  const entityID = 0
  const twinID = 0
  const message = await client.sign(entityID, twinID)
  console.log(message)

  exit(1)
}

signExample()
