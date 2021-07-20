"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ConsumptionReportReceived = void 0;
const model_1 = require("../../generated/graphql-server/model");
const types_1 = require("../generated/types");
async function ConsumptionReportReceived({ store, event, block, extrinsic, }) {
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
exports.ConsumptionReportReceived = ConsumptionReportReceived;
