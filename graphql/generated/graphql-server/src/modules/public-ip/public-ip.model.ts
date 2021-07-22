import { BaseModel, IntField, Model, ManyToOne, StringField } from 'warthog';

import { Farm } from '../farm/farm.model';

@Model({ api: {} })
export class PublicIp extends BaseModel {
  @ManyToOne(
    () => Farm,
    (param: Farm) => param.publicIPs,
    {
      skipGraphQLField: true,
      nullable: true,
      modelName: 'PublicIp',
      relModelName: 'Farm',
      propertyName: 'farm'
    }
  )
  farm?: Farm;

  @StringField({})
  gateway!: string;

  @StringField({})
  ip!: string;

  @IntField({})
  contractId!: number;

  constructor(init?: Partial<PublicIp>) {
    super();
    Object.assign(this, init);
  }
}
