import { Farm, CertificationType } from '../../generated/graphql-server/model'
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
  const [version, farm_id, name, twin_id, pricing_policy_id, country_id, city_id, certificationType]  = new TfgridModule.FarmStoredEvent(event).params

  const farm = new Farm()
  
  farm.gridVersion = version.toNumber()
  farm.farmId = farm_id.toNumber()
  farm.name = hex2a(name.toString())
  farm.twinId = twin_id.toNumber()
  farm.pricingPolicyId = pricing_policy_id.toNumber()
  farm.countryId = country_id.toNumber()
  farm.cityId = city_id.toNumber()

  const certificationTypeAsString = certificationType.toString()
  let certType = CertificationType.None
  switch (certificationTypeAsString) {
    case 'Gold': certType = CertificationType.Gold
    case 'Silver': certType = CertificationType.Silver
  }
  farm.certificationType = certType

  await store.save<Farm>(farm)
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