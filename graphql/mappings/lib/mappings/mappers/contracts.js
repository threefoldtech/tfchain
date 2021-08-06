"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.contractBilled = exports.contractCanceled = exports.contractUpdated = exports.contractCreated = exports.consumptionReportReceived = void 0;
const model_1 = require("../../generated/graphql-server/model");
const types_1 = require("../generated/types");
const util_1 = require("./util");
async function consumptionReportReceived({ store, event, block, extrinsic, }) {
    const newConsumptionReport = new model_1.Consumption();
    const [consumptionReport] = new types_1.SmartContractModule.ConsumptionReportReceivedEvent(event).params;
    newConsumptionReport.contractId = consumptionReport.contract_id.toNumber();
    newConsumptionReport.timestamp = consumptionReport.timestamp.toNumber();
    newConsumptionReport.cru = consumptionReport.cru.toNumber();
    newConsumptionReport.sru = consumptionReport.sru.toNumber();
    newConsumptionReport.hru = consumptionReport.hru.toNumber();
    newConsumptionReport.mru = consumptionReport.mru.toNumber();
    newConsumptionReport.nru = consumptionReport.nru.toNumber();
    await store.save(newConsumptionReport);
}
exports.consumptionReportReceived = consumptionReportReceived;
async function contractCreated({ store, event, block, extrinsic, }) {
    const newNodeContract = new model_1.NodeContract();
    const [nodeContract] = new types_1.SmartContractModule.ContractCreatedEvent(event).params;
    newNodeContract.contractId = nodeContract.contract_id.toNumber();
    newNodeContract.version = nodeContract.version.toNumber();
    newNodeContract.twinId = nodeContract.twin_id.toNumber();
    newNodeContract.nodeId = nodeContract.node_id.toNumber();
    newNodeContract.numberOfPublicIPs = nodeContract.public_ips.toNumber();
    newNodeContract.deploymentData = nodeContract.deploy_mentdata.toString();
    newNodeContract.deploymentHash = nodeContract.deployment_hash.toString();
    let state = model_1.ContractState.Created;
    switch (nodeContract.state.toString()) {
        case 'Created': break;
        case 'Deleted': state = model_1.ContractState.Deleted;
        case 'OutOfFunds': state = model_1.ContractState.OutOfFunds;
    }
    newNodeContract.state = state;
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
    await store.save(newNodeContract);
}
exports.contractCreated = contractCreated;
async function contractUpdated({ store, event, block, extrinsic, }) {
    const [nodeContract] = new types_1.SmartContractModule.ContractCreatedEvent(event).params;
    const savedContract = await store.get(model_1.NodeContract, { where: { contract_id: nodeContract.contract_id.toNumber() } });
    if (!savedContract)
        return;
    savedContract.contractId = nodeContract.contract_id.toNumber();
    savedContract.version = nodeContract.version.toNumber();
    savedContract.twinId = nodeContract.twin_id.toNumber();
    savedContract.nodeId = nodeContract.node_id.toNumber();
    savedContract.numberOfPublicIPs = nodeContract.public_ips.toNumber();
    savedContract.deploymentData = nodeContract.deploy_mentdata.toString();
    savedContract.deploymentHash = nodeContract.deployment_hash.toString();
    let state = model_1.ContractState.Created;
    switch (nodeContract.state.toString()) {
        case 'Created': break;
        case 'Deleted': state = model_1.ContractState.Deleted;
        case 'OutOfFunds': state = model_1.ContractState.OutOfFunds;
    }
    savedContract.state = state;
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
    await store.save(savedContract);
}
exports.contractUpdated = contractUpdated;
async function contractCanceled({ store, event, block, extrinsic, }) {
    const [nodeContract] = new types_1.SmartContractModule.ContractCreatedEvent(event).params;
    const savedContract = await store.get(model_1.NodeContract, { where: { contract_id: nodeContract.contract_id.toNumber() } });
    if (!savedContract)
        return;
    savedContract.state = model_1.ContractState.Deleted;
    await store.save(savedContract);
}
exports.contractCanceled = contractCanceled;
async function contractBilled({ store, event, block, extrinsic, }) {
    const newContractBilledReport = new model_1.ContractBillReport();
    const [contract_id, discount_received, amount_billed] = new types_1.SmartContractModule.ContractBilledEvent(event).params;
    newContractBilledReport.contractId = contract_id.toNumber();
    newContractBilledReport.discountReceived = util_1.hex2a(Buffer.from(discount_received.toString()).toString());
    newContractBilledReport.amountBilled = amount_billed.toNumber();
    await store.save(newContractBilledReport);
}
exports.contractBilled = contractBilled;
