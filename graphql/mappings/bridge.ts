import { MintTransaction, BurnTransaction, RefundTransaction } from '../generated/model'
import { TFTBridgeModule } from '../chain'
import { hex2a } from './util'

import {
  EventContext,
  StoreContext,
} from '@subsquid/hydra-common'

export async function mintCompleted({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const [mintCompleted] = new TFTBridgeModule.MintCompletedEvent(event).params

  const mintTransaction = new MintTransaction()
  mintTransaction.target = mintCompleted.target.toHuman()
  mintTransaction.amount = mintCompleted.amount.toBn()
  mintTransaction.block = mintCompleted.block.toNumber()

  await store.save<MintTransaction>(mintTransaction)
}

export async function burnProcessed({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const [burnProcessed] = new TFTBridgeModule.BurnTransactionProcessedEvent(event).params

  const burnTransaction = new BurnTransaction()
  burnTransaction.target = hex2a(Buffer.from(burnProcessed.target.toString()).toString())
  burnTransaction.amount = burnProcessed.amount.toBn()
  burnTransaction.block = burnProcessed.block.toNumber()

  await store.save<BurnTransaction>(burnTransaction)
}

export async function refundProcessed({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const [refundProcessed] = new TFTBridgeModule.RefundTransactionProcessedEvent(event).params

  const refundTransaction = new RefundTransaction()
  refundTransaction.target = hex2a(Buffer.from(refundProcessed.target.toString()).toString())
  refundTransaction.amount = refundProcessed.amount.toBn()
  refundTransaction.block = refundProcessed.block.toNumber()
  refundTransaction.txHash = hex2a(Buffer.from(refundProcessed.tx_hash.toString()).toString())

  await store.save<RefundTransaction>(refundTransaction)
}