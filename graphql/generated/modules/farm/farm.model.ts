import { BaseModel, IntField, Model, OneToMany, EnumField, StringField, JSONField } from 'warthog';

import { PublicIp } from '../public-ip/public-ip.model';

import * as jsonTypes from '../jsonfields/jsonfields.model';

import { CertificationType } from '../enums/enums';
export { CertificationType };

@Model({ api: {} })
export class Farm extends BaseModel {
  @IntField({})
  gridVersion!: number;

  @IntField({})
  farmId!: number;

  @StringField({})
  name!: string;

  @IntField({})
  twinId!: number;

  @IntField({})
  pricingPolicyId!: number;

  @EnumField('CertificationType', CertificationType, {})
  certificationType!: CertificationType;

  @OneToMany(
    () => PublicIp,
    (param: PublicIp) => param.farm,
    {
      modelName: 'Farm',
      relModelName: 'PublicIp',
      propertyName: 'publicIPs'
    }
  )
  publicIPs?: PublicIp[];

  @StringField({
    nullable: true
  })
  stellarAddress?: string;

  constructor(init?: Partial<Farm>) {
    super();
    Object.assign(this, init);
  }
}
