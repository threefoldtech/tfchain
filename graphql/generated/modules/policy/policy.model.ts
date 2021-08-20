import { BaseModel, IntField, Model, OneToMany, EnumField, StringField, JSONField } from 'warthog';

import { PricingPolicy } from '../pricing-policy/pricing-policy.model';

import * as jsonTypes from '../jsonfields/jsonfields.model';

import { Unit } from '../enums/enums';
export { Unit };

@Model({ api: {} })
export class Policy extends BaseModel {
  @IntField({})
  value!: number;

  @EnumField('Unit', Unit, {})
  unit!: Unit;

  @OneToMany(
    () => PricingPolicy,
    (param: PricingPolicy) => param.su,
    {
      nullable: true,
      modelName: 'Policy',
      relModelName: 'PricingPolicy',
      propertyName: 'pricingpolicysu'
    }
  )
  pricingpolicysu?: PricingPolicy[];

  @OneToMany(
    () => PricingPolicy,
    (param: PricingPolicy) => param.cu,
    {
      nullable: true,
      modelName: 'Policy',
      relModelName: 'PricingPolicy',
      propertyName: 'pricingpolicycu'
    }
  )
  pricingpolicycu?: PricingPolicy[];

  @OneToMany(
    () => PricingPolicy,
    (param: PricingPolicy) => param.nu,
    {
      nullable: true,
      modelName: 'Policy',
      relModelName: 'PricingPolicy',
      propertyName: 'pricingpolicynu'
    }
  )
  pricingpolicynu?: PricingPolicy[];

  @OneToMany(
    () => PricingPolicy,
    (param: PricingPolicy) => param.ipu,
    {
      nullable: true,
      modelName: 'Policy',
      relModelName: 'PricingPolicy',
      propertyName: 'pricingpolicyipu'
    }
  )
  pricingpolicyipu?: PricingPolicy[];

  constructor(init?: Partial<Policy>) {
    super();
    Object.assign(this, init);
  }
}
