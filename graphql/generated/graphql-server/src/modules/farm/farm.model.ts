import { BaseModel, IntField, Model, EnumField, StringField } from 'warthog';

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

  @IntField({
    nullable: true
  })
  countryId?: number;

  @IntField({
    nullable: true
  })
  cityId?: number;

  constructor(init?: Partial<Farm>) {
    super();
    Object.assign(this, init);
  }
}
