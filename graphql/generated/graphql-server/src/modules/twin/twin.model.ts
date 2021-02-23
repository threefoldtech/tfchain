import { BaseModel, NumericField, Model, OneToMany, StringField } from 'warthog';

import BN from 'bn.js';

import { EntityProof } from '../entity-proof/entity-proof.model';

@Model({ api: {} })
export class Twin extends BaseModel {
  @NumericField({
    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined
    }
  })
  gridVersion!: BN;

  @NumericField({
    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined
    }
  })
  twinId!: BN;

  @StringField({})
  address!: string;

  @StringField({})
  ip!: string;

  @OneToMany(
    () => EntityProof,
    (param: EntityProof) => param.twinRel,
    { nullable: true }
  )
  entityprooftwinRel?: EntityProof[];

  constructor(init?: Partial<Twin>) {
    super();
    Object.assign(this, init);
  }
}
