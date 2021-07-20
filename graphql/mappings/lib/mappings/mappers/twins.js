"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.twinEntityRemoved = exports.twinEntityStored = exports.twinDeleted = exports.twinStored = void 0;
const model_1 = require("../../generated/graphql-server/model");
const types_1 = require("../generated/types");
const util_1 = require("./util");
async function twinStored({ store, event, block, extrinsic, }) {
    const [twin] = new types_1.TfgridModule.TwinStoredEvent(event).params;
    const newTwin = new model_1.Twin();
    newTwin.gridVersion = twin.version.toNumber();
    newTwin.twinId = twin.id.toNumber();
    newTwin.accountId = twin.account_id.toHuman();
    newTwin.ip = util_1.hex2a(Buffer.from(twin.ip.toString()).toString());
    await store.save(newTwin);
}
exports.twinStored = twinStored;
async function twinDeleted({ store, event, block, extrinsic, }) {
    const [twinID] = new types_1.TfgridModule.TwinDeletedEvent(event).params;
    const savedTwin = await store.get(model_1.Twin, { where: { twinId: twinID.toNumber() } });
    if (savedTwin) {
        await store.remove(savedTwin);
    }
}
exports.twinDeleted = twinDeleted;
async function twinEntityStored({ store, event, block, extrinsic, }) {
    const [twinID, entityID, signature] = new types_1.TfgridModule.TwinEntityStoredEvent(event).params;
    let savedTwin = await store.get(model_1.Twin, { where: { twinId: twinID.toNumber() } });
    if (savedTwin) {
        const entityProof = new model_1.EntityProof();
        entityProof.entityId = entityID.toNumber();
        entityProof.signature = Buffer.from(signature.toString()).toString();
        // and the twin foreign key to entityproof
        entityProof.twinRel = savedTwin;
        await store.save(entityProof);
    }
}
exports.twinEntityStored = twinEntityStored;
async function twinEntityRemoved({ store, event, block, extrinsic, }) {
    const [twinID, entityID] = new types_1.TfgridModule.TwinEntityRemovedEvent(event).params;
    let savedTwinEntity = await store.get(model_1.EntityProof, { where: { entityId: entityID.toNumber() } });
    if (savedTwinEntity) {
        await store.remove(savedTwinEntity);
    }
}
exports.twinEntityRemoved = twinEntityRemoved;
