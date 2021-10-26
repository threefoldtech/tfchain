import { createTypeUnsafe } from "@polkadot/types/create";
import { SubstrateEvent, SubstrateExtrinsic } from "@subsquid/hydra-common";
import { Codec } from "@polkadot/types/types";
import { typeRegistry } from ".";

import {
  BurnTransaction,
  MintTransaction,
  RefundTransaction,
} from "substrate-tfgrid-ts-types";

export namespace TFTBridgeModule {
  export class MintCompletedEvent {
    public readonly expectedParamTypes = [
      "MintTransaction<AccountId, BlockNumber>",
    ];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [MintTransaction] {
      return [
        createTypeUnsafe<MintTransaction & Codec>(
          typeRegistry,
          "MintTransaction",
          [this.ctx.params[0].value]
        ),
      ];
    }

    validateParams(): boolean {
      if (this.expectedParamTypes.length !== this.ctx.params.length) {
        return false;
      }
      let valid = true;
      this.expectedParamTypes.forEach((type, i) => {
        if (type !== this.ctx.params[i].type) {
          valid = false;
        }
      });
      return valid;
    }
  }

  export class BurnTransactionProcessedEvent {
    public readonly expectedParamTypes = ["BurnTransaction<BlockNumber>"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [BurnTransaction] {
      return [
        createTypeUnsafe<BurnTransaction & Codec>(
          typeRegistry,
          "BurnTransaction",
          [this.ctx.params[0].value]
        ),
      ];
    }

    validateParams(): boolean {
      if (this.expectedParamTypes.length !== this.ctx.params.length) {
        return false;
      }
      let valid = true;
      this.expectedParamTypes.forEach((type, i) => {
        if (type !== this.ctx.params[i].type) {
          valid = false;
        }
      });
      return valid;
    }
  }

  export class RefundTransactionProcessedEvent {
    public readonly expectedParamTypes = ["RefundTransaction<BlockNumber>"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [RefundTransaction] {
      return [
        createTypeUnsafe<RefundTransaction & Codec>(
          typeRegistry,
          "RefundTransaction",
          [this.ctx.params[0].value]
        ),
      ];
    }

    validateParams(): boolean {
      if (this.expectedParamTypes.length !== this.ctx.params.length) {
        return false;
      }
      let valid = true;
      this.expectedParamTypes.forEach((type, i) => {
        if (type !== this.ctx.params[i].type) {
          valid = false;
        }
      });
      return valid;
    }
  }
}
