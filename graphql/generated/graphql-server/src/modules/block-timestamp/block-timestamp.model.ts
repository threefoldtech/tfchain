import { BaseModel, IntField, NumericField, Model, StringField } from 'warthog';

import BN from 'bn.js';

@Model({ api: { description: ` Tracks block timestamps ` } })
export class BlockTimestamp extends BaseModel {
  @IntField({})
  blockNumber!: number;

  @NumericField({
    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined
    }
  })
  timestamp!: BN;

  constructor(init?: Partial<BlockTimestamp>) {
    super();
    Object.assign(this, init);
  }
}
