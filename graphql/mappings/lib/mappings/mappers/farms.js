"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.farmDeleted = exports.farmStored = void 0;
const model_1 = require("../../generated/graphql-server/model");
const types_1 = require("../generated/types");
const util_1 = require("./util");
async function farmStored({ store, event, block, extrinsic, }) {
    const [version, farm_id, name, twin_id, pricing_policy_id, country_id, city_id, certificationType] = new types_1.TfgridModule.FarmStoredEvent(event).params;
    const farm = new model_1.Farm();
    farm.gridVersion = version.toNumber();
    farm.farmId = farm_id.toNumber();
    farm.name = util_1.hex2a(Buffer.from(name.toString()).toString());
    farm.twinId = twin_id.toNumber();
    farm.pricingPolicyId = pricing_policy_id.toNumber();
    farm.countryId = country_id.toNumber();
    farm.cityId = city_id.toNumber();
    const certificationTypeAsString = certificationType.toString();
    let certType = model_1.CertificationType.None;
    switch (certificationTypeAsString) {
        case 'Gold': certType = model_1.CertificationType.Gold;
        case 'Silver': certType = model_1.CertificationType.Silver;
    }
    farm.certificationType = certType;
    await store.save(farm);
}
exports.farmStored = farmStored;
async function farmDeleted({ store, event, block, extrinsic, }) {
    const [farmID] = new types_1.TfgridModule.FarmDeletedEvent(event).params;
    const savedFarm = await store.get(model_1.Farm, { where: { farmId: farmID.toNumber() } });
    if (savedFarm) {
        store.remove(savedFarm);
    }
}
exports.farmDeleted = farmDeleted;
