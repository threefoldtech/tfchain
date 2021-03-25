import { SubstrateEvent, DB } from '../generated/hydra-processor'
import { ExecutedVestingWithdrawal } from '../generated/graphql-server/src/modules/executed-vesting-withdrawal/executed-vesting-withdrawal.model'
import { ExpiredVestingWithdrawal } from '../generated/graphql-server/src/modules/expired-vesting-withdrawal/expired-vesting-withdrawal.model'
import { FailedVestingWithdrawal } from '../generated/graphql-server/src/modules/failed-vesting-withdrawal/failed-vesting-withdrawal.model'

import { hex2a } from './util'
import StellarSdk from 'stellar-sdk'

export async function vestingValidatorModule_TransactionRemoved(db: DB, event: SubstrateEvent) {
  const [tx_id] = event.params
  const transfer = new ExecutedVestingWithdrawal()

  const transactionXDR = hex2a(Buffer.from(tx_id.value as string).toString())
  // parse transaction from xdr string
  const transaction = new StellarSdk.Transaction(transactionXDR, StellarSdk.Networks.TESTNET)

  const operation = transaction.operations[0]
  transfer.to = operation.destination
  transfer.value = parseFloat(operation.amount) 

  transfer.from = transaction.source
  transfer.block = event.blockNumber
  transfer.txXdr = transactionXDR

  await db.save<ExecutedVestingWithdrawal>(transfer)
}

export async function vestingValidatorModule_TransactionExpired(db: DB, event: SubstrateEvent) {
  const [tx_id] = event.params
  const transfer = new ExpiredVestingWithdrawal()

  const transactionXDR = hex2a(Buffer.from(tx_id.value as string).toString())
  // parse transaction from xdr string
  const transaction = new StellarSdk.Transaction(transactionXDR, StellarSdk.Networks.TESTNET)

  const operation = transaction.operations[0]
  transfer.to = operation.destination
  transfer.value = parseFloat(operation.amount) 

  transfer.from = transaction.source
  transfer.block = event.blockNumber
  transfer.txXdr = transactionXDR

  await db.save<ExpiredVestingWithdrawal>(transfer)
}

export async function vestingValidatorModule_TransactionFailed(db: DB, event: SubstrateEvent) {
  const [tx_id, reason] = event.params
  const transfer = new FailedVestingWithdrawal()

  const transactionXDR = hex2a(Buffer.from(tx_id.value as string).toString())
  // parse transaction from xdr string
  const transaction = new StellarSdk.Transaction(transactionXDR, StellarSdk.Networks.TESTNET)

  const operation = transaction.operations[0]
  transfer.to = operation.destination
  transfer.value = parseFloat(operation.amount) 

  transfer.from = transaction.source
  transfer.block = event.blockNumber
  transfer.txXdr = transactionXDR
  transfer.reason = hex2a(Buffer.from(reason.value as string).toString())

  await db.save<FailedVestingWithdrawal>(transfer)
}