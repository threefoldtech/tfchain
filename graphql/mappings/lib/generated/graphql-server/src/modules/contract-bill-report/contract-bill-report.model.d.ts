import { BaseModel } from 'warthog';
export declare class ContractBillReport extends BaseModel {
    contractId: number;
    discountReceived: string;
    amountBilled: number;
    constructor(init?: Partial<ContractBillReport>);
}
