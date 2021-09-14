import { Twin, EntityProof } from '../generated/model'
import { TfgridModule } from '../chain'
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
  const [twin] = new TfgridModule.TwinStoredEvent(event).params
  const newTwin = new Twin()

  newTwin.gridVersion = twin.version.toNumber()
  newTwin.twinId = twin.id.toNumber()
  newTwin.accountId = twin.account_id.toHuman()
  newTwin.ip = hex2a(Buffer.from(twin.ip.toString()).toString())

  await store.save<Twin>(newTwin)
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
