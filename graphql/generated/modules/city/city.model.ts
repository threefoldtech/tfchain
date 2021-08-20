import { BaseModel, IntField, Model, StringField, JSONField } from 'warthog';

import * as jsonTypes from '../jsonfields/jsonfields.model';

@Model({ api: {} })
export class City extends BaseModel {
  @IntField({})
  cityId!: number;

  @IntField({})
  countryId!: number;

  @StringField({})
  name!: string;

  constructor(init?: Partial<City>) {
    super();
    Object.assign(this, init);
  }
}
