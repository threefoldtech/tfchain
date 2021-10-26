import { BaseModel, IntField, NumericField, Model, StringField, JSONField } from '@subsquid/warthog';

import BN from 'bn.js';

import * as jsonTypes from '../jsonfields/jsonfields.model';

@Model({ api: {} })
export class RefundTransaction extends BaseModel {
  @IntField({})
  block!: number;

  @NumericField({
    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined,
    },
  })
  amount!: BN;

  @StringField({})
  target!: string;

  @StringField({})
  txHash!: string;

  constructor(init?: Partial<RefundTransaction>) {
    super();
    Object.assign(this, init);
  }
}
