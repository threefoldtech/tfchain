import { createTypeUnsafe } from "@polkadot/types/create";
import { SubstrateEvent, SubstrateExtrinsic } from "@subsquid/hydra-common";
import { Codec } from "@polkadot/types/types";
import { typeRegistry } from ".";

import {
  Consumption,
  DiscountLevel,
  NodeContract
} from "substrate-tfgrid-ts-types";
import { u128, u64 } from "@polkadot/types";

export namespace SmartContractModule {
  export class ConsumptionReportReceivedEvent {
    public readonly expectedParamTypes = ["types::Consumption"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [Consumption] {
      return [
        createTypeUnsafe<Consumption & Codec>(typeRegistry, "Consumption", [
          this.ctx.params[0].value
        ])
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

  export class ContractCreatedEvent {
    public readonly expectedParamTypes = ["types::NodeContract"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [NodeContract] {
      return [
        createTypeUnsafe<NodeContract & Codec>(typeRegistry, "NodeContract", [
          this.ctx.params[0].value
        ])
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

  export class ContractUpdatedEvent {
    public readonly expectedParamTypes = ["types::NodeContract"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [NodeContract] {
      return [
        createTypeUnsafe<NodeContract & Codec>(typeRegistry, "NodeContract", [
          this.ctx.params[0].value
        ])
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

  export class ContractCanceledEvent {
    public readonly expectedParamTypes = ["u64"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [u64] {
      return [
        createTypeUnsafe<u64 & Codec>(typeRegistry, "u64", [
          this.ctx.params[0].value
        ])
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

  export class ContractBilledEvent {
    public readonly expectedParamTypes = [
      "u64",
      "types::DiscountLevel",
      "u128"
    ];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [u64, DiscountLevel, u128] {
      return [
        createTypeUnsafe<u64 & Codec>(typeRegistry, "u64", [
          this.ctx.params[0].value
        ]),
        createTypeUnsafe<DiscountLevel & Codec>(typeRegistry, "DiscountLevel", [
          this.ctx.params[1].value
        ]),
        createTypeUnsafe<u128 & Codec>(typeRegistry, "u128", [
          this.ctx.params[2].value
        ])
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
