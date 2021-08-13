"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.farmDeleted = exports.farmUpdated = exports.farmStored = void 0;
const model_1 = require("../../generated/graphql-server/model");
const types_1 = require("../generated/types");
const util_1 = require("./util");
async function farmStored({ store, event, block, extrinsic, }) {
    const [farm] = new types_1.TfgridModule.FarmStoredEvent(event).params;
    const newFarm = new model_1.Farm();
    newFarm.gridVersion = farm.version.toNumber();
    newFarm.farmId = farm.id.toNumber();
    newFarm.name = util_1.hex2a(Buffer.from(farm.name.toString()).toString());
    newFarm.twinId = farm.twin_id.toNumber();
    newFarm.pricingPolicyId = farm.pricing_policy_id.toNumber();
    newFarm.countryId = farm.country_id.toNumber();
    newFarm.cityId = farm.city_id.toNumber();
    const certificationTypeAsString = farm.certification_type.toString();
    let certType = model_1.CertificationType.Diy;
    switch (certificationTypeAsString) {
        case 'Diy': certType = model_1.CertificationType.Diy;
        case 'Diy': certType = model_1.CertificationType.Certified;
    }
    newFarm.certificationType = certType;
    await store.save(newFarm);
    const ipPromises = farm.public_ips.map(ip => {
        const newIP = new model_1.PublicIp();
        newIP.ip = util_1.hex2a(Buffer.from(ip.ip.toString()).toString());
        newIP.gateway = util_1.hex2a(Buffer.from(ip.gateway.toString()).toString());
        newIP.contractId = ip.contract_id.toNumber();
        newIP.farm = newFarm;
        return store.save(newIP);
    });
    await Promise.all(ipPromises);
}
exports.farmStored = farmStored;
async function farmUpdated({ store, event, block, extrinsic, }) {
    const [farm] = new types_1.TfgridModule.FarmUpdatedEvent(event).params;
    const savedFarm = await store.get(model_1.Farm, { where: { farmId: farm.id.toNumber() } });
    if (savedFarm) {
        savedFarm.gridVersion = farm.version.toNumber();
        // savedFarm.farmId = farm.id.toNumber()
        savedFarm.name = util_1.hex2a(Buffer.from(farm.name.toString()).toString());
        savedFarm.twinId = farm.twin_id.toNumber();
        savedFarm.pricingPolicyId = farm.pricing_policy_id.toNumber();
        savedFarm.countryId = farm.country_id.toNumber();
        savedFarm.cityId = farm.city_id.toNumber();
        const certificationTypeAsString = farm.certification_type.toString();
        let certType = model_1.CertificationType.Diy;
        switch (certificationTypeAsString) {
            case 'Diy': certType = model_1.CertificationType.Diy;
            case 'Diy': certType = model_1.CertificationType.Certified;
        }
        savedFarm.certificationType = certType;
        const ipPromises = farm.public_ips.map(ip => {
            const newIP = new model_1.PublicIp();
            newIP.ip = util_1.hex2a(Buffer.from(ip.ip.toString()).toString());
            newIP.gateway = util_1.hex2a(Buffer.from(ip.gateway.toString()).toString());
            newIP.contractId = ip.contract_id.toNumber();
            newIP.farm = savedFarm;
            return store.save(newIP);
        });
        await Promise.all(ipPromises);
    }
}
exports.farmUpdated = farmUpdated;
async function farmDeleted({ store, event, block, extrinsic, }) {
    const [farmID] = new types_1.TfgridModule.FarmDeletedEvent(event).params;
    const savedFarm = await store.get(model_1.Farm, { where: { farmId: farmID.toNumber() } });
    if (savedFarm) {
        store.remove(savedFarm);
    }
}
exports.farmDeleted = farmDeleted;
