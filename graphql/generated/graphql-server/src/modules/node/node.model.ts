import { BaseModel, NumericField, Model, ManyToOne, StringField } from 'warthog';

import BN from 'bn.js';

import { Resource } from '../resource/resource.model';
import { Location } from '../location/location.model';

@Model({ api: {} })
export class Node extends BaseModel {
  @NumericField({
    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined
    }
  })
  nodeId!: BN;

  @NumericField({
    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined
    }
  })
  farmId!: BN;

  @NumericField({
    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined
    }
  })
  twinId!: BN;

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

  @NumericField({
    nullable: true,

    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined
    }
  })
  countryId?: BN;

  @NumericField({
    nullable: true,

    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined
    }
  })
  cityId?: BN;

  constructor(init?: Partial<Node>) {
    super();
    Object.assign(this, init);
  }
}
