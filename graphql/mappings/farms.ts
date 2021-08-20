import { Farm, CertificationType, PublicIp } from '../generated/model'
import { TfgridModule } from '../types'
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
    case 'Diy': certType = CertificationType.Diy
    case 'Diy': certType = CertificationType.Certified
  }
  newFarm.certificationType = certType

  await store.save<Farm>(newFarm)

  const ipPromises = farm.public_ips.map(ip => {
    const newIP = new PublicIp()

    newIP.ip = hex2a(Buffer.from(ip.ip.toString()).toString())
    newIP.gateway = hex2a(Buffer.from(ip.gateway.toString()).toString())
    newIP.contractId = ip.contract_id.toNumber()
    newIP.farm = newFarm

    return store.save<PublicIp>(newIP)
  })
  await Promise.all(ipPromises)
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

    const certificationTypeAsString = farm.certification_type.toString()
    let certType = CertificationType.Diy
    switch (certificationTypeAsString) {
      case 'Diy': certType = CertificationType.Diy
      case 'Diy': certType = CertificationType.Certified
    }

    savedFarm.certificationType = certType

    const ipPromises = farm.public_ips.map(ip => {
      const newIP = new PublicIp()
  
      newIP.ip = hex2a(Buffer.from(ip.ip.toString()).toString())
      newIP.gateway = hex2a(Buffer.from(ip.gateway.toString()).toString())
      newIP.contractId = ip.contract_id.toNumber()
      newIP.farm = savedFarm
  
      return store.save<PublicIp>(newIP)
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