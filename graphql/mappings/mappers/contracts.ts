import { Consumption, NodeContract, ContractState, PublicIp, ContractBillReport } from '../../generated/graphql-server/model'
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
  newConsumptionReport.cru = consumptionReport.cru.toNumber()
  newConsumptionReport.sru = consumptionReport.sru.toNumber()
  newConsumptionReport.hru = consumptionReport.hru.toNumber()
  newConsumptionReport.mru = consumptionReport.mru.toNumber()
  newConsumptionReport.nru = consumptionReport.nru.toNumber()

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

  const savedContract = await store.get(NodeContract, { where: { contract_id: nodeContract.contract_id.toNumber() } })

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

  const savedContract = await store.get(NodeContract, { where: { contract_id: nodeContract.contract_id.toNumber() } })

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
  newContractBilledReport.discountReceived = hex2a(Buffer.from(discount_received.toString()).toString())
  newContractBilledReport.amountBilled = amount_billed.toNumber()

  await store.save<ContractBillReport>(newContractBilledReport)
}
