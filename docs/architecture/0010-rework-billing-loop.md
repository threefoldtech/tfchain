# 9. Rework smart contract billing loop

Date: 2023-05-26

## Status

Accepted

## Context

While creating a contract it is inserted in billing loop at a specific index.
It allows the contract to be billed at determined blocks that respect a constant frequency called billing_frequency.
The index in billing loop is actually set to current_block % billing_frequency.
The issue is that, given a contract, once created and inserted in billing loop, we cannot retrieve at which index it was inserted in billing loop.
Which is not convenient to be able to monitor consistency between Contracts and ContractsToBillAt storages.

See [here](https://github.com/threefoldtech/tfchain/issues/709) for more details.

## Decision

// TODO

(1) billing loop insertion
A solution to this issue would be inserting contract in billing loop at index contract_id % billing_frequency.

(2) billing loop removal 