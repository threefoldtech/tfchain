import { BaseModel } from 'warthog';
export declare class Country extends BaseModel {
    code: string;
    name: string;
    region: string;
    subregion: string;
    constructor(init?: Partial<Country>);
}
