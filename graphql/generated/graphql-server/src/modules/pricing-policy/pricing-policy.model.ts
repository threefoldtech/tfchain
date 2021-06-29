import { BaseModel, IntField, Model, StringField } from 'warthog';

@Model({ api: {} })
export class PricingPolicy extends BaseModel {
  @IntField({})
  gridVersion!: number;

  @IntField({})
  pricingPolicyId!: number;

  @StringField({})
  name!: string;

  @StringField({})
  currency!: string;

  @IntField({})
  su!: number;

  @IntField({})
  cu!: number;

  @IntField({})
  nu!: number;

  constructor(init?: Partial<PricingPolicy>) {
    super();
    Object.assign(this, init);
  }
}
