# 19. Rework change power target on node

Date: 2024-01-09

## Status

Accepted

## Context

See [here](https://github.com/threefoldtech/tfchain/issues/924) for more details.

## Decision

Make sure that node has no active contracts on it to be able to change its `PowerTarget` to `Down` when calling `change_power_target()` extrinsic.
