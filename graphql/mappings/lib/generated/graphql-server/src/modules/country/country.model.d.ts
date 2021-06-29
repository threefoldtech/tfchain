import { BaseModel } from 'warthog';
export declare class Country extends BaseModel {
    code: string;
    name: string;
    constructor(init?: Partial<Country>);
}
