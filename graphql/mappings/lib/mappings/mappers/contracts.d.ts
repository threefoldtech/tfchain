import { EventContext, StoreContext } from '@subsquid/hydra-common';
export declare function ConsumptionReportReceived({ store, event, block, extrinsic, }: EventContext & StoreContext): Promise<void>;
