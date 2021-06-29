import { BaseModel } from 'warthog';
import { Twin } from '../twin/twin.model';
export declare class EntityProof extends BaseModel {
    entityId: number;
    signature: string;
    twinRel: Twin;
    constructor(init?: Partial<EntityProof>);
}
