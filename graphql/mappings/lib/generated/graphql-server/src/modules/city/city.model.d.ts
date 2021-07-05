import { BaseModel } from 'warthog';
export declare class City extends BaseModel {
    countryId: number;
    name: string;
    constructor(init?: Partial<City>);
}
