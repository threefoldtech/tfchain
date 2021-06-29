import { BaseModel } from 'warthog';
import { EntityProof } from '../entity-proof/entity-proof.model';
export declare class Twin extends BaseModel {
    gridVersion: number;
    twinId: number;
    address: string;
    ip: string;
    entityprooftwinRel?: EntityProof[];
    constructor(init?: Partial<Twin>);
}
