# 16. Prevent DAO proposal from duplicate veto

Date: 2023-11-06

## Status

Accepted

## Context

See [here](https://github.com/threefoldtech/tfchain/issues/884) for more details.

## Decision

In `pallet-smart-contract`, `cancel_contract()` extrinsic was only authorized to be executed by the node (ZOS) the contract was on.
We now allow a collective approval (council or farmers) or a single council member to cancel a contract.
For this purpose `Cause` enum was extended to better define the cancelation causes.
