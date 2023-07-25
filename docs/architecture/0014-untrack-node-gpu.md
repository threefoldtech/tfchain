# 14. Untrack GPU status on a node 

Date: 2023-06-22

## Status

Accepted

## Context

See [here](https://github.com/threefoldtech/tfchain/issues/759) for more details.

## Decision

Since we don't want to store non-essential data on chain and GPU information is not of any actual value to the chain or minting, the storage map `NodeGpuStatus` (see ./0012-track-node-gpu.md) is removed from `pallet-tfgrid`.
An off chain indexer will handle a node query to fetch further detailed information about nodes GPU (number / models / memory capacity / ...).