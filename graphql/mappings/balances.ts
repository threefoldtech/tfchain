import { Transfer  } from '../generated/model'
import { Balances } from '../types'

import {
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
  await store.save<Transfer>(transfer)
}
