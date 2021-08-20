import { Entity  } from '../generated/model'
import { TfgridModule } from '../types'
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
  const newEntity = new Entity()
  const [entity] = new TfgridModule.EntityStoredEvent(event).params

  newEntity.gridVersion = entity.version.toNumber()
  newEntity.entityId = entity.id.toNumber()
  newEntity.name = hex2a(Buffer.from(entity.name.toString()).toString())
  newEntity.country = hex2a(entity.country.toString())
  newEntity.city = hex2a(entity.city.toString())
  newEntity.accountId = entity.account_id.toHuman()

  await store.save<Entity>(newEntity)
}

export async function entityUpdated({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const newEntity = new Entity()
  const [entity] = new TfgridModule.EntityUpdatedEvent(event).params

  const savedEntity = await store.get(Entity, { where: { entityId: entity.id.toNumber() } })

  if (savedEntity) {
    newEntity.gridVersion = entity.version.toNumber()
    newEntity.entityId = entity.id.toNumber()
    newEntity.name = hex2a(Buffer.from(entity.name.toString()).toString())
    newEntity.country = hex2a(entity.country.toString())
    newEntity.city = hex2a(entity.city.toString())
    newEntity.accountId = entity.account_id.toHuman()
  
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
  const [entityID] = new TfgridModule.EntityDeletedEvent(event).params

  const savedEntity = await store.get(Entity, { where: { entityId: entityID.toNumber() } })

  if (savedEntity) {
    store.remove(savedEntity)
  }
}