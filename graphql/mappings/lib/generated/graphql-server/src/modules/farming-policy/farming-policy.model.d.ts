import { BaseModel } from 'warthog';
import { CertificationType } from '../enums/enums';
export { CertificationType };
export declare class FarmingPolicy extends BaseModel {
    gridVersion: number;
    farmingPolicyId: number;
    name: string;
    cu: number;
    su: number;
    nu: number;
    ipv4: number;
    timestamp: number;
    certificationType: CertificationType;
    constructor(init?: Partial<FarmingPolicy>);
}
