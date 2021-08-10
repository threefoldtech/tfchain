"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.SmartContractModule = void 0;
const create_1 = require("@polkadot/types/create");
const _1 = require(".");
var SmartContractModule;
(function (SmartContractModule) {
    class ConsumptionReportReceivedEvent {
        constructor(ctx) {
            this.ctx = ctx;
            this.expectedParamTypes = ["types::Consumption"];
        }
        get params() {
            return [
                create_1.createTypeUnsafe(_1.typeRegistry, "Consumption", [
                    this.ctx.params[0].value
                ])
            ];
        }
        validateParams() {
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
    SmartContractModule.ConsumptionReportReceivedEvent = ConsumptionReportReceivedEvent;
    class ContractCreatedEvent {
        constructor(ctx) {
            this.ctx = ctx;
            this.expectedParamTypes = ["types::NodeContract"];
        }
        get params() {
            return [
                create_1.createTypeUnsafe(_1.typeRegistry, "NodeContract", [
                    this.ctx.params[0].value
                ])
            ];
        }
        validateParams() {
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
    SmartContractModule.ContractCreatedEvent = ContractCreatedEvent;
    class ContractUpdatedEvent {
        constructor(ctx) {
            this.ctx = ctx;
            this.expectedParamTypes = ["types::NodeContract"];
        }
        get params() {
            return [
                create_1.createTypeUnsafe(_1.typeRegistry, "NodeContract", [
                    this.ctx.params[0].value
                ])
            ];
        }
        validateParams() {
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
    SmartContractModule.ContractUpdatedEvent = ContractUpdatedEvent;
    class ContractCanceledEvent {
        constructor(ctx) {
            this.ctx = ctx;
            this.expectedParamTypes = ["u64"];
        }
        get params() {
            return [
                create_1.createTypeUnsafe(_1.typeRegistry, "u64", [
                    this.ctx.params[0].value
                ])
            ];
        }
        validateParams() {
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
    SmartContractModule.ContractCanceledEvent = ContractCanceledEvent;
    class ContractBilledEvent {
        constructor(ctx) {
            this.ctx = ctx;
            this.expectedParamTypes = ["u64", "Vec<u8>", "u128"];
        }
        get params() {
            return [
                create_1.createTypeUnsafe(_1.typeRegistry, "u64", [
                    this.ctx.params[0].value
                ]),
                create_1.createTypeUnsafe(_1.typeRegistry, "Bytes", [
                    this.ctx.params[1].value
                ]),
                create_1.createTypeUnsafe(_1.typeRegistry, "u128", [
                    this.ctx.params[2].value
                ])
            ];
        }
        validateParams() {
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
    SmartContractModule.ContractBilledEvent = ContractBilledEvent;
})(SmartContractModule = exports.SmartContractModule || (exports.SmartContractModule = {}));
