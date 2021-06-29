"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Transfer = void 0;
const tslib_1 = require("tslib");
const warthog_1 = require("warthog");
const bn_js_1 = tslib_1.__importDefault(require("bn.js"));
let Transfer = class Transfer extends warthog_1.BaseModel {
    constructor(init) {
        super();
        Object.assign(this, init);
    }
};
tslib_1.__decorate([
    warthog_1.BytesField({}),
    tslib_1.__metadata("design:type", Buffer)
], Transfer.prototype, "from", void 0);
tslib_1.__decorate([
    warthog_1.BytesField({}),
    tslib_1.__metadata("design:type", Buffer)
], Transfer.prototype, "to", void 0);
tslib_1.__decorate([
    warthog_1.NumericField({
        transformer: {
            to: (entityValue) => (entityValue !== undefined ? entityValue.toString(10) : null),
            from: (dbValue) => dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new bn_js_1.default(dbValue, 10) : undefined
        }
    }),
    tslib_1.__metadata("design:type", bn_js_1.default)
], Transfer.prototype, "value", void 0);
tslib_1.__decorate([
    warthog_1.StringField({
        nullable: true
    }),
    tslib_1.__metadata("design:type", String)
], Transfer.prototype, "comment", void 0);
tslib_1.__decorate([
    warthog_1.IntField({}),
    tslib_1.__metadata("design:type", Number)
], Transfer.prototype, "block", void 0);
tslib_1.__decorate([
    warthog_1.NumericField({
        transformer: {
            to: (entityValue) => (entityValue !== undefined ? entityValue.toString(10) : null),
            from: (dbValue) => dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new bn_js_1.default(dbValue, 10) : undefined
        }
    }),
    tslib_1.__metadata("design:type", bn_js_1.default)
], Transfer.prototype, "tip", void 0);
tslib_1.__decorate([
    warthog_1.NumericField({
        transformer: {
            to: (entityValue) => (entityValue !== undefined ? entityValue.toString(10) : null),
            from: (dbValue) => dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new bn_js_1.default(dbValue, 10) : undefined
        }
    }),
    tslib_1.__metadata("design:type", bn_js_1.default)
], Transfer.prototype, "timestamp", void 0);
tslib_1.__decorate([
    warthog_1.DateTimeField({}),
    tslib_1.__metadata("design:type", Date)
], Transfer.prototype, "insertedAt", void 0);
Transfer = tslib_1.__decorate([
    warthog_1.Model({ api: { description: ` All transfers ` } }),
    tslib_1.__metadata("design:paramtypes", [Object])
], Transfer);
exports.Transfer = Transfer;
