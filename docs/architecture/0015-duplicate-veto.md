# 15. Prevent DAO proposal from duplicate veto

Date: 2023-09-14

## Status

Accepted

## Context

See [here](https://github.com/threefoldtech/tfchain/issues/858) for more details.

## Decision

Since we don't want a council member to veto a DAO proposal alone by being able to submit its veto more than once we need to prevent duplicate veto.
In `pallet-dao`, `veto()` extrinsic should return an appropriate error while this situation occures.
