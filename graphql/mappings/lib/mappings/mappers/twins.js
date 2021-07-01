"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.twinEntityRemoved = exports.twinEntityStored = exports.twinDeleted = exports.twinStored = void 0;
const model_1 = require("../../generated/graphql-server/model");
const types_1 = require("../generated/types");
const util_1 = require("./util");
async function twinStored({ store, event, block, extrinsic, }) {
    const twin = new model_1.Twin();
    const [version, twin_id, account_id, ip] = new types_1.TfgridModule.TwinStoredEvent(event).params;
    twin.gridVersion = version.toNumber();
    twin.twinId = twin_id.toNumber();
    twin.address = util_1.hex2a(account_id.toString());
    twin.ip = util_1.hex2a(ip.toString());
    await store.save(twin);
}
exports.twinStored = twinStored;
async function twinDeleted({ store, event, block, extrinsic, }) {
    const [twin_id] = new types_1.TfgridModule.TwinDeletedEvent(event).params;
    const savedTwin = await store.get(model_1.Twin, { where: { twinId: twin_id.toNumber() } });
    if (savedTwin) {
        await store.remove(savedTwin);
    }
}
exports.twinDeleted = twinDeleted;
async function twinEntityStored({ store, event, block, extrinsic, }) {
    const entityProof = new model_1.EntityProof();
    const [twin_id, entity_id, signature] = new types_1.TfgridModule.TwinEntityStoredEvent(event).params;
    let savedTwin = await store.get(model_1.Twin, { where: { twinId: twin_id.toNumber() } });
    if (savedTwin) {
        const entityProof = new model_1.EntityProof();
        entityProof.entityId = entity_id.toNumber();
        entityProof.signature = Buffer.from(signature.toString()).toString();
        // and the twin foreign key to entityproof
        entityProof.twinRel = savedTwin;
        await store.save(entityProof);
    }
}
exports.twinEntityStored = twinEntityStored;
async function twinEntityRemoved({ store, event, block, extrinsic, }) {
    const [twin_id, entity_id] = new types_1.TfgridModule.TwinEntityRemovedEvent(event).params;
    let savedTwinEntity = await store.get(model_1.EntityProof, { where: { entityId: entity_id.toNumber() } });
    if (savedTwinEntity) {
        await store.remove(savedTwinEntity);
    }
}
exports.twinEntityRemoved = twinEntityRemoved;
