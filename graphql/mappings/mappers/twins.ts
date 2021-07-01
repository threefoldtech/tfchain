import { Twin, EntityProof } from '../../generated/graphql-server/model'
import { TfgridModule } from '../generated/types'
import { hex2a } from './util'

import {
  EventContext,
  StoreContext,
} from '@subsquid/hydra-common'

export async function twinStored({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const twin = new Twin()
  const [version, twin_id, account_id, ip] = new TfgridModule.TwinStoredEvent(event).params

  twin.gridVersion = version.toNumber()
  twin.twinId = twin_id.toNumber()
  twin.address = hex2a(account_id.toString())
  twin.ip = hex2a(ip.toString())

  await store.save<Twin>(twin)
}

export async function twinDeleted({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const [twin_id] = new TfgridModule.TwinDeletedEvent(event).params
  
  const savedTwin = await store.get(Twin, { where: { twinId: twin_id.toNumber() } })

  if (savedTwin) {
    await store.remove(savedTwin)
  }
}

export async function twinEntityStored({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const entityProof = new EntityProof()
  const [twin_id, entity_id, signature] = new TfgridModule.TwinEntityStoredEvent(event).params

  let savedTwin = await store.get(Twin, { where: { twinId: twin_id.toNumber() } })

  if (savedTwin) {
    const entityProof = new EntityProof()
    entityProof.entityId = entity_id.toNumber()
    entityProof.signature = Buffer.from(signature.toString()).toString()
    
    // and the twin foreign key to entityproof
    entityProof.twinRel = savedTwin

    await store.save<EntityProof>(entityProof)
  }
}

export async function twinEntityRemoved({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const [twin_id, entity_id] = new TfgridModule.TwinEntityRemovedEvent(event).params

  let savedTwinEntity = await store.get(EntityProof, { where: { entityId: entity_id.toNumber() }})
  if (savedTwinEntity) {
    await store.remove(savedTwinEntity)
  }
}
