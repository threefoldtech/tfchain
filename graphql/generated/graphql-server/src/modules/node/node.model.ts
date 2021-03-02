import { BaseModel, IntField, Model, ManyToOne, StringField } from 'warthog';

import { Location } from '../location/location.model';

@Model({ api: {} })
export class Node extends BaseModel {
  @IntField({})
  gridVersion!: number;

  @IntField({})
  nodeId!: number;

  @IntField({})
  farmId!: number;

  @ManyToOne(
    () => Location,
    (param: Location) => param.nodelocation,
    { skipGraphQLField: true }
  )
  location!: Location;

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

  @StringField({})
  pubKey!: string;

  @IntField({
    nullable: true
  })
  hru?: number;

  @IntField({
    nullable: true
  })
  sru?: number;

  @IntField({
    nullable: true
  })
  cru?: number;

  @IntField({
    nullable: true
  })
  mru?: number;

  @StringField({})
  role!: string;

  constructor(init?: Partial<Node>) {
    super();
    Object.assign(this, init);
  }
}
