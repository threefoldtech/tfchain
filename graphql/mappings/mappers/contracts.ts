import { Consumption, NodeContract, ContractState, DiscountLevel, ContractBillReport } from '../../generated/graphql-server/model'
import { SmartContractModule } from '../generated/types'
import { hex2a } from './util'

import {
  EventContext,
  StoreContext,
} from '@subsquid/hydra-common'

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

export async function contractCreated({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const newNodeContract = new NodeContract()
  const [nodeContract] = new SmartContractModule.ContractCreatedEvent(event).params

  newNodeContract.contractId = nodeContract.contract_id.toNumber()
  newNodeContract.version = nodeContract.version.toNumber()
  newNodeContract.twinId = nodeContract.twin_id.toNumber()
  newNodeContract.nodeId = nodeContract.node_id.toNumber()
  newNodeContract.numberOfPublicIPs = nodeContract.public_ips.toNumber()
  newNodeContract.deploymentData = nodeContract.deploy_mentdata.toString()
  newNodeContract.deploymentHash = nodeContract.deployment_hash.toString()

  let state = ContractState.Created
  switch (nodeContract.state.toString()) {
    case 'Created': break
    case 'Deleted': state = ContractState.Deleted
    case 'OutOfFunds': state = ContractState.OutOfFunds
  }
  newNodeContract.state = state

  // await store.save<NodeContract>(newNodeContract)
  
  // const publicIps: PublicIp[] = []
  // nodeContract.public_ips_list.forEach(async ip => {
  //   const savedIp = await store.get(PublicIp, { where: { contract_id: nodeContract.contract_id.toNumber() } })

  //   if (savedIp) {
  //     savedIp.contract = newNodeContract

  //     publicIps.push(savedIp)
  //     await store.save<PublicIp>(savedIp)
  //   }
  // })

  // newNodeContract.publicIPs = publicIps
  await store.save<NodeContract>(newNodeContract)
}

export async function contractUpdated({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const [nodeContract] = new SmartContractModule.ContractCreatedEvent(event).params

  const savedContract = await store.get(NodeContract, { where: { contractId: nodeContract.contract_id.toNumber() } })

  if (!savedContract) return

  savedContract.contractId = nodeContract.contract_id.toNumber()
  savedContract.version = nodeContract.version.toNumber()
  savedContract.twinId = nodeContract.twin_id.toNumber()
  savedContract.nodeId = nodeContract.node_id.toNumber()
  savedContract.numberOfPublicIPs = nodeContract.public_ips.toNumber()
  savedContract.deploymentData = nodeContract.deploy_mentdata.toString()
  savedContract.deploymentHash = nodeContract.deployment_hash.toString()

  let state = ContractState.Created
  switch (nodeContract.state.toString()) {
    case 'Created': break
    case 'Deleted': state = ContractState.Deleted
    case 'OutOfFunds': state = ContractState.OutOfFunds
  }
  savedContract.state = state

  // await store.save<NodeContract>(savedContract)
  
  // const publicIps: PublicIp[] = []
  // nodeContract.public_ips_list.forEach(async ip => {
  //   const savedIp = await store.get(PublicIp, { where: { contract_id: nodeContract.contract_id.toNumber() } })

  //   if (savedIp) {
  //     savedIp.contract = savedContract

  //     publicIps.push(savedIp)
  //     await store.save<PublicIp>(savedIp)
  //   }
  // })

  // savedContract.publicIPs = publicIps
  await store.save<NodeContract>(savedContract)
}

export async function contractCanceled({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const [nodeContract] = new SmartContractModule.ContractCreatedEvent(event).params

  const savedContract = await store.get(NodeContract, { where: { contractId: nodeContract.contract_id.toNumber() } })

  if (!savedContract) return

  savedContract.state = ContractState.Deleted

  await store.save<NodeContract>(savedContract)
}

export async function contractBilled({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const newContractBilledReport = new ContractBillReport()
  const [contract_id, discount_received, amount_billed] = new SmartContractModule.ContractBilledEvent(event).params

  newContractBilledReport.contractId = contract_id.toNumber()

  let level = DiscountLevel.None
  switch (discount_received.toString()) {
    case 'None': break
    case 'Default': level = DiscountLevel.Default
    case 'Bronze': level = DiscountLevel.Bronze
    case 'Silver': level = DiscountLevel.Silver
    case 'Gold': level = DiscountLevel.Gold
  }
  newContractBilledReport.discountReceived = level
  newContractBilledReport.amountBilled = amount_billed.toNumber()

  await store.save<ContractBillReport>(newContractBilledReport)
}