import { BaseModel, Model, OneToMany, StringField, JSONField } from 'warthog';

import { Node } from '../node/node.model';

import * as jsonTypes from '../jsonfields/jsonfields.model';

@Model({ api: {} })
export class Location extends BaseModel {
  @StringField({})
  longitude!: string;

  @StringField({})
  latitude!: string;

  @OneToMany(
    () => Node,
    (param: Node) => param.location,
    {
      nullable: true,
      modelName: 'Location',
      relModelName: 'Node',
      propertyName: 'nodelocation'
    }
  )
  nodelocation?: Node[];

  constructor(init?: Partial<Location>) {
    super();
    Object.assign(this, init);
  }
}
