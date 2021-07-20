"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Twin = void 0;
const tslib_1 = require("tslib");
const warthog_1 = require("warthog");
const entity_proof_model_1 = require("../entity-proof/entity-proof.model");
let Twin = class Twin extends warthog_1.BaseModel {
    constructor(init) {
        super();
        Object.assign(this, init);
    }
};
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], Twin.prototype, "gridVersion", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], Twin.prototype, "twinId", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], Twin.prototype, "accountId", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], Twin.prototype, "ip", void 0);
tslib_1.__decorate([
    warthog_1.OneToMany(() => entity_proof_model_1.EntityProof, (param) => param.twinRel, {
        nullable: true,
        modelName: 'Twin',
        relModelName: 'EntityProof',
        propertyName: 'entityprooftwinRel'
    }),
    tslib_1.__metadata("design:type", Array)
], Twin.prototype, "entityprooftwinRel", void 0);
Twin = tslib_1.__decorate([
    warthog_1.Model({ api: {} }),
    tslib_1.__metadata("design:paramtypes", [Object])
], Twin);
exports.Twin = Twin;
