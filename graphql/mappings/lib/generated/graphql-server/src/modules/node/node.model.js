"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Node = void 0;
const tslib_1 = require("tslib");
const warthog_1 = require("warthog");
const bn_js_1 = tslib_1.__importDefault(require("bn.js"));
const location_model_1 = require("../location/location.model");
const public_config_model_1 = require("../public-config/public-config.model");
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
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], Node.prototype, "twinId", void 0);
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
    warthog_1.NumericField({
        nullable: true,
        transformer: {
            to: (entityValue) => (entityValue !== undefined ? entityValue.toString(10) : null),
            from: (dbValue) => dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new bn_js_1.default(dbValue, 10) : undefined
        }
    }),
    tslib_1.__metadata("design:type", bn_js_1.default)
], Node.prototype, "hru", void 0);
tslib_1.__decorate([
    warthog_1.NumericField({
        nullable: true,
        transformer: {
            to: (entityValue) => (entityValue !== undefined ? entityValue.toString(10) : null),
            from: (dbValue) => dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new bn_js_1.default(dbValue, 10) : undefined
        }
    }),
    tslib_1.__metadata("design:type", bn_js_1.default)
], Node.prototype, "sru", void 0);
tslib_1.__decorate([
    warthog_1.NumericField({
        nullable: true,
        transformer: {
            to: (entityValue) => (entityValue !== undefined ? entityValue.toString(10) : null),
            from: (dbValue) => dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new bn_js_1.default(dbValue, 10) : undefined
        }
    }),
    tslib_1.__metadata("design:type", bn_js_1.default)
], Node.prototype, "cru", void 0);
tslib_1.__decorate([
    warthog_1.NumericField({
        nullable: true,
        transformer: {
            to: (entityValue) => (entityValue !== undefined ? entityValue.toString(10) : null),
            from: (dbValue) => dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new bn_js_1.default(dbValue, 10) : undefined
        }
    }),
    tslib_1.__metadata("design:type", bn_js_1.default)
], Node.prototype, "mru", void 0);
tslib_1.__decorate([
    warthog_1.ManyToOne(() => public_config_model_1.PublicConfig, (param) => param.nodepublicConfig, {
        skipGraphQLField: true,
        nullable: true,
        modelName: 'Node',
        relModelName: 'PublicConfig',
        propertyName: 'publicConfig'
    }),
    tslib_1.__metadata("design:type", public_config_model_1.PublicConfig)
], Node.prototype, "publicConfig", void 0);
tslib_1.__decorate([
    warthog_1.IntField({
        nullable: true
    }),
    tslib_1.__metadata("design:type", Number)
], Node.prototype, "uptime", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], Node.prototype, "created", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], Node.prototype, "farmingPolicyId", void 0);
Node = tslib_1.__decorate([
    warthog_1.Model({ api: {} }),
    tslib_1.__metadata("design:paramtypes", [Object])
], Node);
exports.Node = Node;
