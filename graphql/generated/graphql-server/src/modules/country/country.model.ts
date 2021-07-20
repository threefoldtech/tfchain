import { BaseModel, Model, StringField } from 'warthog';

@Model({ api: {} })
export class Country extends BaseModel {
  @StringField({})
  code!: string;

  @StringField({})
  name!: string;

  @StringField({})
  region!: string;

  @StringField({})
  subregion!: string;

  constructor(init?: Partial<Country>) {
    super();
    Object.assign(this, init);
  }
}
