import { EventContext, StoreContext } from '@subsquid/hydra-common';
export declare function consumptionReportReceived({ store, event, block, extrinsic, }: EventContext & StoreContext): Promise<void>;
export declare function contractCreated({ store, event, block, extrinsic, }: EventContext & StoreContext): Promise<void>;
export declare function contractUpdated({ store, event, block, extrinsic, }: EventContext & StoreContext): Promise<void>;
export declare function contractCanceled({ store, event, block, extrinsic, }: EventContext & StoreContext): Promise<void>;
export declare function contractBilled({ store, event, block, extrinsic, }: EventContext & StoreContext): Promise<void>;
