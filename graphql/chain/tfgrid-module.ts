import { createTypeUnsafe } from "@polkadot/types/create";
import { SubstrateEvent, SubstrateExtrinsic } from "@subsquid/hydra-common";
import { Codec } from "@polkadot/types/types";
import { typeRegistry } from ".";

import {
  Entity,
  Farm,
  FarmingPolicy,
  Node,
  PricingPolicy,
  PublicConfig,
  Twin,
} from "substrate-tfgrid-ts-types";
import { Bytes, u32, u64 } from "@polkadot/types";

export namespace TfgridModule {
  export class EntityStoredEvent {
    public readonly expectedParamTypes = ["types::Entity<AccountId>"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [Entity] {
      return [
        createTypeUnsafe<Entity & Codec>(typeRegistry, "Entity", [
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

  export class EntityUpdatedEvent {
    public readonly expectedParamTypes = ["types::Entity<AccountId>"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [Entity] {
      return [
        createTypeUnsafe<Entity & Codec>(typeRegistry, "Entity", [
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

  export class EntityDeletedEvent {
    public readonly expectedParamTypes = ["u32"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [u32] {
      return [
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
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

  export class FarmStoredEvent {
    public readonly expectedParamTypes = ["types::Farm"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [Farm] {
      return [
        createTypeUnsafe<Farm & Codec>(typeRegistry, "Farm", [
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

  export class FarmUpdatedEvent {
    public readonly expectedParamTypes = ["types::Farm"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [Farm] {
      return [
        createTypeUnsafe<Farm & Codec>(typeRegistry, "Farm", [
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

  export class FarmDeletedEvent {
    public readonly expectedParamTypes = ["u32"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [u32] {
      return [
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
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

  export class NodeStoredEvent {
    public readonly expectedParamTypes = ["types::Node"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [Node] {
      return [
        createTypeUnsafe<Node & Codec>(typeRegistry, "Node", [
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

  export class NodeUpdatedEvent {
    public readonly expectedParamTypes = ["types::Node"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [Node] {
      return [
        createTypeUnsafe<Node & Codec>(typeRegistry, "Node", [
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

  export class NodeDeletedEvent {
    public readonly expectedParamTypes = ["u32"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [u32] {
      return [
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
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

  export class NodeUptimeReportedEvent {
    public readonly expectedParamTypes = ["u32", "u64", "u64"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [u32, u64, u64] {
      return [
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[0].value,
        ]),
        createTypeUnsafe<u64 & Codec>(typeRegistry, "u64", [
          this.ctx.params[1].value,
        ]),
        createTypeUnsafe<u64 & Codec>(typeRegistry, "u64", [
          this.ctx.params[2].value,
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

  export class TwinStoredEvent {
    public readonly expectedParamTypes = ["types::Twin<AccountId>"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [Twin] {
      return [
        createTypeUnsafe<Twin & Codec>(typeRegistry, "Twin", [
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

  export class TwinDeletedEvent {
    public readonly expectedParamTypes = ["u32"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [u32] {
      return [
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
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

  export class TwinEntityStoredEvent {
    public readonly expectedParamTypes = ["u32", "u32", "Vec<u8>"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [u32, u32, Bytes] {
      return [
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[0].value,
        ]),
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[1].value,
        ]),
        createTypeUnsafe<Bytes & Codec>(typeRegistry, "Bytes", [
          this.ctx.params[2].value,
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

  export class TwinEntityRemovedEvent {
    public readonly expectedParamTypes = ["u32", "u32"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [u32, u32] {
      return [
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[0].value,
        ]),
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[1].value,
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

  export class PricingPolicyStoredEvent {
    public readonly expectedParamTypes = ["types::PricingPolicy<AccountId>"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [PricingPolicy] {
      return [
        createTypeUnsafe<PricingPolicy & Codec>(typeRegistry, "PricingPolicy", [
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

  export class FarmingPolicyStoredEvent {
    public readonly expectedParamTypes = ["types::FarmingPolicy"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [FarmingPolicy] {
      return [
        createTypeUnsafe<FarmingPolicy & Codec>(typeRegistry, "FarmingPolicy", [
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

  export class FarmPayoutV2AddressRegisteredEvent {
    public readonly expectedParamTypes = ["u32", "Vec<u8>"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [u32, Bytes] {
      return [
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[0].value,
        ]),
        createTypeUnsafe<Bytes & Codec>(typeRegistry, "Bytes", [
          this.ctx.params[1].value,
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

  export class NodePublicConfigStoredEvent {
    public readonly expectedParamTypes = ["u32", "types::PublicConfig"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [u32, PublicConfig] {
      return [
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[0].value,
        ]),
        createTypeUnsafe<PublicConfig & Codec>(typeRegistry, "PublicConfig", [
          this.ctx.params[1].value,
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
}
