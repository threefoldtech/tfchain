import { SubstrateEvent, DB } from '../generated/hydra-processor'
import { Entity } from '../generated/graphql-server/src/modules/entity/entity.model'
import BN from 'bn.js'

export async function templateModule_EntityStored(db: DB, event: SubstrateEvent) {
  const [entity_id, name, country_id, city_id, pub_key] = event.params
  const entity = new Entity()
  entity.entityId = new BN(entity_id.value as number)
  entity.name = hex2a(Buffer.from(name.value as string).toString())
  entity.countryId = new BN(country_id.value as number)
  entity.cityId = new BN(city_id.value as number)
  entity.pubKey = Buffer.from(pub_key.value as string).toString()

  await db.save<Entity>(entity)
}

export async function templateModule_EntityUpdated(db: DB, event: SubstrateEvent) {
  const [entity_id, name, country_id, city_id, pub_key] = event.params

  const savedEntity = await db.get(Entity, { where: { entityId: new BN(entity_id.value as number) } })

  if (savedEntity) {
    savedEntity.entityId = new BN(entity_id.value as number)
    savedEntity.name = hex2a(Buffer.from(name.value as string).toString())
    savedEntity.countryId = new BN(country_id.value as number)
    savedEntity.cityId = new BN(city_id.value as number)
    savedEntity.pubKey = Buffer.from(pub_key.value as string).toString()
  
    await db.save<Entity>(savedEntity)
  }
}

export async function templateModule_EntityDeleted(db: DB, event: SubstrateEvent) {
  const [entity_id] = event.params

  const savedEntity = await db.get(Entity, { where: { entityId: new BN(entity_id.value as number) } })

  if (savedEntity) {
    await db.remove(savedEntity)
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
