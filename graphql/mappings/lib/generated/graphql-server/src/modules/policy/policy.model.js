"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Policy = exports.Unit = void 0;
const tslib_1 = require("tslib");
const warthog_1 = require("warthog");
const pricing_policy_model_1 = require("../pricing-policy/pricing-policy.model");
const enums_1 = require("../enums/enums");
Object.defineProperty(exports, "Unit", { enumerable: true, get: function () { return enums_1.Unit; } });
let Policy = class Policy extends warthog_1.BaseModel {
    constructor(init) {
        super();
        Object.assign(this, init);
    }
};
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], Policy.prototype, "value", void 0);
tslib_1.__decorate([
    warthog_1.EnumField('Unit', enums_1.Unit, {}),
    tslib_1.__metadata("design:type", String)
], Policy.prototype, "unit", void 0);
tslib_1.__decorate([
    warthog_1.OneToMany(() => pricing_policy_model_1.PricingPolicy, (param) => param.su, {
        nullable: true,
        modelName: 'Policy',
        relModelName: 'PricingPolicy',
        propertyName: 'pricingpolicysu'
    }),
    tslib_1.__metadata("design:type", Array)
], Policy.prototype, "pricingpolicysu", void 0);
tslib_1.__decorate([
    warthog_1.OneToMany(() => pricing_policy_model_1.PricingPolicy, (param) => param.cu, {
        nullable: true,
        modelName: 'Policy',
        relModelName: 'PricingPolicy',
        propertyName: 'pricingpolicycu'
    }),
    tslib_1.__metadata("design:type", Array)
], Policy.prototype, "pricingpolicycu", void 0);
tslib_1.__decorate([
    warthog_1.OneToMany(() => pricing_policy_model_1.PricingPolicy, (param) => param.nu, {
        nullable: true,
        modelName: 'Policy',
        relModelName: 'PricingPolicy',
        propertyName: 'pricingpolicynu'
    }),
    tslib_1.__metadata("design:type", Array)
], Policy.prototype, "pricingpolicynu", void 0);
tslib_1.__decorate([
    warthog_1.OneToMany(() => pricing_policy_model_1.PricingPolicy, (param) => param.ipu, {
        nullable: true,
        modelName: 'Policy',
        relModelName: 'PricingPolicy',
        propertyName: 'pricingpolicyipu'
    }),
    tslib_1.__metadata("design:type", Array)
], Policy.prototype, "pricingpolicyipu", void 0);
Policy = tslib_1.__decorate([
    warthog_1.Model({ api: {} }),
    tslib_1.__metadata("design:paramtypes", [Object])
], Policy);
exports.Policy = Policy;
