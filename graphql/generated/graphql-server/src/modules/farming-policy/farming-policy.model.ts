import { BaseModel, IntField, Model, EnumField, StringField } from 'warthog';

import { CertificationType } from '../enums/enums';
export { CertificationType };

@Model({ api: {} })
export class FarmingPolicy extends BaseModel {
  @IntField({})
  version!: number;

  @IntField({})
  farmingPolicyId!: number;

  @StringField({})
  name!: string;

  @IntField({})
  cu!: number;

  @IntField({})
  su!: number;

  @IntField({})
  nu!: number;

  @IntField({})
  ipv4!: number;

  @IntField({})
  timestamp!: number;

  @EnumField('CertificationType', CertificationType, {})
  certificationType!: CertificationType;

  constructor(init?: Partial<FarmingPolicy>) {
    super();
    Object.assign(this, init);
  }
}
