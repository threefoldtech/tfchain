"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.PricingPolicy = void 0;
const tslib_1 = require("tslib");
const warthog_1 = require("warthog");
const policy_model_1 = require("../policy/policy.model");
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
    warthog_1.ManyToOne(() => policy_model_1.Policy, (param) => param.pricingpolicysu, {
        skipGraphQLField: true,
        modelName: 'PricingPolicy',
        relModelName: 'Policy',
        propertyName: 'su'
    }),
    tslib_1.__metadata("design:type", policy_model_1.Policy)
], PricingPolicy.prototype, "su", void 0);
tslib_1.__decorate([
    warthog_1.ManyToOne(() => policy_model_1.Policy, (param) => param.pricingpolicycu, {
        skipGraphQLField: true,
        modelName: 'PricingPolicy',
        relModelName: 'Policy',
        propertyName: 'cu'
    }),
    tslib_1.__metadata("design:type", policy_model_1.Policy)
], PricingPolicy.prototype, "cu", void 0);
tslib_1.__decorate([
    warthog_1.ManyToOne(() => policy_model_1.Policy, (param) => param.pricingpolicynu, {
        skipGraphQLField: true,
        modelName: 'PricingPolicy',
        relModelName: 'Policy',
        propertyName: 'nu'
    }),
    tslib_1.__metadata("design:type", policy_model_1.Policy)
], PricingPolicy.prototype, "nu", void 0);
tslib_1.__decorate([
    warthog_1.ManyToOne(() => policy_model_1.Policy, (param) => param.pricingpolicyipu, {
        skipGraphQLField: true,
        modelName: 'PricingPolicy',
        relModelName: 'Policy',
        propertyName: 'ipu'
    }),
    tslib_1.__metadata("design:type", policy_model_1.Policy)
], PricingPolicy.prototype, "ipu", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], PricingPolicy.prototype, "foundationAccount", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], PricingPolicy.prototype, "certifiedSalesAccount", void 0);
PricingPolicy = tslib_1.__decorate([
    warthog_1.Model({ api: {} }),
    tslib_1.__metadata("design:paramtypes", [Object])
], PricingPolicy);
exports.PricingPolicy = PricingPolicy;
