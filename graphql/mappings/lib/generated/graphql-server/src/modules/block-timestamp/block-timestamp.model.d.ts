import { BaseModel } from 'warthog';
import BN from 'bn.js';
export declare class BlockTimestamp extends BaseModel {
    blockNumber: number;
    timestamp: BN;
    constructor(init?: Partial<BlockTimestamp>);
}
