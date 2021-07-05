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
  const [version, twinID, accountID, ip] = new TfgridModule.TwinStoredEvent(event).params

  twin.gridVersion = version.toNumber()
  twin.twinId = twinID.toNumber()
  twin.address = accountID.toHuman()
  twin.ip = hex2a(Buffer.from(ip.toString()).toString())

  await store.save<Twin>(twin)
}

export async function twinDeleted({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const [twinID] = new TfgridModule.TwinDeletedEvent(event).params
  
  const savedTwin = await store.get(Twin, { where: { twinId: twinID.toNumber() } })

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
  const [twinID, entityID, signature] = new TfgridModule.TwinEntityStoredEvent(event).params

  let savedTwin = await store.get(Twin, { where: { twinId: twinID.toNumber() } })

  if (savedTwin) {
    const entityProof = new EntityProof()
    entityProof.entityId = entityID.toNumber()
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
  const [twinID, entityID] = new TfgridModule.TwinEntityRemovedEvent(event).params

  let savedTwinEntity = await store.get(EntityProof, { where: { entityId: entityID.toNumber() }})
  if (savedTwinEntity) {
    await store.remove(savedTwinEntity)
  }
}
