"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Country = void 0;
const tslib_1 = require("tslib");
const warthog_1 = require("warthog");
let Country = class Country extends warthog_1.BaseModel {
    constructor(init) {
        super();
        Object.assign(this, init);
    }
};
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], Country.prototype, "countryId", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], Country.prototype, "code", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], Country.prototype, "name", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], Country.prototype, "region", void 0);
tslib_1.__decorate([
    warthog_1.StringField({}),
    tslib_1.__metadata("design:type", String)
], Country.prototype, "subregion", void 0);
tslib_1.__decorate([
    warthog_1.StringField({
        nullable: true
    }),
    tslib_1.__metadata("design:type", String)
], Country.prototype, "lat", void 0);
tslib_1.__decorate([
    warthog_1.StringField({
        nullable: true
    }),
    tslib_1.__metadata("design:type", String)
], Country.prototype, "long", void 0);
Country = tslib_1.__decorate([
    warthog_1.Model({ api: {} }),
    tslib_1.__metadata("design:paramtypes", [Object])
], Country);
exports.Country = Country;
