import { BaseModel, IntField, Model, StringField, JSONField } from '@subsquid/warthog';

import * as jsonTypes from '../jsonfields/jsonfields.model';

@Model({ api: {} })
export class Country extends BaseModel {
  @IntField({})
  countryId!: number;

  @StringField({})
  code!: string;

  @StringField({})
  name!: string;

  @StringField({})
  region!: string;

  @StringField({})
  subregion!: string;

  @StringField({
    nullable: true,
  })
  lat?: string;

  @StringField({
    nullable: true,
  })
  long?: string;

  constructor(init?: Partial<Country>) {
    super();
    Object.assign(this, init);
  }
}
