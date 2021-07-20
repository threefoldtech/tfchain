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
            this.expectedParamTypes = ["Consumption"];
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
})(SmartContractModule = exports.SmartContractModule || (exports.SmartContractModule = {}));
