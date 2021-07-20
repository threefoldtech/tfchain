"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.nodeDeleted = exports.nodeUpdated = exports.nodeStored = void 0;
const model_1 = require("../../generated/graphql-server/model");
const types_1 = require("../generated/types");
const util_1 = require("./util");
async function nodeStored({ store, event, block, extrinsic, }) {
    const [node] = new types_1.TfgridModule.NodeStoredEvent(event).params;
    const newNode = new model_1.Node();
    newNode.gridVersion = node.version.toNumber();
    newNode.farmId = node.farm_id.toNumber();
    newNode.nodeId = node.id.toNumber();
    newNode.sru = node.resources.sru.toBn();
    newNode.hru = node.resources.hru.toBn();
    newNode.mru = node.resources.mru.toBn();
    newNode.cru = node.resources.cru.toBn();
    newNode.countryId = node.country_id.toNumber();
    newNode.cityId = node.city_id.toNumber();
    const newLocation = new model_1.Location();
    newLocation.latitude = util_1.hex2a(node.location.latitude.toString());
    newLocation.longitude = util_1.hex2a(node.location.longitude.toString());
    await store.save(newLocation);
    newNode.location = newLocation;
    if (node.public_config.isSome) {
        const pubConfig = new model_1.PublicConfig();
        const parsedConfig = node.public_config.unwrapOrDefault();
        pubConfig.ipv4 = util_1.hex2a(parsedConfig.ipv4.toString());
        pubConfig.ipv6 = util_1.hex2a(parsedConfig.ipv6.toString());
        pubConfig.gw4 = util_1.hex2a(parsedConfig.gw4.toString());
        pubConfig.gw6 = util_1.hex2a(parsedConfig.gw6.toString());
        await store.save(pubConfig);
        newNode.publicConfig = pubConfig;
    }
    newNode.twinId = node.twin_id.toNumber();
    await store.save(newNode);
}
exports.nodeStored = nodeStored;
// TODO
async function nodeUpdated({ store, event, block, extrinsic, }) {
    const [node] = new types_1.TfgridModule.NodeUpdatedEvent(event).params;
    const savedNode = await store.get(model_1.Node, { where: { nodeId: node.id.toNumber() } });
    if (!savedNode)
        return;
    savedNode.gridVersion = node.version.toNumber();
    savedNode.farmId = node.farm_id.toNumber();
    savedNode.nodeId = node.id.toNumber();
    savedNode.sru = node.resources.sru.toBn();
    savedNode.hru = node.resources.hru.toBn();
    savedNode.mru = node.resources.mru.toBn();
    savedNode.cru = node.resources.cru.toBn();
    savedNode.countryId = node.country_id.toNumber();
    savedNode.cityId = node.city_id.toNumber();
    const newLocation = new model_1.Location();
    newLocation.latitude = util_1.hex2a(node.location.latitude.toString());
    newLocation.longitude = util_1.hex2a(node.location.longitude.toString());
    await store.save(newLocation);
    savedNode.location = newLocation;
    if (node.public_config.isSome) {
        const pubConfig = new model_1.PublicConfig();
        const parsedConfig = node.public_config.unwrapOrDefault();
        pubConfig.ipv4 = util_1.hex2a(parsedConfig.ipv4.toString());
        pubConfig.ipv6 = util_1.hex2a(parsedConfig.ipv6.toString());
        pubConfig.gw4 = util_1.hex2a(parsedConfig.gw4.toString());
        pubConfig.gw6 = util_1.hex2a(parsedConfig.gw6.toString());
        await store.save(pubConfig);
        savedNode.publicConfig = pubConfig;
    }
    savedNode.twinId = node.twin_id.toNumber();
    await store.save(savedNode);
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
