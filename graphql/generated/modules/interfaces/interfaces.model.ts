import { BaseModel, Model, ManyToOne, StringField, JSONField } from '@subsquid/warthog';

import { Node } from '../node/node.model';

import * as jsonTypes from '../jsonfields/jsonfields.model';

@Model({ api: {} })
export class Interfaces extends BaseModel {
  @ManyToOne(() => Node, (param: Node) => param.interfaces, {
    skipGraphQLField: true,

    modelName: 'Interfaces',
    relModelName: 'Node',
    propertyName: 'node',
  })
  node!: Node;

  @StringField({})
  name!: string;

  @StringField({})
  mac!: string;

  @StringField({})
  ips!: string;

  constructor(init?: Partial<Interfaces>) {
    super();
    Object.assign(this, init);
  }
}
