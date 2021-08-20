import { BaseModel, IntField, Model, StringField, JSONField } from 'warthog';

import * as jsonTypes from '../jsonfields/jsonfields.model';

@Model({ api: {} })
export class Entity extends BaseModel {
  @IntField({})
  gridVersion!: number;

  @IntField({})
  entityId!: number;

  @StringField({})
  name!: string;

  @IntField({
    nullable: true
  })
  countryId?: number;

  @IntField({
    nullable: true
  })
  cityId?: number;

  @StringField({})
  accountId!: string;

  constructor(init?: Partial<Entity>) {
    super();
    Object.assign(this, init);
  }
}
