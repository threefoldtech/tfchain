const stellarbase = require('stellar-base')
const { getClient } = require('./src/subclient')
const { exit } = require('yargs')

const testAccount = 'industry dismiss casual gym gap music pave gasp sick owner dumb cost'

async function main () {
  const client = await getClient('', testAccount)

  // FROM STELLAR
  const addr = client.keyring.encodeAddress(stellarbase.StrKey.decodeEd25519PublicKey('GC6HHHS7SH7KNUAOBKVGT2QZIQLRB5UA7QAGLA3IROWPH4TN65UKNJPK'))
  console.log(addr)

  // TO STELLAR
  // const y = stellarbase.StrKey.encodeEd25519PublicKey(client.keyring.decodeAddress(client.address))
  // console.log(y)
  exit(1)
}

main()
