import { EventContext, StoreContext } from '@subsquid/hydra-common';
export declare function entityStored({ store, event, block, extrinsic, }: EventContext & StoreContext): Promise<void>;
export declare function entityUpdated({ store, event, block, extrinsic, }: EventContext & StoreContext): Promise<void>;
export declare function entityDeleted({ store, event, block, extrinsic, }: EventContext & StoreContext): Promise<void>;
