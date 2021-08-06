"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Consumption = void 0;
const tslib_1 = require("tslib");
const warthog_1 = require("warthog");
const bn_js_1 = tslib_1.__importDefault(require("bn.js"));
let Consumption = class Consumption extends warthog_1.BaseModel {
    constructor(init) {
        super();
        Object.assign(this, init);
    }
};
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], Consumption.prototype, "contractId", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], Consumption.prototype, "timestamp", void 0);
tslib_1.__decorate([
    warthog_1.NumericField({
        transformer: {
            to: (entityValue) => (entityValue !== undefined ? entityValue.toString(10) : null),
            from: (dbValue) => dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new bn_js_1.default(dbValue, 10) : undefined
        }
    }),
    tslib_1.__metadata("design:type", bn_js_1.default)
], Consumption.prototype, "cru", void 0);
tslib_1.__decorate([
    warthog_1.NumericField({
        transformer: {
            to: (entityValue) => (entityValue !== undefined ? entityValue.toString(10) : null),
            from: (dbValue) => dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new bn_js_1.default(dbValue, 10) : undefined
        }
    }),
    tslib_1.__metadata("design:type", bn_js_1.default)
], Consumption.prototype, "sru", void 0);
tslib_1.__decorate([
    warthog_1.NumericField({
        transformer: {
            to: (entityValue) => (entityValue !== undefined ? entityValue.toString(10) : null),
            from: (dbValue) => dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new bn_js_1.default(dbValue, 10) : undefined
        }
    }),
    tslib_1.__metadata("design:type", bn_js_1.default)
], Consumption.prototype, "hru", void 0);
tslib_1.__decorate([
    warthog_1.NumericField({
        transformer: {
            to: (entityValue) => (entityValue !== undefined ? entityValue.toString(10) : null),
            from: (dbValue) => dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new bn_js_1.default(dbValue, 10) : undefined
        }
    }),
    tslib_1.__metadata("design:type", bn_js_1.default)
], Consumption.prototype, "mru", void 0);
tslib_1.__decorate([
    warthog_1.NumericField({
        transformer: {
            to: (entityValue) => (entityValue !== undefined ? entityValue.toString(10) : null),
            from: (dbValue) => dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new bn_js_1.default(dbValue, 10) : undefined
        }
    }),
    tslib_1.__metadata("design:type", bn_js_1.default)
], Consumption.prototype, "nru", void 0);
Consumption = tslib_1.__decorate([
    warthog_1.Model({ api: {} }),
    tslib_1.__metadata("design:paramtypes", [Object])
], Consumption);
exports.Consumption = Consumption;
