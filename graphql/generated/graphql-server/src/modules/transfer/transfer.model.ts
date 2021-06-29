import { BaseModel, IntField, NumericField, BytesField, DateTimeField, Model, StringField } from 'warthog';

import BN from 'bn.js';

@Model({ api: { description: ` All transfers ` } })
export class Transfer extends BaseModel {
  @BytesField({})
  from!: Buffer;

  @BytesField({})
  to!: Buffer;

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

  @NumericField({
    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined
    }
  })
  tip!: BN;

  @NumericField({
    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined
    }
  })
  timestamp!: BN;

  @DateTimeField({})
  insertedAt!: Date;

  constructor(init?: Partial<Transfer>) {
    super();
    Object.assign(this, init);
  }
}
