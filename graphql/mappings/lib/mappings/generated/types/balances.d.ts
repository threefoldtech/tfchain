import { SubstrateEvent } from "@subsquid/hydra-common";
import { AccountId, Balance } from "@polkadot/types/interfaces";
export declare namespace Balances {
    /**
     *  Transfer succeeded. \[from, to, value\]
     *
     *  Event parameters: [AccountId, AccountId, Balance, ]
     */
    class TransferEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [AccountId, AccountId, Balance];
        validateParams(): boolean;
    }
}
