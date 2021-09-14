import { BaseModel, NumericField, Model, OneToMany, StringField, JSONField } from '@subsquid/warthog';

import BN from 'bn.js';

import { HistoricalBalance } from '../historical-balance/historical-balance.model';

import * as jsonTypes from '../jsonfields/jsonfields.model';

@Model({ api: {} })
export class Account extends BaseModel {
  @StringField({})
  wallet!: string;

  @NumericField({
    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined,
    },
  })
  balance!: BN;

  @OneToMany(() => HistoricalBalance, (param: HistoricalBalance) => param.account, {
    modelName: 'Account',
    relModelName: 'HistoricalBalance',
    propertyName: 'historicalBalances',
  })
  historicalBalances?: HistoricalBalance[];

  constructor(init?: Partial<Account>) {
    super();
    Object.assign(this, init);
  }
}
