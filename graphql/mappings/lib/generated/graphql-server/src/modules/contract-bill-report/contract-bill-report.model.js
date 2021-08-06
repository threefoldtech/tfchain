"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ContractBillReport = void 0;
const tslib_1 = require("tslib");
const warthog_1 = require("warthog");
let ContractBillReport = class ContractBillReport extends warthog_1.BaseModel {
    constructor(init) {
        super();
        Object.assign(this, init);
    }
};
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], ContractBillReport.prototype, "contractId", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], ContractBillReport.prototype, "discountReceived", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], ContractBillReport.prototype, "amountBilled", void 0);
ContractBillReport = tslib_1.__decorate([
    warthog_1.Model({ api: {} }),
    tslib_1.__metadata("design:paramtypes", [Object])
], ContractBillReport);
exports.ContractBillReport = ContractBillReport;
