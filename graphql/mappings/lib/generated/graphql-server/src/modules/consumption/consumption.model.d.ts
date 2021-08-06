import { BaseModel } from 'warthog';
import BN from 'bn.js';
export declare class Consumption extends BaseModel {
    contractId: number;
    timestamp: number;
    cru?: BN;
    sru?: BN;
    hru?: BN;
    mru?: BN;
    nru?: BN;
    constructor(init?: Partial<Consumption>);
}
