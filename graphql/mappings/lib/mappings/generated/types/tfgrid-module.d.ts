import { SubstrateEvent } from "@subsquid/hydra-common";
import { Entity, Farm, Node, Twin } from "substrate-tfgrid-ts-types";
import { Bytes, u32 } from "@polkadot/types";
export declare namespace TfgridModule {
    class EntityStoredEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [Entity];
        validateParams(): boolean;
    }
    class EntityUpdatedEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [Entity];
        validateParams(): boolean;
    }
    class EntityDeletedEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [u32];
        validateParams(): boolean;
    }
    class FarmStoredEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [Farm];
        validateParams(): boolean;
    }
    class FarmUpdatedEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [Farm];
        validateParams(): boolean;
    }
    class FarmDeletedEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [u32];
        validateParams(): boolean;
    }
    class NodeStoredEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [Node];
        validateParams(): boolean;
    }
    class NodeUpdatedEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [Node];
        validateParams(): boolean;
    }
    class NodeDeletedEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [u32];
        validateParams(): boolean;
    }
    class TwinStoredEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [Twin];
        validateParams(): boolean;
    }
    class TwinDeletedEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [u32];
        validateParams(): boolean;
    }
    class TwinEntityStoredEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [u32, u32, Bytes];
        validateParams(): boolean;
    }
    class TwinEntityRemovedEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [u32, u32];
        validateParams(): boolean;
    }
}
