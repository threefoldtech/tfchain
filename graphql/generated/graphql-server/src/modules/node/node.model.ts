import { BaseModel, IntField, Model, ManyToOne, StringField } from 'warthog';

import { Resource } from '../resource/resource.model';
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
    () => Resource,
    (param: Resource) => param.noderesources,
    { skipGraphQLField: true }
  )
  resources!: Resource;

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

  constructor(init?: Partial<Node>) {
    super();
    Object.assign(this, init);
  }
}
