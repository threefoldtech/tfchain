import { PricingPolicy, FarmingPolicy, Policy, Unit, CertificationType } from '../generated/model'
import { TfgridModule } from '../types'
import { hex2a } from './util'

import {
  EventContext,
  StoreContext,
} from '@subsquid/hydra-common'

export async function pricingPolicyStored({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const newPricingPolicy = new PricingPolicy()
  const [pricing_policy] = new TfgridModule.PricingPolicyStoredEvent(event).params

  newPricingPolicy.gridVersion = pricing_policy.version.toNumber()
  newPricingPolicy.pricingPolicyId = pricing_policy.id.toNumber()
  newPricingPolicy.name = hex2a(Buffer.from(pricing_policy.name.toString()).toString())

  newPricingPolicy.foundationAccount = Buffer.from(pricing_policy.foundation_account.toHex()).toString()
  newPricingPolicy.certifiedSalesAccount = Buffer.from(pricing_policy.certified_sales_account.toHex()).toString()

  const suPolicy = new Policy()
  suPolicy.value = pricing_policy.su.value.toNumber()
  suPolicy.unit = formatUnit(pricing_policy.su.unit.toString())
  await store.save<Policy>(suPolicy)

  const nuPolicy = new Policy()
  nuPolicy.value = pricing_policy.nu.value.toNumber()
  nuPolicy.unit = formatUnit(pricing_policy.nu.unit.toString())
  await store.save<Policy>(nuPolicy)

  const cuPolicy = new Policy()
  cuPolicy.value = pricing_policy.cu.value.toNumber()
  cuPolicy.unit = formatUnit(pricing_policy.cu.unit.toString())
  await store.save<Policy>(cuPolicy)

  const IpuPolicy = new Policy()
  IpuPolicy.value = pricing_policy.ipu.value.toNumber()
  IpuPolicy.unit = formatUnit(pricing_policy.ipu.unit.toString())
  await store.save<Policy>(IpuPolicy)

  newPricingPolicy.su = suPolicy
  newPricingPolicy.cu = cuPolicy
  newPricingPolicy.nu = nuPolicy
  newPricingPolicy.ipu = IpuPolicy

  await store.save<PricingPolicy>(newPricingPolicy)
}

export async function farmingPolicyStored({
  store,
  event,
  block,
  extrinsic,
}: EventContext & StoreContext) {
  const newFarmingPolicy = new FarmingPolicy()
  const [farming_policy] = new TfgridModule.FarmingPolicyStoredEvent(event).params

  newFarmingPolicy.gridVersion = farming_policy.version.toNumber()
  newFarmingPolicy.farmingPolicyId = farming_policy.id.toNumber()
  newFarmingPolicy.name = hex2a(Buffer.from(farming_policy.name.toString()).toString())

  newFarmingPolicy.cu = farming_policy.cu.toNumber()
  newFarmingPolicy.su = farming_policy.su.toNumber()
  newFarmingPolicy.nu = farming_policy.nu.toNumber()
  newFarmingPolicy.ipv4 = farming_policy.ipv4.toNumber()
  newFarmingPolicy.timestamp = farming_policy.timestamp.toNumber()

  const certificationTypeAsString = farming_policy.certification_type.toString()
  let certType = CertificationType.Diy
  switch (certificationTypeAsString) {
    case 'Diy': certType = CertificationType.Diy
    case 'Diy': certType = CertificationType.Certified
  }
  newFarmingPolicy.certificationType = certType

  await store.save<FarmingPolicy>(newFarmingPolicy)
}

function formatUnit(unitAsString: string) : Unit {
  switch (unitAsString) {
    case 'Kilobytes': return Unit.Kilobytes
    case 'Megabytes': return Unit.Megabytes
    case 'Gigabytes': return Unit.Gigabytes
    case 'Terrabytes': return Unit.Terrabytes
    default: return Unit.Bytes
  }
}