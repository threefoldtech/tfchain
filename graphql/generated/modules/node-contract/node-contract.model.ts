import { BaseModel, IntField, Model, EnumField, StringField, JSONField } from 'warthog';

import * as jsonTypes from '../jsonfields/jsonfields.model';

import { ContractState } from '../enums/enums';
export { ContractState };

@Model({ api: {} })
export class NodeContract extends BaseModel {
  @IntField({})
  version!: number;

  @IntField({})
  contractId!: number;

  @IntField({})
  twinId!: number;

  @IntField({})
  nodeId!: number;

  @StringField({})
  deploymentData!: string;

  @StringField({})
  deploymentHash!: string;

  @IntField({})
  numberOfPublicIPs!: number;

  @EnumField('ContractState', ContractState, {})
  state!: ContractState;

  constructor(init?: Partial<NodeContract>) {
    super();
    Object.assign(this, init);
  }
}
