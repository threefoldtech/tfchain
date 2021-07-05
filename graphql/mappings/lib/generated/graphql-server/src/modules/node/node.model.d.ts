import { BaseModel } from 'warthog';
import { Location } from '../location/location.model';
export declare class Node extends BaseModel {
    gridVersion: number;
    nodeId: number;
    farmId: number;
    location: Location;
    countryId?: number;
    cityId?: number;
    address: string;
    pubKey: string;
    hru?: number;
    sru?: number;
    cru?: number;
    mru?: number;
    role: string;
    constructor(init?: Partial<Node>);
}
