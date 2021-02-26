import { SubstrateEvent, DB } from '../generated/hydra-processor'
import { Twin } from '../generated/graphql-server/src/modules/twin/twin.model'
import { EntityProof } from '../generated/graphql-server/src/modules/entity-proof/entity-proof.model'
import { hex2a } from './util'
import BN from 'bn.js'

export async function tfgridModule_TwinStored(db: DB, event: SubstrateEvent) {
  const [version, twin_id, address, ip] = event.params
  const twin = new Twin()

  twin.gridVersion = version.value as number
  twin.twinId = twin_id.value as number
  twin.address = Buffer.from(address.value as string).toString()
  twin.ip = hex2a(Buffer.from(ip.value as string).toString())

  await db.save<Twin>(twin) 
}

export async function tfgridModule_TwinDeleted(db: DB, event: SubstrateEvent) {
  const [twin_id] = event.params

  const savedTwin = await db.get(Twin, { where: { twinId: twin_id.value as number } })

  if (savedTwin) {
    await db.remove(savedTwin)
  }
}

/* TWIN ENTITIES */

export async function tfgridModule_TwinEntityStored(db: DB, event: SubstrateEvent) {
  const [twin_id, entity_id, signature] = event.params

  let savedTwin = await db.get(Twin, { where: { twinId: twin_id.value as number } })

  if (savedTwin) {
    const entityProof = new EntityProof()
    entityProof.entityId = entity_id.value as number
    entityProof.signature = Buffer.from(signature.value as string).toString()
    
    // and the twin foreign key to entityproof
    entityProof.twinRel = savedTwin

    await db.save<EntityProof>(entityProof)
  }
}

export async function tfgridModule_TwinEntityRemoved(db: DB, event: SubstrateEvent) {
  const [twin_id, entity_id] = event.params

  let savedTwinEntity = await db.get(EntityProof, { where: { entityId: new BN(entity_id.value as number) }})
  if (savedTwinEntity) {
    await db.remove(savedTwinEntity)
  }
}