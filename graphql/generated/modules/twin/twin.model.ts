import { BaseModel, IntField, Model, OneToMany, StringField, JSONField } from '@subsquid/warthog';

import { EntityProof } from '../entity-proof/entity-proof.model';

import * as jsonTypes from '../jsonfields/jsonfields.model';

@Model({ api: {} })
export class Twin extends BaseModel {
  @IntField({})
  gridVersion!: number;

  @IntField({})
  twinId!: number;

  @StringField({})
  accountId!: string;

  @StringField({})
  ip!: string;

  @OneToMany(() => EntityProof, (param: EntityProof) => param.twinRel, {
    nullable: true,
    modelName: 'Twin',
    relModelName: 'EntityProof',
    propertyName: 'entityprooftwinRel',
  })
  entityprooftwinRel?: EntityProof[];

  constructor(init?: Partial<Twin>) {
    super();
    Object.assign(this, init);
  }
}
