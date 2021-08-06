import { BaseModel } from 'warthog';
import { ContractState } from '../enums/enums';
export { ContractState };
export declare class NodeContract extends BaseModel {
    version: number;
    contractId: number;
    twinId: number;
    nodeId: number;
    deploymentData: string;
    deploymentHash: string;
    numberOfPublicIPs: number;
    state: ContractState;
    constructor(init?: Partial<NodeContract>);
}
