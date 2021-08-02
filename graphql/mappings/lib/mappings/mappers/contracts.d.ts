import { EventContext, StoreContext } from '@subsquid/hydra-common';
export declare function consumptionReportReceived({ store, event, block, extrinsic, }: EventContext & StoreContext): Promise<void>;
