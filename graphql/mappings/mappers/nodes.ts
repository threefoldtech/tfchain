import { Node, Location } from '../../generated/graphql-server/model'
import { TfgridModule } from '../generated/types'
import { hex2a } from './util'

import {
  EventContext,
  StoreContext,
} from '@subsquid/hydra-common'

export async function nodeStored({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const node = new Node()
  const [version, nodeID, farm_id, resources, location, countryID, cityID, address, role, twinID, publicConfig]  = new TfgridModule.NodeStoredEvent(event).params

  node.gridVersion = version.toNumber()
  node.farmId = farm_id.toNumber()
  node.nodeId = nodeID.toNumber()

  node.sru = resources.sru.toNumber()
  node.hru = resources.hru.toNumber()
  node.mru = resources.mru.toNumber()
  node.cru = resources.cru.toNumber()

  node.countryId = countryID.toNumber()
  node.cityId = cityID.toNumber()

  const newLocation = new Location()
  newLocation.latitude = hex2a(location.latitude.toString())
  newLocation.longitude = hex2a(location.longitude.toString())
  await store.save<Location>(newLocation)

  node.location = newLocation

  node.address = address.toHuman()

  node.role = role.toString()

  // todo twin id
  // todo public config

  await store.save<Node>(node)
}

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

export async function nodeDeleted({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const [nodeID] = new TfgridModule.NodeDeletedEvent(event).params

  const savedNode = await store.get(Node, { where: { nodeId: nodeID.toNumber() } })

  if (savedNode) {
    store.remove(savedNode)
  }
}