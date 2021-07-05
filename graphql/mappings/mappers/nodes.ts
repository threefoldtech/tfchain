import { Node, Location, PublicConfig } from '../../generated/graphql-server/model'
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

  node.sru = resources.sru.toBn()
  node.hru = resources.hru.toBn()
  node.mru = resources.mru.toBn()
  node.cru = resources.cru.toBn()

  node.countryId = countryID.toNumber()
  node.cityId = cityID.toNumber()

  const newLocation = new Location()
  newLocation.latitude = hex2a(location.latitude.toString())
  newLocation.longitude = hex2a(location.longitude.toString())
  await store.save<Location>(newLocation)

  node.location = newLocation

  node.address = address.toString()

  node.role = role.toString()
  
  if (publicConfig.isSome) {
    const pubConfig = new PublicConfig()
    const parsedConfig = publicConfig.unwrapOrDefault()
    pubConfig.ipv4 = hex2a(parsedConfig.ipv4.toString())
    pubConfig.ipv6 = hex2a(parsedConfig.ipv6.toString())
    pubConfig.gw4 = hex2a(parsedConfig.gw4.toString())
    pubConfig.gw6 = hex2a(parsedConfig.gw6.toString())

    await store.save<PublicConfig>(pubConfig)
    node.publicConfig = pubConfig
  }

  node.twinId = twinID.toNumber()

  await store.save<Node>(node)
}

// TODO

export async function nodeUpdated({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const node = new Node()
  const [version, nodeID, farm_id, resources, location, countryID, cityID, address, role, twinID, publicConfig]  = new TfgridModule.NodeUpdatedEvent(event).params

  const savedNode = await store.get(Node, { where: { nodeId: nodeID.toNumber() } })
  if (!savedNode) return

  node.gridVersion = version.toNumber()
  node.farmId = farm_id.toNumber()
  node.nodeId = nodeID.toNumber()

  node.sru = resources.sru.toBn()
  node.hru = resources.hru.toBn()
  node.mru = resources.mru.toBn()
  node.cru = resources.cru.toBn()

  node.countryId = countryID.toNumber()
  node.cityId = cityID.toNumber()

  node.location.latitude = hex2a(location.latitude.toString())
  node.location.longitude = hex2a(location.longitude.toString())

  node.address = hex2a(address.toString())

  if (publicConfig.isSome) {
    const pubConfig = new PublicConfig()
    const parsedConfig = publicConfig.unwrapOrDefault()
    pubConfig.ipv4 = hex2a(parsedConfig.ipv4.toString())
    pubConfig.ipv6 = hex2a(parsedConfig.ipv6.toString())
    pubConfig.gw4 = hex2a(parsedConfig.gw4.toString())
    pubConfig.gw6 = hex2a(parsedConfig.gw6.toString())

    await store.save<PublicConfig>(pubConfig)
    node.publicConfig = pubConfig
  }

  node.role = role.toString()
  node.twinId = twinID.toNumber()

  await store.save<Node>(node)
}

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