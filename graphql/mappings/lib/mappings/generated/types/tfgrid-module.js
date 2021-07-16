"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TfgridModule = void 0;
const create_1 = require("@polkadot/types/create");
const _1 = require(".");
var TfgridModule;
(function (TfgridModule) {
    class EntityStoredEvent {
        constructor(ctx) {
            this.ctx = ctx;
            this.expectedParamTypes = ["types::Entity<AccountId>"];
        }
        get params() {
            return [
                create_1.createTypeUnsafe(_1.typeRegistry, "Entity", [
                    this.ctx.params[0].value
                ])
            ];
        }
        validateParams() {
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
    TfgridModule.EntityStoredEvent = EntityStoredEvent;
    class EntityUpdatedEvent {
        constructor(ctx) {
            this.ctx = ctx;
            this.expectedParamTypes = ["types::Entity<AccountId>"];
        }
        get params() {
            return [
                create_1.createTypeUnsafe(_1.typeRegistry, "Entity", [
                    this.ctx.params[0].value
                ])
            ];
        }
        validateParams() {
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
    TfgridModule.EntityUpdatedEvent = EntityUpdatedEvent;
    class EntityDeletedEvent {
        constructor(ctx) {
            this.ctx = ctx;
            this.expectedParamTypes = ["u32"];
        }
        get params() {
            return [
                create_1.createTypeUnsafe(_1.typeRegistry, "u32", [
                    this.ctx.params[0].value
                ])
            ];
        }
        validateParams() {
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
    TfgridModule.EntityDeletedEvent = EntityDeletedEvent;
    class FarmStoredEvent {
        constructor(ctx) {
            this.ctx = ctx;
            this.expectedParamTypes = ["types::Farm"];
        }
        get params() {
            return [
                create_1.createTypeUnsafe(_1.typeRegistry, "Farm", [
                    this.ctx.params[0].value
                ])
            ];
        }
        validateParams() {
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
    TfgridModule.FarmStoredEvent = FarmStoredEvent;
    class FarmUpdatedEvent {
        constructor(ctx) {
            this.ctx = ctx;
            this.expectedParamTypes = ["types::Farm"];
        }
        get params() {
            return [
                create_1.createTypeUnsafe(_1.typeRegistry, "Farm", [
                    this.ctx.params[0].value
                ])
            ];
        }
        validateParams() {
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
    TfgridModule.FarmUpdatedEvent = FarmUpdatedEvent;
    class FarmDeletedEvent {
        constructor(ctx) {
            this.ctx = ctx;
            this.expectedParamTypes = ["u32"];
        }
        get params() {
            return [
                create_1.createTypeUnsafe(_1.typeRegistry, "u32", [
                    this.ctx.params[0].value
                ])
            ];
        }
        validateParams() {
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
    TfgridModule.FarmDeletedEvent = FarmDeletedEvent;
    class NodeStoredEvent {
        constructor(ctx) {
            this.ctx = ctx;
            this.expectedParamTypes = ["types::Node<AccountId>"];
        }
        get params() {
            return [
                create_1.createTypeUnsafe(_1.typeRegistry, "Node", [
                    this.ctx.params[0].value
                ])
            ];
        }
        validateParams() {
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
    TfgridModule.NodeStoredEvent = NodeStoredEvent;
    class NodeUpdatedEvent {
        constructor(ctx) {
            this.ctx = ctx;
            this.expectedParamTypes = ["types::Node<AccountId>"];
        }
        get params() {
            return [
                create_1.createTypeUnsafe(_1.typeRegistry, "Node", [
                    this.ctx.params[0].value
                ])
            ];
        }
        validateParams() {
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
    TfgridModule.NodeUpdatedEvent = NodeUpdatedEvent;
    class NodeDeletedEvent {
        constructor(ctx) {
            this.ctx = ctx;
            this.expectedParamTypes = ["u32"];
        }
        get params() {
            return [
                create_1.createTypeUnsafe(_1.typeRegistry, "u32", [
                    this.ctx.params[0].value
                ])
            ];
        }
        validateParams() {
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
    TfgridModule.NodeDeletedEvent = NodeDeletedEvent;
    class TwinStoredEvent {
        constructor(ctx) {
            this.ctx = ctx;
            this.expectedParamTypes = ["types::Twin<AccountId>"];
        }
        get params() {
            return [
                create_1.createTypeUnsafe(_1.typeRegistry, "Twin", [
                    this.ctx.params[0].value
                ])
            ];
        }
        validateParams() {
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
    TfgridModule.TwinStoredEvent = TwinStoredEvent;
    class TwinDeletedEvent {
        constructor(ctx) {
            this.ctx = ctx;
            this.expectedParamTypes = ["u32"];
        }
        get params() {
            return [
                create_1.createTypeUnsafe(_1.typeRegistry, "u32", [
                    this.ctx.params[0].value
                ])
            ];
        }
        validateParams() {
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
    TfgridModule.TwinDeletedEvent = TwinDeletedEvent;
    class TwinEntityStoredEvent {
        constructor(ctx) {
            this.ctx = ctx;
            this.expectedParamTypes = ["u32", "u32", "Vec<u8>"];
        }
        get params() {
            return [
                create_1.createTypeUnsafe(_1.typeRegistry, "u32", [
                    this.ctx.params[0].value
                ]),
                create_1.createTypeUnsafe(_1.typeRegistry, "u32", [
                    this.ctx.params[1].value
                ]),
                create_1.createTypeUnsafe(_1.typeRegistry, "Bytes", [
                    this.ctx.params[2].value
                ])
            ];
        }
        validateParams() {
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
    TfgridModule.TwinEntityStoredEvent = TwinEntityStoredEvent;
    class TwinEntityRemovedEvent {
        constructor(ctx) {
            this.ctx = ctx;
            this.expectedParamTypes = ["u32", "u32"];
        }
        get params() {
            return [
                create_1.createTypeUnsafe(_1.typeRegistry, "u32", [
                    this.ctx.params[0].value
                ]),
                create_1.createTypeUnsafe(_1.typeRegistry, "u32", [
                    this.ctx.params[1].value
                ])
            ];
        }
        validateParams() {
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
    TfgridModule.TwinEntityRemovedEvent = TwinEntityRemovedEvent;
})(TfgridModule = exports.TfgridModule || (exports.TfgridModule = {}));
