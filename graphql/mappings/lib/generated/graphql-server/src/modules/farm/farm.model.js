"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Farm = exports.CertificationType = void 0;
const tslib_1 = require("tslib");
const warthog_1 = require("warthog");
const public_ip_model_1 = require("../public-ip/public-ip.model");
const enums_1 = require("../enums/enums");
Object.defineProperty(exports, "CertificationType", { enumerable: true, get: function () { return enums_1.CertificationType; } });
let Farm = class Farm extends warthog_1.BaseModel {
    constructor(init) {
        super();
        Object.assign(this, init);
    }
};
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], Farm.prototype, "gridVersion", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], Farm.prototype, "farmId", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], Farm.prototype, "name", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], Farm.prototype, "twinId", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], Farm.prototype, "pricingPolicyId", void 0);
tslib_1.__decorate([
    warthog_1.EnumField('CertificationType', enums_1.CertificationType, {}),
    tslib_1.__metadata("design:type", String)
], Farm.prototype, "certificationType", void 0);
tslib_1.__decorate([
    warthog_1.IntField({
        nullable: true
    }),
    tslib_1.__metadata("design:type", Number)
], Farm.prototype, "countryId", void 0);
tslib_1.__decorate([
    warthog_1.IntField({
        nullable: true
    }),
    tslib_1.__metadata("design:type", Number)
], Farm.prototype, "cityId", void 0);
tslib_1.__decorate([
    warthog_1.OneToMany(() => public_ip_model_1.PublicIp, (param) => param.farm, {
        modelName: 'Farm',
        relModelName: 'PublicIp',
        propertyName: 'publicIPs'
    }),
    tslib_1.__metadata("design:type", Array)
], Farm.prototype, "publicIPs", void 0);
Farm = tslib_1.__decorate([
    warthog_1.Model({ api: {} }),
    tslib_1.__metadata("design:paramtypes", [Object])
], Farm);
exports.Farm = Farm;
