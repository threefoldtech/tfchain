import { BaseModel, IntField, Model, StringField, JSONField } from 'warthog';

import * as jsonTypes from '../jsonfields/jsonfields.model';

@Model({ api: {} })
export class UptimeEvent extends BaseModel {
  @IntField({})
  nodeId!: number;

  @IntField({})
  uptime!: number;

  @IntField({})
  timestamp!: number;

  constructor(init?: Partial<UptimeEvent>) {
    super();
    Object.assign(this, init);
  }
}
