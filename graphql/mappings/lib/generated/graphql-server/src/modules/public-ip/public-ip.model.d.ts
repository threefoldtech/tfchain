import { BaseModel } from 'warthog';
import { Farm } from '../farm/farm.model';
export declare class PublicIp extends BaseModel {
    farm?: Farm;
    gateway: string;
    ip: string;
    contractId: number;
    constructor(init?: Partial<PublicIp>);
}
