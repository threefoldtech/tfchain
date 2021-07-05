"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.PublicConfig = void 0;
const tslib_1 = require("tslib");
const warthog_1 = require("warthog");
const node_model_1 = require("../node/node.model");
let PublicConfig = class PublicConfig extends warthog_1.BaseModel {
    constructor(init) {
        super();
        Object.assign(this, init);
    }
};
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], PublicConfig.prototype, "ipv4", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], PublicConfig.prototype, "ipv6", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], PublicConfig.prototype, "gw4", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], PublicConfig.prototype, "gw6", void 0);
tslib_1.__decorate([
    warthog_1.OneToMany(() => node_model_1.Node, (param) => param.publicConfig, {
        nullable: true,
        modelName: 'PublicConfig',
        relModelName: 'Node',
        propertyName: 'nodepublicConfig'
    }),
    tslib_1.__metadata("design:type", Array)
], PublicConfig.prototype, "nodepublicConfig", void 0);
PublicConfig = tslib_1.__decorate([
    warthog_1.Model({ api: {} }),
    tslib_1.__metadata("design:paramtypes", [Object])
], PublicConfig);
exports.PublicConfig = PublicConfig;
