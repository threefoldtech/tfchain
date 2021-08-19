"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.DiscountLevel = exports.ContractState = exports.Unit = exports.CertificationType = void 0;
var CertificationType;
(function (CertificationType) {
    CertificationType["Diy"] = "Diy";
    CertificationType["Certified"] = "Certified";
})(CertificationType = exports.CertificationType || (exports.CertificationType = {}));
var Unit;
(function (Unit) {
    Unit["Bytes"] = "Bytes";
    Unit["Kilobytes"] = "Kilobytes";
    Unit["Megabytes"] = "Megabytes";
    Unit["Gigabytes"] = "Gigabytes";
    Unit["Terrabytes"] = "Terrabytes";
})(Unit = exports.Unit || (exports.Unit = {}));
var ContractState;
(function (ContractState) {
    ContractState["Created"] = "Created";
    ContractState["Deleted"] = "Deleted";
    ContractState["OutOfFunds"] = "OutOfFunds";
})(ContractState = exports.ContractState || (exports.ContractState = {}));
var DiscountLevel;
(function (DiscountLevel) {
    DiscountLevel["None"] = "None";
    DiscountLevel["Default"] = "Default";
    DiscountLevel["Bronze"] = "Bronze";
    DiscountLevel["Silver"] = "Silver";
    DiscountLevel["Gold"] = "Gold";
})(DiscountLevel = exports.DiscountLevel || (exports.DiscountLevel = {}));
