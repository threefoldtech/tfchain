"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.entityDeleted = exports.entityUpdated = exports.entityStored = void 0;
const model_1 = require("../../generated/graphql-server/model");
const types_1 = require("../generated/types");
const util_1 = require("./util");
async function entityStored({ store, event, block, extrinsic, }) {
    const entity = new model_1.Entity();
    const [version, entityID, name, countryID, cityID, accountID] = new types_1.TfgridModule.EntityStoredEvent(event).params;
    entity.gridVersion = version.toNumber();
    entity.entityId = entityID.toNumber();
    entity.name = util_1.hex2a(Buffer.from(name.toString()).toString());
    entity.countryId = countryID.toNumber();
    entity.cityId = cityID.toNumber();
    entity.address = accountID.toHuman();
    await store.save(entity);
}
exports.entityStored = entityStored;
async function entityUpdated({ store, event, block, extrinsic, }) {
    const entity = new model_1.Entity();
    const [entityID, name, countryID, cityID, accountID] = new types_1.TfgridModule.EntityUpdatedEvent(event).params;
    const savedEntity = await store.get(model_1.Entity, { where: { entityId: entityID.toNumber() } });
    if (savedEntity) {
        // entity.gridVersion = version.toNumber()
        savedEntity.entityId = entityID.toNumber();
        savedEntity.name = util_1.hex2a(Buffer.from(name.toString()).toString());
        savedEntity.countryId = countryID.toNumber();
        savedEntity.cityId = cityID.toNumber();
        savedEntity.address = accountID.toHuman();
        await store.save(savedEntity);
    }
}
exports.entityUpdated = entityUpdated;
async function entityDeleted({ store, event, block, extrinsic, }) {
    const entity = new model_1.Entity();
    const [entityID] = new types_1.TfgridModule.EntityDeletedEvent(event).params;
    const savedEntity = await store.get(model_1.Entity, { where: { entityId: entityID.toNumber() } });
    if (savedEntity) {
        store.remove(savedEntity);
    }
}
exports.entityDeleted = entityDeleted;
