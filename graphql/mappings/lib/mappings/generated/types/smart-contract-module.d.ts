import { SubstrateEvent } from "@subsquid/hydra-common";
import { Consumption } from "substrate-tfgrid-ts-types";
export declare namespace SmartContractModule {
    class ConsumptionReportReceivedEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [Consumption];
        validateParams(): boolean;
    }
}
