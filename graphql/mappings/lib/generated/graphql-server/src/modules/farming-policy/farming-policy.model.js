"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.FarmingPolicy = exports.CertificationType = void 0;
const tslib_1 = require("tslib");
const warthog_1 = require("warthog");
const enums_1 = require("../enums/enums");
Object.defineProperty(exports, "CertificationType", { enumerable: true, get: function () { return enums_1.CertificationType; } });
let FarmingPolicy = class FarmingPolicy extends warthog_1.BaseModel {
    constructor(init) {
        super();
        Object.assign(this, init);
    }
};
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], FarmingPolicy.prototype, "version", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], FarmingPolicy.prototype, "farmingPolicyId", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], FarmingPolicy.prototype, "name", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], FarmingPolicy.prototype, "cu", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], FarmingPolicy.prototype, "su", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], FarmingPolicy.prototype, "nu", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], FarmingPolicy.prototype, "ipv4", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], FarmingPolicy.prototype, "timestamp", void 0);
tslib_1.__decorate([
    warthog_1.EnumField('CertificationType', enums_1.CertificationType, {}),
    tslib_1.__metadata("design:type", String)
], FarmingPolicy.prototype, "certificationType", void 0);
FarmingPolicy = tslib_1.__decorate([
    warthog_1.Model({ api: {} }),
    tslib_1.__metadata("design:paramtypes", [Object])
], FarmingPolicy);
exports.FarmingPolicy = FarmingPolicy;
