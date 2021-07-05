import { BaseModel, IntField, Model, StringField } from 'warthog';

@Model({ api: {} })
export class PublicIp extends BaseModel {
  @IntField({})
  farmId!: number;

  @StringField({})
  ip!: string;

  @IntField({})
  workloadId!: number;

  constructor(init?: Partial<PublicIp>) {
    super();
    Object.assign(this, init);
  }
}
