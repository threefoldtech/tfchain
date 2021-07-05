"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Location = void 0;
const tslib_1 = require("tslib");
const warthog_1 = require("warthog");
const node_model_1 = require("../node/node.model");
let Location = class Location extends warthog_1.BaseModel {
    constructor(init) {
        super();
        Object.assign(this, init);
    }
};
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], Location.prototype, "longitude", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], Location.prototype, "latitude", void 0);
tslib_1.__decorate([
    warthog_1.OneToMany(() => node_model_1.Node, (param) => param.location, {
        nullable: true,
        modelName: 'Location',
        relModelName: 'Node',
        propertyName: 'nodelocation'
    }),
    tslib_1.__metadata("design:type", Array)
], Location.prototype, "nodelocation", void 0);
Location = tslib_1.__decorate([
    warthog_1.Model({ api: {} }),
    tslib_1.__metadata("design:paramtypes", [Object])
], Location);
exports.Location = Location;
