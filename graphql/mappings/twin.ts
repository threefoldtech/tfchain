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

// export async function templateModule_EntityUpdated(db: DB, event: SubstrateEvent) {
//   const [entity_id, name, country_id, city_id, pub_key] = event.params

//   const savedEntity = await db.get(Entity, { where: { entityId: new BN(entity_id.value as number) } })

//   if (savedEntity) {
//     savedEntity.entityId = new BN(entity_id.value as number)
//     savedEntity.name = hex2a(Buffer.from(name.value as string).toString())
//     savedEntity.countryId = new BN(country_id.value as number)
//     savedEntity.cityId = new BN(city_id.value as number)
//     savedEntity.pubKey = Buffer.from(pub_key.value as string).toString()
  
//     await db.save<Entity>(savedEntity)
//   }
// }

function hex2a (hex: string): string {
  var str = ''
  for (var i = 0; i < hex.length; i += 2) {
    var v = parseInt(hex.substr(i, 2), 16)
    if (v) str += String.fromCharCode(v)
  }
  return str
}
