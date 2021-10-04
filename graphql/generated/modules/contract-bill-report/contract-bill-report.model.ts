import { BaseModel, IntField, NumericField, Model, EnumField, StringField, JSONField } from '@subsquid/warthog';

import BN from 'bn.js';

import * as jsonTypes from '../jsonfields/jsonfields.model';

import { DiscountLevel } from '../enums/enums';
export { DiscountLevel };

@Model({ api: {} })
export class ContractBillReport extends BaseModel {
  @IntField({})
  contractId!: number;

  @EnumField('DiscountLevel', DiscountLevel, {})
  discountReceived!: DiscountLevel;

  @NumericField({
    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined,
    },
  })
  amountBilled!: BN;

  @IntField({})
  timestamp!: number;

  constructor(init?: Partial<ContractBillReport>) {
    super();
    Object.assign(this, init);
  }
}
