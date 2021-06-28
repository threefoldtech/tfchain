import { SubstrateEvent, DB } from '../generated/hydra-processor'
import { Node } from '../generated/graphql-server/src/modules/node/node.model'
import { Location } from '../generated/graphql-server/src/modules/location/location.model'

import { hex2a } from './util'

interface IResource {
  hru: number
  sru: number
  cru: number
  mru: number
}

interface ILocation {
  latitude: string
  longitude: string
}

export async function tfgridModule_NodeStored(db: DB, event: SubstrateEvent) {
  const [version, node_id, farm_id, resources, location, country_id, city_id, pub_key, address, role] = event.params
  const node = new Node()
  
  node.gridVersion = version.value as number
  node.nodeId = node_id.value as number
  node.farmId = farm_id.value as number

  // skip parsed node resource if node id is 1
  // node with id 1 is on accident stored node with old data model
  if (node.nodeId === 1) return

  const parsedResource = (resources.value as unknown) as IResource

  node.sru = parsedResource.sru
  node.cru = parsedResource.cru
  node.mru = parsedResource.mru
  node.hru = parsedResource.hru

  const parsedLocation = (location.value as unknown) as ILocation
  
  if (parsedLocation) {
    const loc = new Location()
    loc.latitude = hex2a(parsedLocation.latitude)
    loc.longitude = hex2a(parsedLocation.longitude)
    node.location = loc
    await db.save<Location>(loc)
  }
  
  node.countryId = country_id.value as number
  node.cityId = city_id.value as number
  node.pubKey = hex2a(Buffer.from(pub_key.value as string).toString())
  node.address = Buffer.from(address.value as string).toString()

  node.role = Buffer.from(role.value as string).toString()

  await db.save<Node>(node)
}

export async function tfgridModule_NodeDeleted(db: DB, event: SubstrateEvent) {
  const [node_id] = event.params

  const savedNode = await db.get(Node, { where: { nodeId: node_id.value as number } })

  if (savedNode) {
    await db.remove(savedNode)
  }
}