"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.City = void 0;
const tslib_1 = require("tslib");
const warthog_1 = require("warthog");
let City = class City extends warthog_1.BaseModel {
    constructor(init) {
        super();
        Object.assign(this, init);
    }
};
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], City.prototype, "cityId", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], City.prototype, "countryId", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], City.prototype, "name", void 0);
City = tslib_1.__decorate([
    warthog_1.Model({ api: {} }),
    tslib_1.__metadata("design:paramtypes", [Object])
], City);
exports.City = City;
