import { Consumption, NodeContract, NameContract, ContractState, DiscountLevel, ContractBillReport, PublicIp } from '../generated/model'
import { SmartContractModule } from '../chain'
import { hex2a } from './util'

import {
  EventContext,
  StoreContext,
  DatabaseManager,
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
    case 'Deleted': 
      state = ContractState.Deleted
      break
    case 'OutOfFunds': 
      state = ContractState.OutOfFunds
      break
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

    contract.public_ips_list.forEach(async ip => {
      const savedIp = await store.get(PublicIp, { where: { ip: hex2a(Buffer.from(ip.ip.toString()).toString()) } })

      if (savedIp) {
        savedIp.contractId = newNodeContract.contractId
        await store.save<PublicIp>(savedIp)
      }
    })
  }
}

export async function contractUpdated({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const [contract] = new SmartContractModule.ContractUpdatedEvent(event).params

  const SavedNodeContract = await store.get(NodeContract, { where: { contractId: contract.contract_id.toNumber() } })
  if (SavedNodeContract) {
    await updateNodeContract(contract, SavedNodeContract, store)
  }

  const SavedNameContract = await store.get(NameContract, { where: { contractId: contract.contract_id.toNumber() } })
  if (SavedNameContract) {
    await updateNameContract(contract, SavedNameContract, store)
  }
}

async function updateNodeContract(ctr: any, contract: NodeContract, store: DatabaseManager) {
  const parsedNodeContract = ctr.contract_type.asNodeContract

  contract.contractId = ctr.contract_id.toNumber()
  contract.version = ctr.version.toNumber()
  contract.twinId = ctr.twin_id.toNumber()
  contract.nodeId = parsedNodeContract.node_id.toNumber()
  contract.numberOfPublicIPs = parsedNodeContract.public_ips.toNumber()
  contract.deploymentData = parsedNodeContract.deployment_data.toString()
  contract.deploymentHash = parsedNodeContract.deployment_hash.toString()

  let state = ContractState.Created
  switch (ctr.state.toString()) {
    case 'Created': break
    case 'Deleted':
      state = ContractState.Deleted
      break
    case 'OutOfFunds': 
      state = ContractState.OutOfFunds
      break
  }
  contract.state = state
  await store.save<NodeContract>(contract)
}

async function updateNameContract(ctr: any, contract: NameContract, store: DatabaseManager) {
  const parsedNameContract = ctr.contract_type.asNameContract

  contract.contractId = ctr.contract_id.toNumber()
  contract.version = ctr.version.toNumber()
  contract.twinId = ctr.twin_id.toNumber()
  contract.name = hex2a(Buffer.from(contract.name.toString()).toString())

  let state = ContractState.Created
  switch (parsedNameContract.state.toString()) {
    case 'Created': break
    case 'Deleted': 
      state = ContractState.Deleted
      break
    case 'OutOfFunds': 
      state = ContractState.OutOfFunds
      break
  }
  contract.state = state

  await store.save<NameContract>(contract)
}

export async function nodeContractCanceled({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const [id] = new SmartContractModule.NodeContractCanceledEvent(event).params

  const savedContract = await store.get(NodeContract, { where: { contractId: id.toNumber() } })

  if (!savedContract) return

  const savedIps = await store.getMany(PublicIp, { where: { contractId: id.toNumber() } })
  await savedIps.forEach(async ip => {
    ip.contractId = 0
    await store.save<PublicIp>(ip)
  })

  savedContract.state = ContractState.Deleted

  console.log(`saving contract to delete state ${id}`)

  await store.save<NodeContract>(savedContract)
}

export async function contractCanceled({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const [id] = new ContractCanceledEvent(event).params

  const savedContract = await store.get(NodeContract, { where: { contractId: id.toNumber() } })

  if (!savedContract) return

  const savedIps = await store.getMany(PublicIp, { where: { contractId: id.toNumber() } })
  await savedIps.forEach(async ip => {
    ip.contractId = 0
    await store.save<PublicIp>(ip)
  })

  savedContract.state = ContractState.Deleted

  console.log(`saving contract to delete state ${id}`)

  await store.save<NodeContract>(savedContract)
}

export async function nameContractCanceled({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const [id] = new SmartContractModule.NameContractCanceledEvent(event).params

  const savedContract = await store.get(NameContract, { where: { contractId: id.toNumber() } })

  if (!savedContract) return

  savedContract.state = ContractState.Deleted

  await store.save<NameContract>(savedContract)
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
    case 'Default':
      level = DiscountLevel.Default
      break
    case 'Bronze': 
      level = DiscountLevel.Bronze
      break
    case 'Silver': 
      level = DiscountLevel.Silver
      break
    case 'Gold': level = DiscountLevel.Gold
  }
  newContractBilledReport.discountReceived = level
  newContractBilledReport.amountBilled = contract_billed_event.amount_billed.toBn()
  newContractBilledReport.timestamp = contract_billed_event.timestamp.toNumber()

  await store.save<ContractBillReport>(newContractBilledReport)
}


// Deprecated event types
import { createTypeUnsafe } from "@polkadot/types/create";
import { SubstrateEvent } from "@subsquid/hydra-common";
import { Codec } from "@polkadot/types/types";
import { typeRegistry } from "../chain/index";

import { u64 } from "@polkadot/types";

export class ContractCanceledEvent {
  public readonly expectedParamTypes = ["u64"];

  constructor(public readonly ctx: SubstrateEvent) {}

  get params(): [u64] {
    return [
      createTypeUnsafe<u64 & Codec>(typeRegistry, "u64", [
        this.ctx.params[0].value,
      ]),
    ];
  }

  validateParams(): boolean {
    if (this.expectedParamTypes.length !== this.ctx.params.length) {
      return false;
    }
    let valid = true;
    this.expectedParamTypes.forEach((type, i) => {
      if (type !== this.ctx.params[i].type) {
        valid = false;
      }
    });
    return valid;
  }
}