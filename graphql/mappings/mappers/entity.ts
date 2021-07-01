import { Entity  } from '../../generated/graphql-server/model'
import { TfgridModule } from '../generated/types'
import { hex2a } from './util'

import {
  EventContext,
  StoreContext,
} from '@subsquid/hydra-common'

export async function entityStored({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const entity = new Entity()
  const [version, entity_id, name, country_id, city_id, account_id] = new TfgridModule.EntityStoredEvent(event).params

  entity.gridVersion = version.toNumber()
  entity.entityId = entity_id.toNumber()
  entity.name = hex2a(name.toString())
  entity.countryId = country_id.toNumber()
  entity.cityId = city_id.toNumber()
  entity.address = hex2a(account_id.toString())

  await store.save<Entity>(entity)
}

export async function entityUpdated({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const entity = new Entity()
  const [entity_id, name, country_id, city_id, account_id] = new TfgridModule.EntityUpdatedEvent(event).params

  const savedEntity = await store.get(Entity, { where: { entityId: entity_id.toNumber() } })

  if (savedEntity) {
    // entity.gridVersion = version.toNumber()
    savedEntity.entityId = entity_id.toNumber()
    savedEntity.name = hex2a(name.toString())
    savedEntity.countryId = country_id.toNumber()
    savedEntity.cityId = city_id.toNumber()
    savedEntity.address = hex2a(account_id.toString())
  
    await store.save<Entity>(savedEntity)
  }
}

export async function entityDeleted({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const entity = new Entity()
  const [entity_id] = new TfgridModule.EntityDeletedEvent(event).params

  const savedEntity = await store.get(Entity, { where: { entityId: entity_id.toNumber() } })

  if (savedEntity) {
    store.remove(savedEntity)
  }
}