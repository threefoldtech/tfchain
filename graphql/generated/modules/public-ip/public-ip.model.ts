import { BaseModel, IntField, Model, ManyToOne, StringField, JSONField } from 'warthog';

import { Farm } from '../farm/farm.model';

import * as jsonTypes from '../jsonfields/jsonfields.model';

@Model({ api: {} })
export class PublicIp extends BaseModel {
  @ManyToOne(
    () => Farm,
    (param: Farm) => param.publicIPs,
    {
      skipGraphQLField: true,

      modelName: 'PublicIp',
      relModelName: 'Farm',
      propertyName: 'farm'
    }
  )
  farm!: Farm;

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
