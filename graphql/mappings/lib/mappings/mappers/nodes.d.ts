import { EventContext, StoreContext } from '@subsquid/hydra-common';
export declare function nodeStored({ store, event, block, extrinsic, }: EventContext & StoreContext): Promise<void>;
export declare function nodeDeleted({ store, event, block, extrinsic, }: EventContext & StoreContext): Promise<void>;
