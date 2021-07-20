"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.entityDeleted = exports.entityUpdated = exports.entityStored = void 0;
const model_1 = require("../../generated/graphql-server/model");
const types_1 = require("../generated/types");
const util_1 = require("./util");
async function entityStored({ store, event, block, extrinsic, }) {
    const newEntity = new model_1.Entity();
    const [entity] = new types_1.TfgridModule.EntityStoredEvent(event).params;
    newEntity.gridVersion = entity.version.toNumber();
    newEntity.entityId = entity.id.toNumber();
    newEntity.name = util_1.hex2a(Buffer.from(entity.name.toString()).toString());
    newEntity.countryId = entity.country_id.toNumber();
    newEntity.cityId = entity.city_id.toNumber();
    newEntity.accountId = entity.account_id.toHuman();
    await store.save(newEntity);
}
exports.entityStored = entityStored;
async function entityUpdated({ store, event, block, extrinsic, }) {
    const newEntity = new model_1.Entity();
    const [entity] = new types_1.TfgridModule.EntityUpdatedEvent(event).params;
    const savedEntity = await store.get(model_1.Entity, { where: { entityId: entity.id.toNumber() } });
    if (savedEntity) {
        newEntity.gridVersion = entity.version.toNumber();
        newEntity.entityId = entity.id.toNumber();
        newEntity.name = util_1.hex2a(Buffer.from(entity.name.toString()).toString());
        newEntity.countryId = entity.country_id.toNumber();
        newEntity.cityId = entity.city_id.toNumber();
        newEntity.accountId = entity.account_id.toHuman();
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
