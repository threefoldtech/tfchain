import { BaseModel, IntField, Model, StringField } from 'warthog';

@Model({ api: {} })
export class Consumption extends BaseModel {
  @IntField({})
  contractId!: number;

  @IntField({})
  timestamp!: number;

  @IntField({})
  cru!: number;

  @IntField({})
  sru!: number;

  @IntField({})
  hru!: number;

  @IntField({})
  mru!: number;

  @IntField({})
  nru!: number;

  constructor(init?: Partial<Consumption>) {
    super();
    Object.assign(this, init);
  }
}
