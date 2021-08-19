import { BaseModel } from 'warthog';
import { DiscountLevel } from '../enums/enums';
export { DiscountLevel };
export declare class ContractBillReport extends BaseModel {
    contractId: number;
    discountReceived: DiscountLevel;
    amountBilled: number;
    timestamp: number;
    constructor(init?: Partial<ContractBillReport>);
}
