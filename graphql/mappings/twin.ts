import { SubstrateEvent, DB } from '../generated/hydra-processor'
import { Twin } from '../generated/graphql-server/src/modules/twin/twin.model'
import BN from 'bn.js'

export async function templateModule_TwinStored(db: DB, event: SubstrateEvent) {
  const [twin_id, pubkey, entity_id] = event.params
  const twin = new Twin()
  twin.twinId = new BN(twin_id.value as number)
  twin.pubKey = hex2a(Buffer.from(pubkey.value as string).toString())
  twin.entityId = new BN(entity_id.value as number)

  await db.save<Twin>(twin)
}

export async function templateModule_TwinDeleted(db: DB, event: SubstrateEvent) {
  const [twin_id] = event.params

  const savedTwin = await db.get(Twin, { where: { twinId: new BN(twin_id.value as number) } })

  if (savedTwin) {
    await db.remove(savedTwin)
  }
}

function hex2a (hex: string): string {
  var str = ''
  for (var i = 0; i < hex.length; i += 2) {
    var v = parseInt(hex.substr(i, 2), 16)
    if (v) str += String.fromCharCode(v)
  }
  return str
}
