"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.farmingPolicyStored = exports.pricingPolicyStored = void 0;
const model_1 = require("../../generated/graphql-server/model");
const types_1 = require("../generated/types");
const util_1 = require("./util");
async function pricingPolicyStored({ store, event, block, extrinsic, }) {
    const newPricingPolicy = new model_1.PricingPolicy();
    const [pricing_policy] = new types_1.TfgridModule.PricingPolicyStoredEvent(event).params;
    newPricingPolicy.gridVersion = pricing_policy.version.toNumber();
    newPricingPolicy.pricingPolicyId = pricing_policy.id.toNumber();
    newPricingPolicy.name = util_1.hex2a(Buffer.from(pricing_policy.name.toString()).toString());
    const suPolicy = new model_1.Policy();
    suPolicy.value = pricing_policy.su.value.toNumber();
    suPolicy.unit = formatUnit(pricing_policy.su.unit.toString());
    await store.save(suPolicy);
    const nuPolicy = new model_1.Policy();
    nuPolicy.value = pricing_policy.nu.value.toNumber();
    nuPolicy.unit = formatUnit(pricing_policy.nu.unit.toString());
    await store.save(nuPolicy);
    const cuPolicy = new model_1.Policy();
    cuPolicy.value = pricing_policy.cu.value.toNumber();
    cuPolicy.unit = formatUnit(pricing_policy.cu.unit.toString());
    await store.save(cuPolicy);
    const IpuPolicy = new model_1.Policy();
    IpuPolicy.value = pricing_policy.ipu.value.toNumber();
    IpuPolicy.unit = formatUnit(pricing_policy.ipu.unit.toString());
    await store.save(IpuPolicy);
    newPricingPolicy.su = suPolicy;
    newPricingPolicy.cu = cuPolicy;
    newPricingPolicy.nu = nuPolicy;
    newPricingPolicy.ipu = IpuPolicy;
    newPricingPolicy.foundationAccount = Buffer.from(pricing_policy.foundation_account.toHex()).toString();
    newPricingPolicy.certifiedSalesAccount = Buffer.from(pricing_policy.certified_sales_account.toHex()).toString();
    await store.save(newPricingPolicy);
}
exports.pricingPolicyStored = pricingPolicyStored;
async function farmingPolicyStored({ store, event, block, extrinsic, }) {
    const newFarmingPolicy = new model_1.FarmingPolicy();
    const [farming_policy] = new types_1.TfgridModule.FarmingPolicyStoredEvent(event).params;
    newFarmingPolicy.gridVersion = farming_policy.version.toNumber();
    newFarmingPolicy.farmingPolicyId = farming_policy.id.toNumber();
    newFarmingPolicy.name = util_1.hex2a(Buffer.from(farming_policy.name.toString()).toString());
    newFarmingPolicy.cu = farming_policy.cu.toNumber();
    newFarmingPolicy.su = farming_policy.su.toNumber();
    newFarmingPolicy.nu = farming_policy.nu.toNumber();
    newFarmingPolicy.ipv4 = farming_policy.ipv4.toNumber();
    newFarmingPolicy.timestamp = farming_policy.timestamp.toNumber();
    const certificationTypeAsString = farming_policy.certification_type.toString();
    let certType = model_1.CertificationType.Diy;
    switch (certificationTypeAsString) {
        case 'Diy': certType = model_1.CertificationType.Diy;
        case 'Diy': certType = model_1.CertificationType.Certified;
    }
    newFarmingPolicy.certificationType = certType;
    await store.save(newFarmingPolicy);
}
exports.farmingPolicyStored = farmingPolicyStored;
function formatUnit(unitAsString) {
    switch (unitAsString) {
        case 'Kilobytes': return model_1.Unit.Kilobytes;
        case 'Megabytes': return model_1.Unit.Megabytes;
        case 'Gigabytes': return model_1.Unit.Gigabytes;
        case 'Terrabytes': return model_1.Unit.Terrabytes;
        default: return model_1.Unit.Bytes;
    }
}
