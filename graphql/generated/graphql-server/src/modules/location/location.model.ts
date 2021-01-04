import { BaseModel, Model, OneToMany, StringField } from 'warthog';

import { Node } from '../node/node.model';

@Model({ api: {} })
export class Location extends BaseModel {
  @StringField({})
  longitude!: string;

  @StringField({})
  latitude!: string;

  @OneToMany(
    () => Node,
    (param: Node) => param.location,
    { nullable: true }
  )
  nodelocation?: Node[];

  constructor(init?: Partial<Location>) {
    super();
    Object.assign(this, init);
  }
}
