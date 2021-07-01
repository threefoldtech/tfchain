"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.nodeDeleted = exports.nodeStored = void 0;
const model_1 = require("../../generated/graphql-server/model");
const types_1 = require("../generated/types");
const util_1 = require("./util");
async function nodeStored({ store, event, block, extrinsic, }) {
    const node = new model_1.Node();
    const [version, nodeID, farm_id, resources, location, countryID, cityID, address, role, twinID, publicConfig] = new types_1.TfgridModule.NodeStoredEvent(event).params;
    node.gridVersion = version.toNumber();
    node.farmId = farm_id.toNumber();
    node.nodeId = nodeID.toNumber();
    node.sru = resources.sru.toNumber();
    node.hru = resources.hru.toNumber();
    node.mru = resources.mru.toNumber();
    node.cru = resources.cru.toNumber();
    node.countryId = countryID.toNumber();
    node.cityId = cityID.toNumber();
    node.location.latitude = util_1.hex2a(location.latitude.toString());
    node.location.longitude = util_1.hex2a(location.longitude.toString());
    node.address = util_1.hex2a(address.toString());
    node.role = role.toString();
    // todo twin id
    // todo public config
    await store.save(node);
}
exports.nodeStored = nodeStored;
// TODO
// export async function nodeUpdated({
//   store,
//   event,
//   block,
//   extrinsic,
// }: EventContext & StoreContext) {
//   const node = new Node()
//   const [version, nodeID, farm_id, resources, location, countryID, cityID, address, role, x, publicConfig]  = new TfgridModule.NodeUpdatedEvent(event).params
//   const savedNode = await store.get(Node, { where: { nodeId: nodeID.toNumber() } })
//   if (!savedNode) return
//   node.gridVersion = version.toNumber()
//   node.farmId = farm_id.toNumber()
//   node.nodeId = nodeID.toNumber()
//   node.sru = resources.sru.toNumber()
//   node.hru = resources.hru.toNumber()
//   node.mru = resources.mru.toNumber()
//   node.cru = resources.cru.toNumber()
//   node.countryId = countryID.toNumber()
//   node.cityId = cityID.toNumber()
//   node.location.latitude = hex2a(location.latitude.toString())
//   node.location.longitude = hex2a(location.longitude.toString())
//   node.address = hex2a(address.toString())
//   node.role = role.toString()
//   await store.save<Node>(node)
// }
async function nodeDeleted({ store, event, block, extrinsic, }) {
    const [nodeID] = new types_1.TfgridModule.NodeDeletedEvent(event).params;
    const savedNode = await store.get(model_1.Node, { where: { nodeId: nodeID.toNumber() } });
    if (savedNode) {
        store.remove(savedNode);
    }
}
exports.nodeDeleted = nodeDeleted;
