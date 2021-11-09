import {
  BaseModel,
  IntField,
  NumericField,
  Model,
  ManyToOne,
  OneToMany,
  StringField,
  JSONField,
} from '@subsquid/warthog';

import BN from 'bn.js';

import { Location } from '../location/location.model';
import { Interfaces } from '../interfaces/interfaces.model';

import * as jsonTypes from '../jsonfields/jsonfields.model';

@Model({ api: {} })
export class Node extends BaseModel {
  @IntField({})
  gridVersion!: number;

  @IntField({})
  nodeId!: number;

  @IntField({})
  farmId!: number;

  @IntField({})
  twinId!: number;

  @ManyToOne(() => Location, (param: Location) => param.nodelocation, {
    skipGraphQLField: true,

    modelName: 'Node',
    relModelName: 'Location',
    propertyName: 'location',
  })
  location!: Location;

  @StringField({
    nullable: true,
  })
  country?: string;

  @StringField({
    nullable: true,
  })
  city?: string;

  @NumericField({
    nullable: true,

    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined,
    },
  })
  hru?: BN;

  @NumericField({
    nullable: true,

    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined,
    },
  })
  sru?: BN;

  @NumericField({
    nullable: true,

    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined,
    },
  })
  cru?: BN;

  @NumericField({
    nullable: true,

    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined,
    },
  })
  mru?: BN;

  @JSONField({ filter: true, gqlFieldType: jsonTypes.PublicConfig, nullable: true })
  publicConfig?: jsonTypes.PublicConfig;

  @IntField({
    nullable: true,
  })
  uptime?: number;

  @IntField({})
  created!: number;

  @IntField({})
  farmingPolicyId!: number;

  @OneToMany(() => Interfaces, (param: Interfaces) => param.node, {
    modelName: 'Node',
    relModelName: 'Interfaces',
    propertyName: 'interfaces',
  })
  interfaces?: Interfaces[];

  constructor(init?: Partial<Node>) {
    super();
    Object.assign(this, init);
  }
}
