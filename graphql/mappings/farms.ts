import { Farm, CertificationType, PublicIp } from '../generated/model'
import { TfgridModule } from '../chain'
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

  const certificationTypeAsString = farm.certification_type.toString()
  let certType = CertificationType.Diy
  switch (certificationTypeAsString) {
    case 'Diy': 
      certType = CertificationType.Diy
      break
    case 'Certified': 
      certType = CertificationType.Certified
      break
  }

  newFarm.certificationType = certType

  await store.save<Farm>(newFarm)

  const ipPromises = farm.public_ips.map(ip => {
    const newIP = new PublicIp()

    newIP.ip = hex2a(Buffer.from(ip.ip.toString()).toString())
    newIP.gateway = hex2a(Buffer.from(ip.gateway.toString()).toString())
    newIP.contractId = ip.contract_id.toNumber()
    newIP.farm = newFarm

    newFarm.publicIPs?.push(newIP)

    return store.save<PublicIp>(newIP)
  })
  await Promise.all(ipPromises)
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
    // savedFarm.farmId = farm.id.toNumber()
    savedFarm.name = hex2a(Buffer.from(farm.name.toString()).toString())
    savedFarm.twinId = farm.twin_id.toNumber()
    savedFarm.pricingPolicyId = farm.pricing_policy_id.toNumber()

    const ipPromises = farm.public_ips.map(async ip => {
      const newIP = new PublicIp()

      const savedIP = await store.get(PublicIp, { where: { ip: hex2a(Buffer.from(ip.ip.toString()).toString()) }})
      // ip is already there in storage, don't save it again
      if (savedIP) return
  
      newIP.ip = hex2a(Buffer.from(ip.ip.toString()).toString())
      newIP.gateway = hex2a(Buffer.from(ip.gateway.toString()).toString())
      newIP.contractId = ip.contract_id.toNumber()
      newIP.farm = savedFarm
  
      return await store.save<PublicIp>(newIP)
    })

    await Promise.all(ipPromises)
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

export async function farmPayoutV2AddressRegistered({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const [farmID, stellarAddress] = new TfgridModule.FarmPayoutV2AddressRegisteredEvent(event).params

  const savedFarm = await store.get(Farm, { where: { farmId: farmID.toNumber() } })

  if (savedFarm) {
    savedFarm.stellarAddress = hex2a(Buffer.from(stellarAddress.toString()).toString())
    await store.save<Farm>(savedFarm)
  }
}