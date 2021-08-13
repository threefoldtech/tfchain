import { BaseModel } from 'warthog';
export declare class PricingPolicy extends BaseModel {
    gridVersion: number;
    pricingPolicyId: number;
    name: string;
    currency: string;
    su: number;
    cu: number;
    nu: number;
    foundationAccount: string;
    certifiedSalesAccount: string;
    constructor(init?: Partial<PricingPolicy>);
}
