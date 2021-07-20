import { BaseModel } from 'warthog';
export declare class Consumption extends BaseModel {
    contractId: number;
    timestamp: number;
    cru: number;
    sru: number;
    hru: number;
    mru: number;
    nru: number;
    constructor(init?: Partial<Consumption>);
}
