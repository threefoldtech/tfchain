"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Entity = void 0;
const tslib_1 = require("tslib");
const warthog_1 = require("warthog");
let Entity = class Entity extends warthog_1.BaseModel {
    constructor(init) {
        super();
        Object.assign(this, init);
    }
};
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], Entity.prototype, "gridVersion", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], Entity.prototype, "entityId", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], Entity.prototype, "name", void 0);
tslib_1.__decorate([
    warthog_1.IntField({
        nullable: true
    }),
    tslib_1.__metadata("design:type", Number)
], Entity.prototype, "countryId", void 0);
tslib_1.__decorate([
    warthog_1.IntField({
        nullable: true
    }),
    tslib_1.__metadata("design:type", Number)
], Entity.prototype, "cityId", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], Entity.prototype, "address", void 0);
Entity = tslib_1.__decorate([
    warthog_1.Model({ api: {} }),
    tslib_1.__metadata("design:paramtypes", [Object])
], Entity);
exports.Entity = Entity;
