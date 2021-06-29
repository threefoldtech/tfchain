import { BaseModel } from 'warthog';
import BN from 'bn.js';
export declare class Transfer extends BaseModel {
    from: string;
    to: string;
    value: BN;
    comment?: string;
    block: number;
    constructor(init?: Partial<Transfer>);
}
