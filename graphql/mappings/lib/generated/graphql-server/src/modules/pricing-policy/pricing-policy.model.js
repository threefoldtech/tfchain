"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.PricingPolicy = void 0;
const tslib_1 = require("tslib");
const warthog_1 = require("warthog");
let PricingPolicy = class PricingPolicy extends warthog_1.BaseModel {
    constructor(init) {
        super();
        Object.assign(this, init);
    }
};
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], PricingPolicy.prototype, "gridVersion", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], PricingPolicy.prototype, "pricingPolicyId", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], PricingPolicy.prototype, "name", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], PricingPolicy.prototype, "currency", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], PricingPolicy.prototype, "su", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], PricingPolicy.prototype, "cu", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], PricingPolicy.prototype, "nu", void 0);
PricingPolicy = tslib_1.__decorate([
    warthog_1.Model({ api: {} }),
    tslib_1.__metadata("design:paramtypes", [Object])
], PricingPolicy);
exports.PricingPolicy = PricingPolicy;
