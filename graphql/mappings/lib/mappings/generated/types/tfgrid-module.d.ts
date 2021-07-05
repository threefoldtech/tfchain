import { SubstrateEvent } from "@subsquid/hydra-common";
import { Bytes, Option, u32 } from "@polkadot/types";
import { AccountId } from "@polkadot/types/interfaces";
import { CertificationType, Location, PublicConfig, Resources, Role } from "substrate-tfgrid-ts-types";
export declare namespace TfgridModule {
    class EntityStoredEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [u32, u32, Bytes, u32, u32, AccountId];
        validateParams(): boolean;
    }
    class EntityUpdatedEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [u32, Bytes, u32, u32, AccountId];
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
        get params(): [u32, u32, Bytes, u32, u32, u32, u32, CertificationType];
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
        get params(): [u32, u32, u32, Resources, Location, u32, u32, AccountId, Role, u32, Option<PublicConfig>];
        validateParams(): boolean;
    }
    class NodeUpdatedEvent {
        readonly ctx: SubstrateEvent;
        readonly expectedParamTypes: string[];
        constructor(ctx: SubstrateEvent);
        get params(): [u32, u32, u32, Resources, Location, u32, u32, AccountId, Role, u32, Option<PublicConfig>];
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
        get params(): [u32, u32, AccountId, Bytes];
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
