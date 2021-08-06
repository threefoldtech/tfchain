import { BaseModel, IntField, Model, StringField } from 'warthog';

@Model({ api: {} })
export class ContractBillReport extends BaseModel {
  @IntField({})
  contractId!: number;

  @StringField({})
  discountReceived!: string;

  @IntField({})
  amountBilled!: number;

  constructor(init?: Partial<ContractBillReport>) {
    super();
    Object.assign(this, init);
  }
}
