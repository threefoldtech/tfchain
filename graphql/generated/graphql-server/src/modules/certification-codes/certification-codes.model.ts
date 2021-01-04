import { BaseModel, NumericField, Model, EnumField, StringField } from 'warthog';

import BN from 'bn.js';

import { CertificationCodeType } from '../enums/enums';
export { CertificationCodeType };

@Model({ api: {} })
export class CertificationCodes extends BaseModel {
  @NumericField({
    transformer: {
      to: (entityValue: BN) => (entityValue !== undefined ? entityValue.toString(10) : null),
      from: (dbValue: string) =>
        dbValue !== undefined && dbValue !== null && dbValue.length > 0 ? new BN(dbValue, 10) : undefined
    }
  })
  certificationCodeId!: BN;

  @StringField({})
  name!: string;

  @StringField({
    nullable: true
  })
  description?: string;

  @EnumField('CertificationCodeType', CertificationCodeType, {
    nullable: true
  })
  certificationCodeType?: CertificationCodeType;

  constructor(init?: Partial<CertificationCodes>) {
    super();
    Object.assign(this, init);
  }
}
