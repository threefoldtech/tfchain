import { BaseModel, IntField, NumericField, Model, StringField, JSONField } from 'warthog';

import BN from 'bn.js';

import * as jsonTypes from '../jsonfields/jsonfields.model';

@Model({ api: { description: ` All transfers ` } })
export class Transfer extends BaseModel {
  @StringField({})
  from!: string;

  @StringField({})
  to!: string;

  @NumericField({
    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined
    }
  })
  value!: BN;

  @StringField({
    nullable: true
  })
  comment?: string;

  @IntField({})
  block!: number;

  constructor(init?: Partial<Transfer>) {
    super();
    Object.assign(this, init);
  }
}
