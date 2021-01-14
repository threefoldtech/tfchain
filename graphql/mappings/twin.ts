import { SubstrateEvent, DB } from '../generated/hydra-processor'
import { Twin } from '../generated/graphql-server/src/modules/twin/twin.model'
import { EntityProof } from '../generated/graphql-server/src/modules/entity-proof/entity-proof.model'
import BN from 'bn.js'

export async function templateModule_TwinStored(db: DB, event: SubstrateEvent) {
  const [pubkey, twin_id, peer_id] = event.params
  const twin = new Twin()
  twin.twinId = new BN(twin_id.value as number)
  twin.pubKey = Buffer.from(pubkey.value as string).toString()
  twin.peerId = Buffer.from(peer_id.value as string).toString()

  await db.save<Twin>(twin)
}

export async function templateModule_TwinUpdated(db: DB, event: SubstrateEvent) {
  const [twin_id, peer_id] = event.params

  let savedTwin = await db.get(Twin, { where: { twinId: new BN(twin_id.value as number) } })

  if (savedTwin) {
    // update peer id and save
    savedTwin.peerId = Buffer.from(peer_id.value as string).toString()
    await db.save<Twin>(savedTwin)
  }
}

export async function templateModule_TwinDeleted(db: DB, event: SubstrateEvent) {
  const [twin_id] = event.params

  const savedTwin = await db.get(Twin, { where: { twinId: new BN(twin_id.value as number) } })

  if (savedTwin) {
    await db.remove(savedTwin)
  }
}

/* TWIN ENTITIES */

export async function templateModule_TwinEntityStored(db: DB, event: SubstrateEvent) {
  const [twin_id, entity_id, signature] = event.params

  let savedTwin = await db.get(Twin, { where: { twinId: new BN(twin_id.value as number) } })

  if (savedTwin) {
    const entityProof = new EntityProof()
    entityProof.entityId = new BN(entity_id.value as number)
    entityProof.signature = Buffer.from(signature.value as string).toString()
    
    // and the twin foreign key to entityproof
    entityProof.twinRel = savedTwin

    await db.save<EntityProof>(entityProof)
  }
}