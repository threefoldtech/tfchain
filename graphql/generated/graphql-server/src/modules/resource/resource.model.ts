import { BaseModel, IntField, Model, OneToMany, StringField } from 'warthog';

import { Node } from '../node/node.model';

@Model({ api: {} })
export class Resource extends BaseModel {
  @IntField({
    nullable: true
  })
  hru?: number;

  @IntField({
    nullable: true
  })
  sru?: number;

  @IntField({
    nullable: true
  })
  cru?: number;

  @IntField({
    nullable: true
  })
  mru?: number;

  @OneToMany(
    () => Node,
    (param: Node) => param.resources,
    { nullable: true }
  )
  noderesources?: Node[];

  constructor(init?: Partial<Resource>) {
    super();
    Object.assign(this, init);
  }
}
