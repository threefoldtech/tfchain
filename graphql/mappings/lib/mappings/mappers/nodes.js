"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.nodeDeleted = exports.nodeUpdated = exports.nodeStored = void 0;
const model_1 = require("../../generated/graphql-server/model");
const types_1 = require("../generated/types");
const util_1 = require("./util");
async function nodeStored({ store, event, block, extrinsic, }) {
    const node = new model_1.Node();
    const [version, nodeID, farm_id, resources, location, countryID, cityID, address, role, twinID, publicConfig] = new types_1.TfgridModule.NodeStoredEvent(event).params;
    node.gridVersion = version.toNumber();
    node.farmId = farm_id.toNumber();
    node.nodeId = nodeID.toNumber();
    node.sru = resources.sru.toBn();
    node.hru = resources.hru.toBn();
    node.mru = resources.mru.toBn();
    node.cru = resources.cru.toBn();
    node.countryId = countryID.toNumber();
    node.cityId = cityID.toNumber();
    const newLocation = new model_1.Location();
    newLocation.latitude = util_1.hex2a(location.latitude.toString());
    newLocation.longitude = util_1.hex2a(location.longitude.toString());
    await store.save(newLocation);
    node.location = newLocation;
    node.address = address.toString();
    node.role = role.toString();
    if (publicConfig.isSome) {
        const pubConfig = new model_1.PublicConfig();
        const parsedConfig = publicConfig.unwrapOrDefault();
        pubConfig.ipv4 = util_1.hex2a(parsedConfig.ipv4.toString());
        pubConfig.ipv6 = util_1.hex2a(parsedConfig.ipv6.toString());
        pubConfig.gw4 = util_1.hex2a(parsedConfig.gw4.toString());
        pubConfig.gw6 = util_1.hex2a(parsedConfig.gw6.toString());
        await store.save(pubConfig);
        node.publicConfig = pubConfig;
    }
    node.twinId = twinID.toNumber();
    await store.save(node);
}
exports.nodeStored = nodeStored;
// TODO
async function nodeUpdated({ store, event, block, extrinsic, }) {
    const node = new model_1.Node();
    const [version, nodeID, farm_id, resources, location, countryID, cityID, address, role, twinID, publicConfig] = new types_1.TfgridModule.NodeUpdatedEvent(event).params;
    const savedNode = await store.get(model_1.Node, { where: { nodeId: nodeID.toNumber() } });
    if (!savedNode)
        return;
    node.gridVersion = version.toNumber();
    node.farmId = farm_id.toNumber();
    node.nodeId = nodeID.toNumber();
    node.sru = resources.sru.toBn();
    node.hru = resources.hru.toBn();
    node.mru = resources.mru.toBn();
    node.cru = resources.cru.toBn();
    node.countryId = countryID.toNumber();
    node.cityId = cityID.toNumber();
    node.location.latitude = util_1.hex2a(location.latitude.toString());
    node.location.longitude = util_1.hex2a(location.longitude.toString());
    node.address = util_1.hex2a(address.toString());
    if (publicConfig.isSome) {
        const pubConfig = new model_1.PublicConfig();
        const parsedConfig = publicConfig.unwrapOrDefault();
        pubConfig.ipv4 = util_1.hex2a(parsedConfig.ipv4.toString());
        pubConfig.ipv6 = util_1.hex2a(parsedConfig.ipv6.toString());
        pubConfig.gw4 = util_1.hex2a(parsedConfig.gw4.toString());
        pubConfig.gw6 = util_1.hex2a(parsedConfig.gw6.toString());
        await store.save(pubConfig);
        node.publicConfig = pubConfig;
    }
    node.role = role.toString();
    node.twinId = twinID.toNumber();
    await store.save(node);
}
exports.nodeUpdated = nodeUpdated;
async function nodeDeleted({ store, event, block, extrinsic, }) {
    const [nodeID] = new types_1.TfgridModule.NodeDeletedEvent(event).params;
    const savedNode = await store.get(model_1.Node, { where: { nodeId: nodeID.toNumber() } });
    if (savedNode) {
        store.remove(savedNode);
    }
}
exports.nodeDeleted = nodeDeleted;
