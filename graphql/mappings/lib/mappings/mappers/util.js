"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hex2a = void 0;
function hex2a(hex) {
    var str = '';
    for (var i = 0; i < hex.length; i += 2) {
        var v = parseInt(hex.substr(i, 2), 16);
        if (v)
            str += String.fromCharCode(v);
    }
    return str;
}
exports.hex2a = hex2a;
