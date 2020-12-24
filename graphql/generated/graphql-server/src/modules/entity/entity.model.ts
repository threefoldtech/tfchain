import { BaseModel, NumericField, Model, StringField } from 'warthog';

import BN from 'bn.js';

@Model({ api: {} })
export class Entity extends BaseModel {
  @StringField({
    nullable: true
  })
  name?: string;

  @NumericField({
    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined
    }
  })
  countryId!: BN;

  @NumericField({
    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined
    }
  })
  cityId!: BN;

  constructor(init?: Partial<Entity>) {
    super();
    Object.assign(this, init);
  }
}
