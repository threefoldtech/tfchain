"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.NodeContract = exports.ContractState = void 0;
const tslib_1 = require("tslib");
const warthog_1 = require("warthog");
const enums_1 = require("../enums/enums");
Object.defineProperty(exports, "ContractState", { enumerable: true, get: function () { return enums_1.ContractState; } });
let NodeContract = class NodeContract extends warthog_1.BaseModel {
    constructor(init) {
        super();
        Object.assign(this, init);
    }
};
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], NodeContract.prototype, "version", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], NodeContract.prototype, "contractId", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], NodeContract.prototype, "twinId", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], NodeContract.prototype, "nodeId", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], NodeContract.prototype, "deploymentData", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], NodeContract.prototype, "deploymentHash", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], NodeContract.prototype, "numberOfPublicIPs", void 0);
tslib_1.__decorate([
    warthog_1.EnumField('ContractState', enums_1.ContractState, {}),
    tslib_1.__metadata("design:type", String)
], NodeContract.prototype, "state", void 0);
NodeContract = tslib_1.__decorate([
    warthog_1.Model({ api: {} }),
    tslib_1.__metadata("design:paramtypes", [Object])
], NodeContract);
exports.NodeContract = NodeContract;
