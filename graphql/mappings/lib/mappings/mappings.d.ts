import { ExtrinsicContext, EventContext, StoreContext } from '@subsquid/hydra-common';
export declare function balancesTransfer({ store, event, block, extrinsic, }: EventContext & StoreContext): Promise<void>;
export declare function timestampCall({ store, event, block, }: ExtrinsicContext & StoreContext): Promise<void>;
