"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.entityDeleted = exports.entityUpdated = exports.entityStored = void 0;
const model_1 = require("../../generated/graphql-server/model");
const types_1 = require("../generated/types");
const util_1 = require("./util");
async function entityStored({ store, event, block, extrinsic, }) {
    const entity = new model_1.Entity();
    const [version, entity_id, name, country_id, city_id, account_id] = new types_1.TfgridModule.EntityStoredEvent(event).params;
    entity.gridVersion = version.toNumber();
    entity.entityId = entity_id.toNumber();
    entity.name = util_1.hex2a(name.toString());
    entity.countryId = country_id.toNumber();
    entity.cityId = city_id.toNumber();
    entity.address = util_1.hex2a(account_id.toString());
    await store.save(entity);
}
exports.entityStored = entityStored;
async function entityUpdated({ store, event, block, extrinsic, }) {
    const entity = new model_1.Entity();
    const [entity_id, name, country_id, city_id, account_id] = new types_1.TfgridModule.EntityUpdatedEvent(event).params;
    const savedEntity = await store.get(model_1.Entity, { where: { entityId: entity_id.toNumber() } });
    if (savedEntity) {
        // entity.gridVersion = version.toNumber()
        savedEntity.entityId = entity_id.toNumber();
        savedEntity.name = util_1.hex2a(name.toString());
        savedEntity.countryId = country_id.toNumber();
        savedEntity.cityId = city_id.toNumber();
        savedEntity.address = util_1.hex2a(account_id.toString());
        await store.save(savedEntity);
    }
}
exports.entityUpdated = entityUpdated;
async function entityDeleted({ store, event, block, extrinsic, }) {
    const entity = new model_1.Entity();
    const [entity_id] = new types_1.TfgridModule.EntityDeletedEvent(event).params;
    const savedEntity = await store.get(model_1.Entity, { where: { entityId: entity_id.toNumber() } });
    if (savedEntity) {
        store.remove(savedEntity);
    }
}
exports.entityDeleted = entityDeleted;
