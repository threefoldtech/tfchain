"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Node = void 0;
const tslib_1 = require("tslib");
const warthog_1 = require("warthog");
const location_model_1 = require("../location/location.model");
let Node = class Node extends warthog_1.BaseModel {
    constructor(init) {
        super();
        Object.assign(this, init);
    }
};
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], Node.prototype, "gridVersion", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], Node.prototype, "nodeId", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], Node.prototype, "farmId", void 0);
tslib_1.__decorate([
    warthog_1.ManyToOne(() => location_model_1.Location, (param) => param.nodelocation, {
        skipGraphQLField: true,
        modelName: 'Node',
        relModelName: 'Location',
        propertyName: 'location'
    }),
    tslib_1.__metadata("design:type", location_model_1.Location)
], Node.prototype, "location", void 0);
tslib_1.__decorate([
    warthog_1.IntField({
        nullable: true
    }),
    tslib_1.__metadata("design:type", Number)
], Node.prototype, "countryId", void 0);
tslib_1.__decorate([
    warthog_1.IntField({
        nullable: true
    }),
    tslib_1.__metadata("design:type", Number)
], Node.prototype, "cityId", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], Node.prototype, "address", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], Node.prototype, "pubKey", void 0);
tslib_1.__decorate([
    warthog_1.IntField({
        nullable: true
    }),
    tslib_1.__metadata("design:type", Number)
], Node.prototype, "hru", void 0);
tslib_1.__decorate([
    warthog_1.IntField({
        nullable: true
    }),
    tslib_1.__metadata("design:type", Number)
], Node.prototype, "sru", void 0);
tslib_1.__decorate([
    warthog_1.IntField({
        nullable: true
    }),
    tslib_1.__metadata("design:type", Number)
], Node.prototype, "cru", void 0);
tslib_1.__decorate([
    warthog_1.IntField({
        nullable: true
    }),
    tslib_1.__metadata("design:type", Number)
], Node.prototype, "mru", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], Node.prototype, "role", void 0);
Node = tslib_1.__decorate([
    warthog_1.Model({ api: {} }),
    tslib_1.__metadata("design:paramtypes", [Object])
], Node);
exports.Node = Node;
