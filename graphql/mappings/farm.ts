import { SubstrateEvent, DB } from '../generated/hydra-processor'
import { CertificationType, Farm } from '../generated/graphql-server/src/modules/farm/farm.model'
import BN from 'bn.js'
import { hex2a } from './util'

export async function tfgridModule_FarmStored(db: DB, event: SubstrateEvent) {
  const [farm_id, name, entity_id, twin_id, pricing_policy_id, country_id, city_id, certification_type] = event.params
  const farm = new Farm()
  
  farm.farmId = new BN(farm_id.value as number)
  farm.name = hex2a(Buffer.from(name.value as string).toString())
  farm.entityId = new BN(entity_id.value as number)
  farm.twinId = new BN(twin_id.value as number)
  farm.pricingPolicyId = new BN(pricing_policy_id.value as number)
  farm.countryId = new BN(country_id.value as number)
  farm.cityId = new BN(city_id.value as number)

  let certificationTypeAsString = Buffer.from(certification_type.value as string).toString()
  let certificationType = CertificationType.None
  switch (certificationTypeAsString) {
      case 'None': certificationType = CertificationType.None
      case 'Silver': certificationType = CertificationType.Silver
      case 'Gold': certificationType = CertificationType.Gold
      default: certificationType = CertificationType.None
  }
  farm.certificationType = certificationType

  await db.save<Farm>(farm)
}

export async function tfgridModule_FarmDeleted(db: DB, event: SubstrateEvent) {
    const [farm_id] = event.params
  
    const savedFarm = await db.get(Farm, { where: { farmId: new BN(farm_id.value as number) } })
  
    if (savedFarm) {
      await db.remove(savedFarm)
    }
  }