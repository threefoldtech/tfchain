import { BaseModel, IntField, Model, StringField, JSONField } from '@subsquid/warthog';

import * as jsonTypes from '../jsonfields/jsonfields.model';

@Model({ api: {} })
export class Entity extends BaseModel {
  @IntField({})
  gridVersion!: number;

  @IntField({})
  entityId!: number;

  @StringField({})
  name!: string;

  @StringField({
    nullable: true,
  })
  country?: string;

  @StringField({
    nullable: true,
  })
  city?: string;

  @StringField({})
  accountId!: string;

  constructor(init?: Partial<Entity>) {
    super();
    Object.assign(this, init);
  }
}
