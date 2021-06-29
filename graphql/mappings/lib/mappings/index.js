"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
// Export here all the event handler functions
// so that the indexer picks them up
//export { balancesTransfer as balances_Transfer } from './transfer'
var mappings_1 = require("./mappings");
Object.defineProperty(exports, "balancesTransfer", { enumerable: true, get: function () { return mappings_1.balancesTransfer; } });
Object.defineProperty(exports, "entityStored", { enumerable: true, get: function () { return mappings_1.entityStored; } });
