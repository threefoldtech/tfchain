import { BaseModel, NumericField, Model, ManyToOne, StringField } from 'warthog';

import BN from 'bn.js';

import { Twin } from '../twin/twin.model';

@Model({ api: {} })
export class EntityProof extends BaseModel {
  @NumericField({
    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined
    }
  })
  entityId!: BN;

  @StringField({})
  signature!: string;

  @ManyToOne(
    () => Twin,
    (param: Twin) => param.entityprooftwinRel,
    { skipGraphQLField: true }
  )
  twinRel!: Twin;

  constructor(init?: Partial<EntityProof>) {
    super();
    Object.assign(this, init);
  }
}
