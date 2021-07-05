import { BaseModel } from 'warthog';
export declare class Entity extends BaseModel {
    gridVersion: number;
    entityId: number;
    name: string;
    countryId?: number;
    cityId?: number;
    address: string;
    constructor(init?: Partial<Entity>);
}
