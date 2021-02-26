import { BaseModel, IntField, Model, StringField } from 'warthog';

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
  address!: string;

  constructor(init?: Partial<Entity>) {
    super();
    Object.assign(this, init);
  }
}
