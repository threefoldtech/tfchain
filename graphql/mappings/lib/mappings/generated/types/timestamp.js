"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Timestamp = void 0;
const create_1 = require("@polkadot/types/create");
const _1 = require(".");
var Timestamp;
(function (Timestamp) {
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
    class SetCall {
        constructor(ctx) {
            this.ctx = ctx;
            this.expectedArgTypes = ["Compact<Moment>"];
            if (ctx.extrinsic === undefined) {
                throw new Error(`No call data has been provided`);
            }
            this.extrinsic = ctx.extrinsic;
        }
        get args() {
            return new Set_Args(this.extrinsic);
        }
        validateArgs() {
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
    Timestamp.SetCall = SetCall;
    class Set_Args {
        constructor(extrinsic) {
            this.extrinsic = extrinsic;
        }
        get now() {
            return create_1.createTypeUnsafe(_1.typeRegistry, "Compact<Moment>", [this.extrinsic.args[0].value]);
        }
    }
})(Timestamp = exports.Timestamp || (exports.Timestamp = {}));
