import { BaseModel, IntField, Model, EnumField, StringField } from 'warthog';

import { DiscountLevel } from '../enums/enums';
export { DiscountLevel };

@Model({ api: {} })
export class ContractBillReport extends BaseModel {
  @IntField({})
  contractId!: number;

  @EnumField('DiscountLevel', DiscountLevel, {})
  discountReceived!: DiscountLevel;

  @IntField({})
  amountBilled!: number;

  @IntField({})
  timestamp!: number;

  constructor(init?: Partial<ContractBillReport>) {
    super();
    Object.assign(this, init);
  }
}
