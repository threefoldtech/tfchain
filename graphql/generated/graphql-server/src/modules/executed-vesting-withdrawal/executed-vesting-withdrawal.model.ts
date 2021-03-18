import { BaseModel, IntField, Model, StringField } from 'warthog';

@Model({ api: {} })
export class ExecutedVestingWithdrawal extends BaseModel {
  @StringField({})
  from!: string;

  @StringField({})
  to!: string;

  @IntField({})
  value!: number;

  @StringField({})
  txXdr!: string;

  @IntField({})
  block!: number;

  constructor(init?: Partial<ExecutedVestingWithdrawal>) {
    super();
    Object.assign(this, init);
  }
}
