# 17. Allow collective approval to cancel contracts

Date: 2023-11-06

## Status

Accepted

## Context

See [here](https://github.com/threefoldtech/tfchain/issues/884) for more details.

## Decision

In `pallet-smart-contract`, add `cancel_contract_collective()` extrinsic to allow a collective approval (council or farmers) to cancel a contract.
For this purpose we also add a new entry `CanceledByCollective` in `Cause` enum to better qualify the cancelation cause.
