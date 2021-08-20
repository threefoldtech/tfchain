import { BaseModel, IntField, Model, ManyToOne, StringField, JSONField } from 'warthog';

import { Twin } from '../twin/twin.model';

import * as jsonTypes from '../jsonfields/jsonfields.model';

@Model({ api: {} })
export class EntityProof extends BaseModel {
  @IntField({})
  entityId!: number;

  @StringField({})
  signature!: string;

  @ManyToOne(
    () => Twin,
    (param: Twin) => param.entityprooftwinRel,
    {
      skipGraphQLField: true,

      modelName: 'EntityProof',
      relModelName: 'Twin',
      propertyName: 'twinRel'
    }
  )
  twinRel!: Twin;

  constructor(init?: Partial<EntityProof>) {
    super();
    Object.assign(this, init);
  }
}
