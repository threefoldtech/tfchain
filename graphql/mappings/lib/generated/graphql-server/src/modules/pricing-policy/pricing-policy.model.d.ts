import { BaseModel } from 'warthog';
export declare class PricingPolicy extends BaseModel {
    gridVersion: number;
    pricingPolicyId: number;
    name: string;
    currency: string;
    su: number;
    cu: number;
    nu: number;
    constructor(init?: Partial<PricingPolicy>);
}
