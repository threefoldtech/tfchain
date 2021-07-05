"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.EntityProof = void 0;
const tslib_1 = require("tslib");
const warthog_1 = require("warthog");
const twin_model_1 = require("../twin/twin.model");
let EntityProof = class EntityProof extends warthog_1.BaseModel {
    constructor(init) {
        super();
        Object.assign(this, init);
    }
};
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], EntityProof.prototype, "entityId", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], EntityProof.prototype, "signature", void 0);
tslib_1.__decorate([
    warthog_1.ManyToOne(() => twin_model_1.Twin, (param) => param.entityprooftwinRel, {
        skipGraphQLField: true,
        modelName: 'EntityProof',
        relModelName: 'Twin',
        propertyName: 'twinRel'
    }),
    tslib_1.__metadata("design:type", twin_model_1.Twin)
], EntityProof.prototype, "twinRel", void 0);
EntityProof = tslib_1.__decorate([
    warthog_1.Model({ api: {} }),
    tslib_1.__metadata("design:paramtypes", [Object])
], EntityProof);
exports.EntityProof = EntityProof;
