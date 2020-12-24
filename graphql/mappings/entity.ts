import { SubstrateEvent, DB } from '../generated/hydra-processor'
import { Entity } from '../generated/graphql-server/src/modules/entity/entity.model'
import BN from 'bn.js'

export async function templateModule_EntityStored(db: DB, event: SubstrateEvent) {
  const [entity_id, name, country_id, city_id] = event.params
  const entity = new Entity()
  entity.entityId = new BN(entity_id.value as number)
  entity.name = Buffer.from(name.value as string).toString()
  entity.countryId = new BN(country_id.value as number)
  entity.cityId = new BN(city_id.value as number)

  await db.save<Entity>(entity)
}

function convertBN(s: string): BN {
  if (String(s).startsWith('0x')) {
    return new BN(s.substring(2), 16)
  }
  return new BN(s)
}
