import { BaseModel } from 'warthog';
export declare class PublicIp extends BaseModel {
    farmId: number;
    ip: string;
    workloadId: number;
    constructor(init?: Partial<PublicIp>);
}
