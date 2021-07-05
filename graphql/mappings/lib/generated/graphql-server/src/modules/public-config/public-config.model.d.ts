import { BaseModel } from 'warthog';
import { Node } from '../node/node.model';
export declare class PublicConfig extends BaseModel {
    ipv4: string;
    ipv6: string;
    gw4: string;
    gw6: string;
    nodepublicConfig?: Node[];
    constructor(init?: Partial<PublicConfig>);
}
