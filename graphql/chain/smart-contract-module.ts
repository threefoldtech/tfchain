import { createTypeUnsafe } from "@polkadot/types/create";
import { SubstrateEvent, SubstrateExtrinsic } from "@subsquid/hydra-common";
import { Codec } from "@polkadot/types/types";
import { typeRegistry } from ".";

import { Consumption, Contract, ContractBill } from "substrate-tfgrid-ts-types";
import { u64 } from "@polkadot/types";

export namespace SmartContractModule {
  export class ConsumptionReportReceivedEvent {
    public readonly expectedParamTypes = ["types::Consumption"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [Consumption] {
      return [
        (createTypeUnsafe<Consumption & Codec>(typeRegistry, "Consumption", [
          this.ctx.params[0].value,
        ]) as any) as Consumption,
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
    public readonly expectedParamTypes = ["types::Contract"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [Contract] {
      return [
        (createTypeUnsafe<Contract & Codec>(typeRegistry, "Contract", [
          this.ctx.params[0].value,
        ]) as any) as Contract,
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
    public readonly expectedParamTypes = ["types::Contract"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [Contract] {
      return [
        (createTypeUnsafe<Contract & Codec>(typeRegistry, "Contract", [
          this.ctx.params[0].value,
        ]) as any) as Contract,
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
          this.ctx.params[0].value,
        ]),
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
    public readonly expectedParamTypes = ["types::ContractBill"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [ContractBill] {
      return [
        (createTypeUnsafe<ContractBill & Codec>(typeRegistry, "ContractBill", [
          this.ctx.params[0].value,
        ]) as any) as ContractBill,
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
