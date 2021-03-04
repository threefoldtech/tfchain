import { SubstrateEvent, DB } from '../generated/hydra-processor'
import { CertificationType, Farm } from '../generated/graphql-server/src/modules/farm/farm.model'
import { hex2a } from './util'

export async function tfgridModule_FarmStored(db: DB, event: SubstrateEvent) {
  const [version, farm_id, name, twin_id, pricing_policy_id, country_id, city_id, certification_type] = event.params
  const farm = new Farm()
  
  farm.gridVersion = version.value as number
  farm.farmId = farm_id.value as number
  farm.name = hex2a(Buffer.from(name.value as string).toString())
  farm.twinId = twin_id.value as number
  farm.pricingPolicyId = pricing_policy_id.value as number
  farm.countryId = country_id.value as number
  farm.cityId = city_id.value as number

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

  const savedFarm = await db.get(Farm, { where: { farmId: farm_id.value as number } })

  if (savedFarm) {
    await db.remove(savedFarm)
  }
}