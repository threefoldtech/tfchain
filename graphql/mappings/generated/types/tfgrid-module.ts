import { createTypeUnsafe } from "@polkadot/types/create";
import { SubstrateEvent, SubstrateExtrinsic } from "@subsquid/hydra-common";
import { Codec } from "@polkadot/types/types";
import { typeRegistry } from ".";

import { Bytes, u32 } from "@polkadot/types";
import { AccountId } from "@polkadot/types/interfaces";
import {
  CertificationType,
  Location,
  Resources,
  Role
} from "substrate-tfgrid-ts-types";

export namespace TfgridModule {
  export class EntityStoredEvent {
    public readonly expectedParamTypes = [
      "u32",
      "u32",
      "Vec<u8>",
      "u32",
      "u32",
      "AccountId"
    ];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [u32, u32, Bytes, u32, u32, AccountId] {
      return [
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[0].value
        ]),
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[1].value
        ]),
        createTypeUnsafe<Bytes & Codec>(typeRegistry, "Bytes", [
          this.ctx.params[2].value
        ]),
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[3].value
        ]),
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[4].value
        ]),
        createTypeUnsafe<AccountId & Codec>(typeRegistry, "AccountId", [
          this.ctx.params[5].value
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

  export class EntityUpdatedEvent {
    public readonly expectedParamTypes = [
      "u32",
      "Vec<u8>",
      "u32",
      "u32",
      "AccountId"
    ];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [u32, Bytes, u32, u32, AccountId] {
      return [
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[0].value
        ]),
        createTypeUnsafe<Bytes & Codec>(typeRegistry, "Bytes", [
          this.ctx.params[1].value
        ]),
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[2].value
        ]),
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[3].value
        ]),
        createTypeUnsafe<AccountId & Codec>(typeRegistry, "AccountId", [
          this.ctx.params[4].value
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

  export class EntityDeletedEvent {
    public readonly expectedParamTypes = ["u32"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [u32] {
      return [
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
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

  export class FarmStoredEvent {
    public readonly expectedParamTypes = [
      "u32",
      "u32",
      "Vec<u8>",
      "u32",
      "u32",
      "u32",
      "u32",
      "types::CertificationType"
    ];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [u32, u32, Bytes, u32, u32, u32, u32, CertificationType] {
      return [
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[0].value
        ]),
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[1].value
        ]),
        createTypeUnsafe<Bytes & Codec>(typeRegistry, "Bytes", [
          this.ctx.params[2].value
        ]),
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[3].value
        ]),
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[4].value
        ]),
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[5].value
        ]),
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[6].value
        ]),
        createTypeUnsafe<CertificationType & Codec>(
          typeRegistry,
          "CertificationType",
          [this.ctx.params[7].value]
        )
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

  export class NodeStoredEvent {
    public readonly expectedParamTypes = [
      "u32",
      "u32",
      "u32",
      "types::Resources",
      "types::Location",
      "u32",
      "u32",
      "Vec<u8>",
      "AccountId",
      "types::Role"
    ];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [
      u32,
      u32,
      u32,
      Resources,
      Location,
      u32,
      u32,
      Bytes,
      AccountId,
      Role
    ] {
      return [
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[0].value
        ]),
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[1].value
        ]),
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[2].value
        ]),
        createTypeUnsafe<Resources & Codec>(typeRegistry, "Resources", [
          this.ctx.params[3].value
        ]),
        createTypeUnsafe<Location & Codec>(typeRegistry, "Location", [
          this.ctx.params[4].value
        ]),
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[5].value
        ]),
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[6].value
        ]),
        createTypeUnsafe<Bytes & Codec>(typeRegistry, "Bytes", [
          this.ctx.params[7].value
        ]),
        createTypeUnsafe<AccountId & Codec>(typeRegistry, "AccountId", [
          this.ctx.params[8].value
        ]),
        createTypeUnsafe<Role & Codec>(typeRegistry, "Role", [
          this.ctx.params[9].value
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

  export class NodeDeletedEvent {
    public readonly expectedParamTypes = ["u32"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [u32] {
      return [
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
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

  export class TwinStoredEvent {
    public readonly expectedParamTypes = ["u32", "u32", "AccountId", "Vec<u8>"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [u32, u32, AccountId, Bytes] {
      return [
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[0].value
        ]),
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[1].value
        ]),
        createTypeUnsafe<AccountId & Codec>(typeRegistry, "AccountId", [
          this.ctx.params[2].value
        ]),
        createTypeUnsafe<Bytes & Codec>(typeRegistry, "Bytes", [
          this.ctx.params[3].value
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

  export class TwinDeletedEvent {
    public readonly expectedParamTypes = ["u32"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [u32] {
      return [
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
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

  export class TwinEntityStoredEvent {
    public readonly expectedParamTypes = ["u32", "u32", "Vec<u8>"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [u32, u32, Bytes] {
      return [
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[0].value
        ]),
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[1].value
        ]),
        createTypeUnsafe<Bytes & Codec>(typeRegistry, "Bytes", [
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

  export class TwinEntityRemovedEvent {
    public readonly expectedParamTypes = ["u32", "u32"];

    constructor(public readonly ctx: SubstrateEvent) {}

    get params(): [u32, u32] {
      return [
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[0].value
        ]),
        createTypeUnsafe<u32 & Codec>(typeRegistry, "u32", [
          this.ctx.params[1].value
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
