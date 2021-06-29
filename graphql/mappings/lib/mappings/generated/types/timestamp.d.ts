import { SubstrateEvent, SubstrateExtrinsic } from "@subsquid/hydra-common";
import { Compact } from "@polkadot/types";
import { Moment } from "@polkadot/types/interfaces";
export declare namespace Timestamp {
    /**
     *  Set the current time.
     *
     *  This call should be invoked exactly once per block. It will panic at the finalization
     *  phase, if this call hasn't been invoked by that time.
     *
     *  The timestamp should be greater than the previous one by the amount specified by
     *  `MinimumPeriod`.
     *
     *  The dispatch origin for this call must be `Inherent`.
     *
     *  # <weight>
     *  - `O(1)` (Note that implementations of `OnTimestampSet` must also be `O(1)`)
     *  - 1 storage read and 1 storage mutation (codec `O(1)`). (because of `DidUpdate::take` in `on_finalize`)
     *  - 1 event handler `on_timestamp_set`. Must be `O(1)`.
     *  # </weight>
     */
    export class SetCall {
        readonly ctx: SubstrateEvent;
        readonly extrinsic: SubstrateExtrinsic;
        readonly expectedArgTypes: string[];
        constructor(ctx: SubstrateEvent);
        get args(): Set_Args;
        validateArgs(): boolean;
    }
    class Set_Args {
        readonly extrinsic: SubstrateExtrinsic;
        constructor(extrinsic: SubstrateExtrinsic);
        get now(): Compact<Moment>;
    }
    export {};
}
