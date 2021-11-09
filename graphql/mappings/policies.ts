import { PricingPolicy, FarmingPolicy, Policy, CertificationType } from '../generated/model'
import { TfgridModule } from '../chain'
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
  let pricingPolicy = new PricingPolicy()
  const [pricing_policy] = new TfgridModule.PricingPolicyStoredEvent(event).params

  const savedPolicy = await store.get(PricingPolicy, { where: { pricingPolicyId: pricing_policy.id.toNumber() }})
  if (savedPolicy) {
    pricingPolicy = savedPolicy
  }

  pricingPolicy.gridVersion = pricing_policy.version.toNumber()
  pricingPolicy.pricingPolicyId = pricing_policy.id.toNumber()
  pricingPolicy.name = hex2a(Buffer.from(pricing_policy.name.toString()).toString())

  pricingPolicy.foundationAccount = Buffer.from(pricing_policy.foundation_account.toHex()).toString()
  pricingPolicy.certifiedSalesAccount = Buffer.from(pricing_policy.certified_sales_account.toHex()).toString()

  const suPolicy = new Policy()
  suPolicy.value = pricing_policy.su.value.toNumber()
  suPolicy.unit = pricing_policy.su.unit.toString()

  const nuPolicy = new Policy()
  nuPolicy.value = pricing_policy.nu.value.toNumber()
  nuPolicy.unit = pricing_policy.nu.unit.toString()

  const cuPolicy = new Policy()
  cuPolicy.value = pricing_policy.cu.value.toNumber()
  cuPolicy.unit = pricing_policy.cu.unit.toString()

  const IpuPolicy = new Policy()
  IpuPolicy.value = pricing_policy.ipu.value.toNumber()
  IpuPolicy.unit = pricing_policy.ipu.unit.toString()

  pricingPolicy.su = suPolicy
  pricingPolicy.cu = cuPolicy
  pricingPolicy.nu = nuPolicy
  pricingPolicy.ipu = IpuPolicy

  await store.save<PricingPolicy>(pricingPolicy)
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
    case 'Diy': 
      certType = CertificationType.Diy
      break
    case 'Certified': 
      certType = CertificationType.Certified
      break
  }

  newFarmingPolicy.certificationType = certType

  await store.save<FarmingPolicy>(newFarmingPolicy)
}