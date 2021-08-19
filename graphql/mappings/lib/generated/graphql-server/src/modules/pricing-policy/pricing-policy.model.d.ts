import { BaseModel } from 'warthog';
import { Policy } from '../policy/policy.model';
export declare class PricingPolicy extends BaseModel {
    gridVersion: number;
    pricingPolicyId: number;
    name: string;
    su: Policy;
    cu: Policy;
    nu: Policy;
    ipu: Policy;
    foundationAccount: string;
    certifiedSalesAccount: string;
    constructor(init?: Partial<PricingPolicy>);
}
