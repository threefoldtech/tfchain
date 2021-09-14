import { BaseModel, IntField, Model, EnumField, StringField, JSONField } from '@subsquid/warthog';

import * as jsonTypes from '../jsonfields/jsonfields.model';

import { ContractState } from '../enums/enums';
export { ContractState };

@Model({ api: {} })
export class NameContract extends BaseModel {
  @IntField({})
  version!: number;

  @IntField({})
  contractId!: number;

  @IntField({})
  twinId!: number;

  @StringField({})
  name!: string;

  @EnumField('ContractState', ContractState, {})
  state!: ContractState;

  constructor(init?: Partial<NameContract>) {
    super();
    Object.assign(this, init);
  }
}
