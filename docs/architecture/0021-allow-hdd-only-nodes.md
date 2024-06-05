# 21. Allow 'only hdd' nodes to register on chain

Date: 2024-05-22

## Status

Accepted

## Context

See [here](https://github.com/threefoldtech/tfchain/issues/967) for more
details.

## Decision

Similar to what already exists for SSD, add a minimum HDD size requirement (also
100 GB) on `resources.validate_hru()`. Then, on validating, make sure node has
at least 1 minimum storage capacity available.
