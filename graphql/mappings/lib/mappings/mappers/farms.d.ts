import { EventContext, StoreContext } from '@subsquid/hydra-common';
export declare function farmStored({ store, event, block, extrinsic, }: EventContext & StoreContext): Promise<void>;
export declare function farmUpdated({ store, event, block, extrinsic, }: EventContext & StoreContext): Promise<void>;
export declare function farmDeleted({ store, event, block, extrinsic, }: EventContext & StoreContext): Promise<void>;
