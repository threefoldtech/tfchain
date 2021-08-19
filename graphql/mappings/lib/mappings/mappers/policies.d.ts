import { EventContext, StoreContext } from '@subsquid/hydra-common';
export declare function pricingPolicyStored({ store, event, block, extrinsic, }: EventContext & StoreContext): Promise<void>;
export declare function farmingPolicyStored({ store, event, block, extrinsic, }: EventContext & StoreContext): Promise<void>;
