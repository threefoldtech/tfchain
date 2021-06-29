"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.entityStored = exports.balancesTransfer = void 0;
const model_1 = require("../generated/graphql-server/model");
// run 'NODE_URL=<RPC_ENDPOINT> EVENTS=<comma separated list of events> yarn codegen:mappings-types'
// to genenerate typescript classes for events, such as Balances.TransferEvent
const types_1 = require("./generated/types");
async function balancesTransfer({ store, event, block, extrinsic, }) {
    const transfer = new model_1.Transfer();
    const [from, to, value] = new types_1.Balances.TransferEvent(event).params;
    transfer.from = Buffer.from(from.toHex()).toString();
    transfer.to = Buffer.from(to.toHex()).toString();
    transfer.value = value.toBn();
    transfer.block = block.height;
    transfer.comment = `Transferred ${transfer.value} from ${transfer.from} to ${transfer.to}`;
    console.log(`Saving transfer: ${JSON.stringify(transfer, null, 2)}`);
    await store.save(transfer);
}
exports.balancesTransfer = balancesTransfer;
async function entityStored({ store, event, block, extrinsic, }) {
    const entity = new model_1.Entity();
    const [version, entity_id, name, country_id, city_id, account_id] = new types_1.TfgridModule.EntityStoredEvent(event).params;
    entity.gridVersion = version.toNumber();
    entity.entityId = entity_id.toNumber();
    entity.name = name.toString();
    entity.countryId = country_id.toNumber();
    entity.cityId = city_id.toNumber();
    entity.address = Buffer.from(account_id.toHex()).toString();
    await store.save(entity);
}
exports.entityStored = entityStored;
