# 10. Billing loop insertion/removal for smart contracts

Date: 2023-05-26

## Status

Accepted

## Context

While creating a contract it is inserted in billing loop at a specific index.
It allows the contract to be billed at determined blocks that respect a constant frequency called `billing_frequency`.
The index in billing loop is actually set to `current_block % billing_frequency`.
Since `billing_frequency = 600 blocks` the index belongs to `[0; 599]` range.
The issue is that, given a contract, once created and inserted in billing loop, we cannot retrieve at which index it was inserted in billing loop.
This led to a contract removal process in 2 steps at 2 different blocks: (1) remove from `Contracts`; and (2) remove from `ContractsToBillAt` at next cycle, which is not ideal since `Contracts` and `ContractsToBillAt` storages are not synchronized.

See also [here](https://github.com/threefoldtech/tfchain/issues/709) for more details.

## Decision

1. Billing loop insertion:
A solution to this issue is inserting contract in billing loop at index `contract_id % billing_frequency`.
The index will still continue in `[0; 599]` range but will be easy to get starting from the contract object since it always carries its unique id.  

2. Billing loop removal:
Thanks to the new insertion rule we don't need anymore to put responsability on the billing process to remove contract from the loop (and wait for extra billing cycle after contract removal to be able to also remove it from the billing loop).
When a contract get removed (after grace period ended, node deletion or contract cancelation) we now directly remove it from the billing loop too.
This simplifies the billing process by keeping synchronized `Contracts` and `ContractsToBillAt` storages.