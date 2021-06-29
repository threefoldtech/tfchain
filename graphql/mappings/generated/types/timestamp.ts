import { createTypeUnsafe } from "@polkadot/types/create";
import { SubstrateEvent, SubstrateExtrinsic } from "@subsquid/hydra-common";
import { Codec } from "@polkadot/types/types";
import { typeRegistry } from ".";

import { Compact } from "@polkadot/types";
import { Moment } from "@polkadot/types/interfaces";

export namespace Timestamp {
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
    public readonly extrinsic: SubstrateExtrinsic;
    public readonly expectedArgTypes = ["Compact<Moment>"];

    constructor(public readonly ctx: SubstrateEvent) {
      if (ctx.extrinsic === undefined) {
        throw new Error(`No call data has been provided`);
      }
      this.extrinsic = ctx.extrinsic;
    }

    get args(): Set_Args {
      return new Set_Args(this.extrinsic);
    }

    validateArgs(): boolean {
      if (this.expectedArgTypes.length !== this.extrinsic.args.length) {
        return false;
      }
      let valid = true;
      this.expectedArgTypes.forEach((type, i) => {
        if (type !== this.extrinsic.args[i].type) {
          valid = false;
        }
      });
      return valid;
    }
  }

  class Set_Args {
    constructor(public readonly extrinsic: SubstrateExtrinsic) {}

    get now(): Compact<Moment> {
      return createTypeUnsafe<Compact<Moment> & Codec>(
        typeRegistry,
        "Compact<Moment>",
        [this.extrinsic.args[0].value]
      );
    }
  }
}
