import { BaseModel, IntField, Model, ManyToOne, StringField, JSONField } from '@subsquid/warthog';

import { Policy } from '../policy/policy.model';

import * as jsonTypes from '../jsonfields/jsonfields.model';

@Model({ api: {} })
export class PricingPolicy extends BaseModel {
  @IntField({})
  gridVersion!: number;

  @IntField({})
  pricingPolicyId!: number;

  @StringField({})
  name!: string;

  @ManyToOne(() => Policy, (param: Policy) => param.pricingpolicysu, {
    skipGraphQLField: true,

    modelName: 'PricingPolicy',
    relModelName: 'Policy',
    propertyName: 'su',
  })
  su!: Policy;

  @ManyToOne(() => Policy, (param: Policy) => param.pricingpolicycu, {
    skipGraphQLField: true,

    modelName: 'PricingPolicy',
    relModelName: 'Policy',
    propertyName: 'cu',
  })
  cu!: Policy;

  @ManyToOne(() => Policy, (param: Policy) => param.pricingpolicynu, {
    skipGraphQLField: true,

    modelName: 'PricingPolicy',
    relModelName: 'Policy',
    propertyName: 'nu',
  })
  nu!: Policy;

  @ManyToOne(() => Policy, (param: Policy) => param.pricingpolicyipu, {
    skipGraphQLField: true,

    modelName: 'PricingPolicy',
    relModelName: 'Policy',
    propertyName: 'ipu',
  })
  ipu!: Policy;

  @StringField({})
  foundationAccount!: string;

  @StringField({})
  certifiedSalesAccount!: string;

  constructor(init?: Partial<PricingPolicy>) {
    super();
    Object.assign(this, init);
  }
}
