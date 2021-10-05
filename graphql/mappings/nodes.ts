import { Node, Location, PublicConfig, UptimeEvent, Interfaces } from '../generated/model'
import { TfgridModule } from '../chain'
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

  newNode.country = hex2a(node.country.toString())
  newNode.city = hex2a(node.city.toString())

  newNode.created = node.created.toNumber()
  newNode.uptime = node.uptime.toNumber()
  newNode.farmingPolicyId = node.farming_policy_id.toNumber()

  const newLocation = new Location()
  newLocation.latitude = hex2a(node.location.latitude.toString())
  newLocation.longitude = hex2a(node.location.longitude.toString())
  await store.save<Location>(newLocation)

  newNode.location = newLocation
  
  if (node.public_config.isSome) {
    const pubConfig = new PublicConfig()
    const parsedConfig = node.public_config.unwrapOrDefault()
    console.log(parsedConfig)
    pubConfig.ipv4 = hex2a(parsedConfig.ipv4.toString())
    pubConfig.ipv6 = hex2a(parsedConfig.ipv6.toString())
    pubConfig.gw4 = hex2a(parsedConfig.gw4.toString())
    pubConfig.gw6 = hex2a(parsedConfig.gw6.toString())
    pubConfig.domain = hex2a(parsedConfig.domain.toString()) || ''

    await store.save<PublicConfig>(pubConfig)
    newNode.publicConfig = pubConfig
  }

  newNode.twinId = node.twin_id.toNumber()

  await store.save<Node>(newNode)

  const interfacesPromisses = node.interfaces.map(intf => {
    const newInterface = new Interfaces()

    newInterface.name = hex2a(Buffer.from(intf.name.toString()).toString())
    newInterface.mac = hex2a(Buffer.from(intf.mac.toString()).toString())
    newInterface.node = newNode
    newInterface.ips = intf.ips.map(ip => hex2a(Buffer.from(ip.toString()).toString())).join(',')
    return store.save<Interfaces>(newInterface)
  })
  await Promise.all(interfacesPromisses)
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

  savedNode.country = hex2a(node.country.toString())
  savedNode.city = hex2a(node.city.toString())

  savedNode.created = node.created.toNumber()
  savedNode.uptime = node.uptime.toNumber()
  savedNode.farmingPolicyId = node.farming_policy_id.toNumber()

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
    pubConfig.domain = hex2a(parsedConfig.domain.toString()) || ''

    await store.save<PublicConfig>(pubConfig)
    savedNode.publicConfig = pubConfig
  }

  savedNode.twinId = node.twin_id.toNumber()

  await store.save<Node>(savedNode)

  const interfacesPromisses = node.interfaces.map(intf => {
    const newInterface = new Interfaces()

    newInterface.name = hex2a(Buffer.from(intf.name.toString()).toString())
    newInterface.mac = hex2a(Buffer.from(intf.mac.toString()).toString())
    newInterface.node = savedNode
    newInterface.ips = intf.ips.map(ip => hex2a(Buffer.from(ip.toString()).toString())).join(',')
    return store.save<Interfaces>(newInterface)
  })
  await Promise.all(interfacesPromisses)
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

export async function nodeUptimeReported({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const [nodeID, now, uptime] = new TfgridModule.NodeUptimeReportedEvent(event).params

  const newUptimeEvent = new UptimeEvent()
  newUptimeEvent.nodeId = nodeID.toNumber()
  newUptimeEvent.timestamp = now.toNumber()
  newUptimeEvent.uptime = uptime.toNumber()
  await store.save<UptimeEvent>(newUptimeEvent)

  const savedNode = await store.get(Node, { where: { nodeId: nodeID.toNumber() } })

  if (savedNode) {
    savedNode.uptime = uptime.toNumber()
    await store.save<Node>(savedNode)
  }
}