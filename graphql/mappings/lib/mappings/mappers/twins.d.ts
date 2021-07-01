import { EventContext, StoreContext } from '@subsquid/hydra-common';
export declare function twinStored({ store, event, block, extrinsic, }: EventContext & StoreContext): Promise<void>;
export declare function twinDeleted({ store, event, block, extrinsic, }: EventContext & StoreContext): Promise<void>;
export declare function twinEntityStored({ store, event, block, extrinsic, }: EventContext & StoreContext): Promise<void>;
export declare function twinEntityRemoved({ store, event, block, extrinsic, }: EventContext & StoreContext): Promise<void>;
