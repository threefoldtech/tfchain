import { BaseModel } from 'warthog';
import { CertificationType } from '../enums/enums';
export { CertificationType };
export declare class Farm extends BaseModel {
    gridVersion: number;
    farmId: number;
    name: string;
    twinId: number;
    pricingPolicyId: number;
    certificationType: CertificationType;
    countryId?: number;
    cityId?: number;
    constructor(init?: Partial<Farm>);
}
