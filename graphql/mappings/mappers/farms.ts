import { Farm, CertificationType, PublicIp } from '../../generated/graphql-server/model'
import { TfgridModule } from '../generated/types'
import { hex2a } from './util'

import {
  EventContext,
  StoreContext,
} from '@subsquid/hydra-common'

export async function farmStored({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const [farm]  = new TfgridModule.FarmStoredEvent(event).params

  const newFarm = new Farm()
  
  newFarm.gridVersion = farm.version.toNumber()
  newFarm.farmId = farm.id.toNumber()
  newFarm.name = hex2a(Buffer.from(farm.name.toString()).toString())
  newFarm.twinId = farm.twin_id.toNumber()
  newFarm.pricingPolicyId = farm.pricing_policy_id.toNumber()
  newFarm.countryId = farm.country_id.toNumber()
  newFarm.cityId = farm.city_id.toNumber()

  const certificationTypeAsString = farm.certification_type.toString()
  let certType = CertificationType.None
  switch (certificationTypeAsString) {
    case 'Gold': certType = CertificationType.Gold
    case 'Silver': certType = CertificationType.Silver
  }
  newFarm.certificationType = certType

  await store.save<Farm>(newFarm)
  
  const publicIps: PublicIp[] = []
  
  farm.public_ips.forEach(async ip => {
    const newIP = new PublicIp()

    newIP.ip = hex2a(Buffer.from(ip.ip.toString()).toString())
    newIP.gateway = hex2a(Buffer.from(ip.gateway.toString()).toString())
    newIP.contractId = ip.contract_id.toNumber()
    newIP.farm = newFarm

    await store.save<PublicIp>(newIP)

    publicIps.push(newIP)
  })

  newFarm.publicIPs = publicIps

  await store.save<Farm>(newFarm)
}

export async function farmUpdated({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const [farm]  = new TfgridModule.FarmUpdatedEvent(event).params
  
  const savedFarm = await store.get(Farm, { where: { farmId: farm.id.toNumber() } })
  
  if (savedFarm) {
    savedFarm.gridVersion = farm.version.toNumber()
    savedFarm.farmId = farm.id.toNumber()
    savedFarm.name = hex2a(Buffer.from(farm.name.toString()).toString())
    savedFarm.twinId = farm.twin_id.toNumber()
    savedFarm.pricingPolicyId = farm.pricing_policy_id.toNumber()
    savedFarm.countryId = farm.country_id.toNumber()
    savedFarm.cityId = farm.city_id.toNumber()
    const certificationTypeAsString = farm.certification_type.toString()
    let certType = CertificationType.None
    switch (certificationTypeAsString) {
      case 'Gold': certType = CertificationType.Gold
      case 'Silver': certType = CertificationType.Silver
    }
    savedFarm.certificationType = certType

    const publicIps: PublicIp[] = []
  
    farm.public_ips.forEach(async ip => {
      const newIP = new PublicIp()

      newIP.ip = hex2a(Buffer.from(ip.ip.toString()).toString())
      newIP.gateway = hex2a(Buffer.from(ip.gateway.toString()).toString())
      newIP.contractId = ip.contract_id.toNumber()
      newIP.farm = savedFarm

      await store.save<PublicIp>(newIP)

      publicIps.push(newIP)
    })

    savedFarm.publicIPs = publicIps
  
    await store.save<Farm>(savedFarm)
  }
}

export async function farmDeleted({
    store,
    event,
    block,
    extrinsic,
  }: EventContext & StoreContext) {
    const [farmID] = new TfgridModule.FarmDeletedEvent(event).params
  
    const savedFarm = await store.get(Farm, { where: { farmId: farmID.toNumber() } })
  
    if (savedFarm) {
      store.remove(savedFarm)
    }
  }