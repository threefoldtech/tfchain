import { Consumption, NodeContract, NameContract, ContractState, DiscountLevel, ContractBillReport } from '../generated/model'
import { SmartContractModule } from '../chain'
import { hex2a } from './util'

import {
  EventContext,
  StoreContext,
} from '@subsquid/hydra-common'

export async function contractCreated({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const [nodeContract] = new SmartContractModule.ContractCreatedEvent(event).params

  let state = ContractState.Created
  switch (nodeContract.state.toString()) {
    case 'Created': break
    case 'Deleted': state = ContractState.Deleted
    case 'OutOfFunds': state = ContractState.OutOfFunds
  }

  let contract
  if (nodeContract.contract_type.isNameContract) {
    let newNameContract = new NameContract()
    contract = nodeContract.contract_type.asNameContract
    newNameContract.name = hex2a(Buffer.from(contract.name.toString()).toString())
    newNameContract.contractId = nodeContract.contract_id.toNumber()
    newNameContract.version = nodeContract.version.toNumber()
    newNameContract.twinId = nodeContract.twin_id.toNumber()
    newNameContract.state = state
    await store.save<NameContract>(newNameContract)
  }
  else if (nodeContract.contract_type.isNodeContract) {
    let newNodeContract = new NodeContract()
    contract = nodeContract.contract_type.asNodeContract
    newNodeContract.contractId = nodeContract.contract_id.toNumber()
    newNodeContract.version = nodeContract.version.toNumber()
    newNodeContract.twinId = nodeContract.twin_id.toNumber()
    newNodeContract.nodeId = contract.node_id.toNumber()
    newNodeContract.numberOfPublicIPs = contract.public_ips.toNumber()
    newNodeContract.deploymentData = contract.deployment_data.toString()
    newNodeContract.deploymentHash = contract.deployment_hash.toString()
    newNodeContract.state = state
    await store.save<NodeContract>(newNodeContract)
  }
}

export async function contractUpdated({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const [nodeContract] = new SmartContractModule.ContractUpdatedEvent(event).params

  const savedContract = await store.get(NodeContract, { where: { contractId: nodeContract.contract_id.toNumber() } })

  if (!savedContract) return

  const parsedNodeContract = nodeContract.contract_type.asNodeContract

  savedContract.contractId = nodeContract.contract_id.toNumber()
  savedContract.version = nodeContract.version.toNumber()
  savedContract.twinId = nodeContract.twin_id.toNumber()
  savedContract.nodeId = parsedNodeContract.node_id.toNumber()
  savedContract.numberOfPublicIPs = parsedNodeContract.public_ips.toNumber()
  savedContract.deploymentData = parsedNodeContract.deployment_data.toString()
  savedContract.deploymentHash = parsedNodeContract.deployment_hash.toString()

  let state = ContractState.Created
  switch (nodeContract.state.toString()) {
    case 'Created': break
    case 'Deleted': state = ContractState.Deleted
    case 'OutOfFunds': state = ContractState.OutOfFunds
  }
  savedContract.state = state

  await store.save<NodeContract>(savedContract)
}

export async function contractCanceled({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const [id] = new SmartContractModule.ContractCanceledEvent(event).params

  const savedContract = await store.get(NodeContract, { where: { contractId: id.toNumber() } })

  if (!savedContract) return

  savedContract.state = ContractState.Deleted

  await store.save<NodeContract>(savedContract)
}

export async function consumptionReportReceived({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const newConsumptionReport = new Consumption()
  const [consumptionReport] = new SmartContractModule.ConsumptionReportReceivedEvent(event).params

  newConsumptionReport.contractId = consumptionReport.contract_id.toNumber()
  newConsumptionReport.timestamp = consumptionReport.timestamp.toNumber()
  newConsumptionReport.cru = consumptionReport.cru.toBn()
  newConsumptionReport.sru = consumptionReport.sru.toBn()
  newConsumptionReport.hru = consumptionReport.hru.toBn()
  newConsumptionReport.mru = consumptionReport.mru.toBn()
  newConsumptionReport.nru = consumptionReport.nru.toBn()

  await store.save<Consumption>(newConsumptionReport)
}

export async function contractBilled({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const newContractBilledReport = new ContractBillReport()
  const [contract_billed_event] = new SmartContractModule.ContractBilledEvent(event).params

  newContractBilledReport.contractId = contract_billed_event.contract_id.toNumber()

  let level = DiscountLevel.None
  switch (contract_billed_event.discount_level.toString()) {
    case 'None': break
    case 'Default': level = DiscountLevel.Default
    case 'Bronze': level = DiscountLevel.Bronze
    case 'Silver': level = DiscountLevel.Silver
    case 'Gold': level = DiscountLevel.Gold
  }
  newContractBilledReport.discountReceived = level
  newContractBilledReport.amountBilled = contract_billed_event.amount_billed.toNumber()
  newContractBilledReport.timestamp = contract_billed_event.timestamp.toNumber()

  await store.save<ContractBillReport>(newContractBilledReport)
}