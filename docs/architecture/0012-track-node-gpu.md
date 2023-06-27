# 12. Track if node has GPU

Date: 2023-06-22

## Status

Accepted

## Context

See [here](https://github.com/threefoldtech/tfchain/issues/737) for more details.

## Decision

Added a new storage map `NodeGpuStatus` on `pallet-tfgrid` that informs, using a boolean, if there is GPU capacity available on node.
It can be set by the node (ZOS) by calling `set_node_gpu_status`.
Since ZOS is able to identify if there are some GPU devices on the node, it can communicate it to the chain so we can track if a node has GPU capacity or not.
Note that this storage is temporary since we will soon have an indexer (under development) that will provide further information on nodes GPU.
