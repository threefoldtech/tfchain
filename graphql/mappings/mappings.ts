import { Transfer, Entity  } from '../generated/graphql-server/model'

// run 'NODE_URL=<RPC_ENDPOINT> EVENTS=<comma separated list of events> yarn codegen:mappings-types'
// to genenerate typescript classes for events, such as Balances.TransferEvent
import { Balances, TfgridModule } from './generated/types'
import BN from 'bn.js'
import {
  ExtrinsicContext,
  EventContext,
  StoreContext,
} from '@subsquid/hydra-common'

export async function balancesTransfer({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const transfer = new Transfer()
  const [from, to, value] = new Balances.TransferEvent(event).params
  transfer.from = Buffer.from(from.toHex()).toString()
  transfer.to = Buffer.from(to.toHex()).toString()
  transfer.value = value.toBn()

  transfer.block = block.height
  transfer.comment = `Transferred ${transfer.value} from ${transfer.from} to ${transfer.to}`
  console.log(`Saving transfer: ${JSON.stringify(transfer, null, 2)}`)
  await store.save<Transfer>(transfer)
}

export async function entityStored({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const entity = new Entity()
  const [version, entity_id, name, country_id, city_id, account_id] = new TfgridModule.EntityStoredEvent(event).params

  entity.version = version.toNumber()
  entity.entityId = entity_id.toNumber()
  entity.name = name.toString()
  entity.countryId = country_id.toNumber()
  entity.cityId = city_id.toNumber()
  entity.address = Buffer.from(account_id.toHex()).toString()

  await store.save<Entity>(entity)
}