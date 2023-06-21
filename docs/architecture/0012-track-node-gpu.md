# 12. Track number of GPUs on a node 

Date: 2023-06-20

## Status

Accepted

## Context

See [here](https://github.com/threefoldtech/tfchain/issues/737) and [here](https://github.com/threefoldtech/tfchain/issues/756) for more details.

## Decision

Added a new storage map `NodeGpuNumber` on `pallet-tfgrid` that contains the number of GPUs (`u8`) a node has.
It can be set by the node (ZOS) by calling `set_node_gpu_number`.
Since ZOS is able to identify if there is some GPU capacity on the node, it can communicate it to the chain so we can track how many GPU devices a node has.