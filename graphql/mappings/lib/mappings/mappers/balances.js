"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.balancesTransfer = void 0;
const model_1 = require("../../generated/graphql-server/model");
const types_1 = require("../generated/types");
async function balancesTransfer({ store, event, block, extrinsic, }) {
    const transfer = new model_1.Transfer();
    const [from, to, value] = new types_1.Balances.TransferEvent(event).params;
    transfer.from = Buffer.from(from.toHex()).toString();
    transfer.to = Buffer.from(to.toHex()).toString();
    transfer.value = value.toBn();
    transfer.block = block.height;
    transfer.comment = `Transferred ${transfer.value} from ${transfer.from} to ${transfer.to}`;
    await store.save(transfer);
}
exports.balancesTransfer = balancesTransfer;
