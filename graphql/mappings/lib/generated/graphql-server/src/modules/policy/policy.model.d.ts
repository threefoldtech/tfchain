import { BaseModel } from 'warthog';
import { PricingPolicy } from '../pricing-policy/pricing-policy.model';
import { Unit } from '../enums/enums';
export { Unit };
export declare class Policy extends BaseModel {
    value: number;
    unit: Unit;
    pricingpolicysu?: PricingPolicy[];
    pricingpolicycu?: PricingPolicy[];
    pricingpolicynu?: PricingPolicy[];
    pricingpolicyipu?: PricingPolicy[];
    constructor(init?: Partial<Policy>);
}
