import { BaseModel, NumericField, Model, StringField } from 'warthog';

import BN from 'bn.js';

@Model({ api: {} })
export class City extends BaseModel {
  @NumericField({
    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined
    }
  })
  countryId!: BN;

  @StringField({})
  name!: string;

  constructor(init?: Partial<City>) {
    super();
    Object.assign(this, init);
  }
}
