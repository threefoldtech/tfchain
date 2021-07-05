"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.entityDeleted = exports.entityUpdated = exports.entityStored = void 0;
const model_1 = require("../../generated/graphql-server/model");
const types_1 = require("../generated/types");
const util_1 = require("./util");
async function entityStored({ store, event, block, extrinsic, }) {
    const entity = new model_1.Entity();
    const [version, eneityID, name, countryID, cityID, accountID] = new types_1.TfgridModule.EntityStoredEvent(event).params;
    entity.gridVersion = version.toNumber();
    entity.entityId = eneityID.toNumber();
    entity.name = util_1.hex2a(name.toString());
    entity.countryId = countryID.toNumber();
    entity.cityId = cityID.toNumber();
    entity.address = accountID.toHuman();
    await store.save(entity);
}
exports.entityStored = entityStored;
async function entityUpdated({ store, event, block, extrinsic, }) {
    const entity = new model_1.Entity();
    const [eneityID, name, countryID, cityID, accountID] = new types_1.TfgridModule.EntityUpdatedEvent(event).params;
    const savedEntity = await store.get(model_1.Entity, { where: { entityId: eneityID.toNumber() } });
    if (savedEntity) {
        // entity.gridVersion = version.toNumber()
        savedEntity.entityId = eneityID.toNumber();
        savedEntity.name = util_1.hex2a(name.toString());
        savedEntity.countryId = countryID.toNumber();
        savedEntity.cityId = cityID.toNumber();
        savedEntity.address = accountID.toHuman();
        await store.save(savedEntity);
    }
}
exports.entityUpdated = entityUpdated;
async function entityDeleted({ store, event, block, extrinsic, }) {
    const entity = new model_1.Entity();
    const [eneityID] = new types_1.TfgridModule.EntityDeletedEvent(event).params;
    const savedEntity = await store.get(model_1.Entity, { where: { entityId: eneityID.toNumber() } });
    if (savedEntity) {
        store.remove(savedEntity);
    }
}
exports.entityDeleted = entityDeleted;
