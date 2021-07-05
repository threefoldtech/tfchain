"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Balances = void 0;
const create_1 = require("@polkadot/types/create");
const _1 = require(".");
var Balances;
(function (Balances) {
    /**
     *  Transfer succeeded. \[from, to, value\]
     *
     *  Event parameters: [AccountId, AccountId, Balance, ]
     */
    class TransferEvent {
        constructor(ctx) {
            this.ctx = ctx;
            this.expectedParamTypes = ["AccountId", "AccountId", "Balance"];
        }
        get params() {
            return [
                create_1.createTypeUnsafe(_1.typeRegistry, "AccountId", [
                    this.ctx.params[0].value
                ]),
                create_1.createTypeUnsafe(_1.typeRegistry, "AccountId", [
                    this.ctx.params[1].value
                ]),
                create_1.createTypeUnsafe(_1.typeRegistry, "Balance", [
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
    Balances.TransferEvent = TransferEvent;
})(Balances = exports.Balances || (exports.Balances = {}));
