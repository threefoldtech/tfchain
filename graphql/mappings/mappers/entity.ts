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
  const [version, entityID, name, countryID, cityID, accountID] = new TfgridModule.EntityStoredEvent(event).params

  entity.gridVersion = version.toNumber()
  entity.entityId = entityID.toNumber()
  entity.name = hex2a(Buffer.from(name.toString()).toString())
  entity.countryId = countryID.toNumber()
  entity.cityId = cityID.toNumber()
  entity.address = accountID.toHuman()

  await store.save<Entity>(entity)
}

export async function entityUpdated({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const entity = new Entity()
  const [entityID, name, countryID, cityID, accountID] = new TfgridModule.EntityUpdatedEvent(event).params

  const savedEntity = await store.get(Entity, { where: { entityId: entityID.toNumber() } })

  if (savedEntity) {
    // entity.gridVersion = version.toNumber()
    savedEntity.entityId = entityID.toNumber()
    savedEntity.name = hex2a(Buffer.from(name.toString()).toString())
    savedEntity.countryId = countryID.toNumber()
    savedEntity.cityId = cityID.toNumber()
    savedEntity.address = accountID.toHuman()
  
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