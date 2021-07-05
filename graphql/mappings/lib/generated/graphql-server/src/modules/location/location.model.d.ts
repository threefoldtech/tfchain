import { BaseModel } from 'warthog';
import { Node } from '../node/node.model';
export declare class Location extends BaseModel {
    longitude: string;
    latitude: string;
    nodelocation?: Node[];
    constructor(init?: Partial<Location>);
}
