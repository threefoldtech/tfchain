import { BaseModel } from 'warthog';
import BN from 'bn.js';
import { Location } from '../location/location.model';
import { PublicConfig } from '../public-config/public-config.model';
export declare class Node extends BaseModel {
    gridVersion: number;
    nodeId: number;
    farmId: number;
    twinId: number;
    location: Location;
    countryId?: number;
    cityId?: number;
    address: string;
    hru?: BN;
    sru?: BN;
    cru?: BN;
    mru?: BN;
    role: string;
    publicConfig?: PublicConfig;
    constructor(init?: Partial<Node>);
}
