import { SubstrateEvent, DB } from '../generated/hydra-processor'
import { Node } from '../generated/graphql-server/src/modules/node/node.model'
import { Resource } from '../generated/graphql-server/src/modules/resource/resource.model'
import { Location } from '../generated/graphql-server/src/modules/location/location.model'
import { hex2a } from './util'

import BN from 'bn.js'

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
  const [node_id, farm_id, twin_id, resources, location, country_id, city_id] = event.params
  const node = new Node()
  
  node.nodeId = new BN(node_id.value as number)
  node.twinId = new BN(twin_id.value as number)
  node.farmId = new BN(farm_id.value as number)

  const parsedResource = (resources.value as unknown) as IResource

  const resource = new Resource()
  resource.sru = new BN(parsedResource.sru)
  resource.cru = new BN(parsedResource.cru)
  resource.mru = new BN(parsedResource.mru)
  resource.hru = new BN(parsedResource.hru)
  await db.save<Resource>(resource)

  node.resources = resource

  const parsedLocation = (location.value as unknown) as ILocation
  
  if (parsedLocation) {
    const loc = new Location()
    loc.latitude = hex2a(parsedLocation.latitude)
    loc.longitude = hex2a(parsedLocation.longitude)
    node.location = loc
    await db.save<Location>(loc)
  }
  
  node.countryId = new BN(country_id.value as number)
  node.cityId = new BN(city_id.value as number)

  await db.save<Node>(node)
}

export async function tfgridModule_NodeDeleted(db: DB, event: SubstrateEvent) {
    const [node_id] = event.params
  
    const savedNode = await db.get(Node, { where: { farmId: new BN(node_id.value as number) } })
  
    if (savedNode) {
      await db.remove(savedNode)
    }
  }