import { SubstrateEvent } from "@subsquid/hydra-common";
import { Consumption, NodeContract } from "substrate-tfgrid-ts-types";
import { Bytes, u128, u64 } from "@polkadot/types";
export declare namespace SmartContractModule {
    class ConsumptionReportReceivedEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [Consumption];
        validateParams(): boolean;
    }
    class ContractCreatedEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [NodeContract];
        validateParams(): boolean;
    }
    class ContractUpdatedEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [NodeContract];
        validateParams(): boolean;
    }
    class ContractCanceledEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [u64];
        validateParams(): boolean;
    }
    class ContractBilledEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [u64, Bytes, u128];
        validateParams(): boolean;
    }
}
