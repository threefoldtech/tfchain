import { BaseModel, IntField, Model, StringField, JSONField } from '@subsquid/warthog';

import * as jsonTypes from '../jsonfields/jsonfields.model';

@Model({ api: {} })
export class PricingPolicy extends BaseModel {
  @IntField({})
  gridVersion!: number;

  @IntField({})
  pricingPolicyId!: number;

  @StringField({})
  name!: string;

  @JSONField({ filter: true, gqlFieldType: jsonTypes.Policy })
  su!: jsonTypes.Policy;

  @JSONField({ filter: true, gqlFieldType: jsonTypes.Policy })
  cu!: jsonTypes.Policy;

  @JSONField({ filter: true, gqlFieldType: jsonTypes.Policy })
  nu!: jsonTypes.Policy;

  @JSONField({ filter: true, gqlFieldType: jsonTypes.Policy })
  ipu!: jsonTypes.Policy;

  @StringField({})
  foundationAccount!: string;

  @StringField({})
  certifiedSalesAccount!: string;

  constructor(init?: Partial<PricingPolicy>) {
    super();
    Object.assign(this, init);
  }
}
