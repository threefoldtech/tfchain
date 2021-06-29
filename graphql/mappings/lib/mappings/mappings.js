"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.timestampCall = exports.balancesTransfer = void 0;
const tslib_1 = require("tslib");
const model_1 = require("../generated/graphql-server/model");
// run 'NODE_URL=<RPC_ENDPOINT> EVENTS=<comma separated list of events> yarn codegen:mappings-types'
// to genenerate typescript classes for events, such as Balances.TransferEvent
const types_1 = require("./generated/types");
const bn_js_1 = tslib_1.__importDefault(require("bn.js"));
async function balancesTransfer({ store, event, block, extrinsic, }) {
    const transfer = new model_1.Transfer();
    const [from, to, value] = new types_1.Balances.TransferEvent(event).params;
    transfer.from = Buffer.from(from.toHex());
    transfer.to = Buffer.from(to.toHex());
    transfer.value = value.toBn();
    transfer.tip = extrinsic ? new bn_js_1.default(extrinsic.tip.toString(10)) : new bn_js_1.default(0);
    transfer.insertedAt = new Date(block.timestamp);
    transfer.block = block.height;
    transfer.comment = `Transferred ${transfer.value} from ${transfer.from} to ${transfer.to}`;
    transfer.timestamp = new bn_js_1.default(block.timestamp);
    console.log(`Saving transfer: ${JSON.stringify(transfer, null, 2)}`);
    await store.save(transfer);
}
exports.balancesTransfer = balancesTransfer;
async function timestampCall({ store, event, block, }) {
    const call = new types_1.Timestamp.SetCall(event);
    const blockT = new model_1.BlockTimestamp();
    blockT.timestamp = call.args.now.toBn();
    blockT.blockNumber = block.height;
    await store.save(blockT);
}
exports.timestampCall = timestampCall;
