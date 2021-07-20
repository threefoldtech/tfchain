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
  const [node]  = new TfgridModule.NodeStoredEvent(event).params
  const newNode = new Node()

  newNode.gridVersion = node.version.toNumber()
  newNode.farmId = node.farm_id.toNumber()
  newNode.nodeId = node.id.toNumber()

  newNode.sru = node.resources.sru.toBn()
  newNode.hru = node.resources.hru.toBn()
  newNode.mru = node.resources.mru.toBn()
  newNode.cru = node.resources.cru.toBn()

  newNode.countryId = node.country_id.toNumber()
  newNode.cityId = node.city_id.toNumber()

  const newLocation = new Location()
  newLocation.latitude = hex2a(node.location.latitude.toString())
  newLocation.longitude = hex2a(node.location.longitude.toString())
  await store.save<Location>(newLocation)

  newNode.location = newLocation
  
  if (node.public_config.isSome) {
    const pubConfig = new PublicConfig()
    const parsedConfig = node.public_config.unwrapOrDefault()
    pubConfig.ipv4 = hex2a(parsedConfig.ipv4.toString())
    pubConfig.ipv6 = hex2a(parsedConfig.ipv6.toString())
    pubConfig.gw4 = hex2a(parsedConfig.gw4.toString())
    pubConfig.gw6 = hex2a(parsedConfig.gw6.toString())

    await store.save<PublicConfig>(pubConfig)
    newNode.publicConfig = pubConfig
  }

  newNode.twinId = node.twin_id.toNumber()

  await store.save<Node>(newNode)
}

// TODO

export async function nodeUpdated({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const [node]  = new TfgridModule.NodeUpdatedEvent(event).params

  const savedNode = await store.get(Node, { where: { nodeId: node.id.toNumber() } })
  if (!savedNode) return

  savedNode.gridVersion = node.version.toNumber()
  savedNode.farmId = node.farm_id.toNumber()
  savedNode.nodeId = node.id.toNumber()

  savedNode.sru = node.resources.sru.toBn()
  savedNode.hru = node.resources.hru.toBn()
  savedNode.mru = node.resources.mru.toBn()
  savedNode.cru = node.resources.cru.toBn()

  savedNode.countryId = node.country_id.toNumber()
  savedNode.cityId = node.city_id.toNumber()

  const newLocation = new Location()
  newLocation.latitude = hex2a(node.location.latitude.toString())
  newLocation.longitude = hex2a(node.location.longitude.toString())
  await store.save<Location>(newLocation)

  savedNode.location = newLocation
  
  if (node.public_config.isSome) {
    const pubConfig = new PublicConfig()
    const parsedConfig = node.public_config.unwrapOrDefault()
    pubConfig.ipv4 = hex2a(parsedConfig.ipv4.toString())
    pubConfig.ipv6 = hex2a(parsedConfig.ipv6.toString())
    pubConfig.gw4 = hex2a(parsedConfig.gw4.toString())
    pubConfig.gw6 = hex2a(parsedConfig.gw6.toString())

    await store.save<PublicConfig>(pubConfig)
    savedNode.publicConfig = pubConfig
  }

  savedNode.twinId = node.twin_id.toNumber()

  await store.save<Node>(savedNode)
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