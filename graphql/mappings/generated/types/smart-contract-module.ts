import { createTypeUnsafe } from "@polkadot/types/create";
import { SubstrateEvent, SubstrateExtrinsic } from "@subsquid/hydra-common";
import { Codec } from "@polkadot/types/types";
import { typeRegistry } from ".";

import { Consumption } from "substrate-tfgrid-ts-types";

export namespace SmartContractModule {
  export class ConsumptionReportReceivedEvent {
    public readonly expectedParamTypes = ["Consumption"];

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
}
