# 11. Dedicated nodes extra fee

Date: 2023-06-06

## Status

Accepted

## Context

For supporting GPU on the grid (https://github.com/threefoldtech/home/issues/1392) we are adding a the option for a farmer to set an extra fee for his nodes that have a GPU or other special feautures enabled. This fee will be added to the price of the capacity price. The full fee will be paid to the farmer.

By setting a price on a node, this node becomes dedicated and can only be rented by creating a `RentContract` on that node. This means that when a farmer sets an extra price on a node, this node cannot be shared anymore.

## Decision

Added a new storage map `DedicatedNodesExtraFee` on `pallet-smart-contract` which can be set by the owner of that node (the farmer) by calling `set_dedicated_node_extra_fee`. 
The input of this call is `node_id` and `fee` which is expressed in mUSD (USD * 1000). This fee is expressed in a fee per month.

We also changed the `ContractLock` storage to include a new field `extra_amount_locked` which is the total fee that is locked by this contract.