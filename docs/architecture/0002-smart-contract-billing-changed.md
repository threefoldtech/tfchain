# 2. Changed smart contract billing workflow

Date: 2022-10-12

## Status

Accepted

## Context

TFchain bills contract using an internal feature of susbtrate called `on_finalize`. This function / hook is executed after every block, in this function the time to compute is limited since this runs before the block production is finished. We are worried that if the usage of the grid grows, this on finalize method would need to do too much computations and that the block time would be impacted.

## Decision

We searched for an alternative to this `on_finalize` method within the substrate framework. We found a hook that fires after a block is produced, but in that hook the result of some computation must be submitted onchain with an extrinsic. This offchain worker cannot modify chain storage directly, rather it can only do that through an extrinsic.

## Consequences

### the good

- The billing for a contract is now executed after a block is produced and not within that same block.
- Atleast one offchain worker must run with the `smct` keytype in order to bill contracts.
- Contracts billing cycles do not rely anymore on a previously successful bill.
- External users can call `bill_contract` but don't need to.

### the worrying

- If no validators run an offchain worker with the `smct` keytype, no contracts will be billed.
