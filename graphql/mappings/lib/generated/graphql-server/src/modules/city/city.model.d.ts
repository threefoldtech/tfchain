import { BaseModel } from 'warthog';
export declare class City extends BaseModel {
    cityId: number;
    countryId: number;
    name: string;
    constructor(init?: Partial<City>);
}
