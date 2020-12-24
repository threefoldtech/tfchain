import { SubstrateEvent, DB } from '../generated/hydra-processor'
import { Entity } from '../generated/graphql-server/src/modules/entity/entity.model'
import BN from 'bn.js'

export async function templateModule_EntityStored(db: DB, event: SubstrateEvent) {
  const [id, name, country_id, city_id] = event.params
  const entity = new Entity()
  entity.id = Buffer.from(id.value as string).toString()
  entity.name = Buffer.from(name.value as string).toString()
  entity.countryId = convertBN(country_id.value as string)
  entity.cityId = convertBN(city_id.value as string)

  await db.save<Entity>(entity)
}

function convertBN(s: string): BN {
  if (String(s).startsWith('0x')) {
    return new BN(s.substring(2), 16)
  }
  return new BN(s)
}
