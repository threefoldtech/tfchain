# 20. Harden closing of a DAO motion voting

Date: 2024-05-02

## Status

Accepted

## Context

See [here](https://github.com/threefoldtech/tfchain/issues/889) for more
details.

## Decision

In `propose()` extrinsic, set a minimum threshold `MotionMinThreshold` of 5
votes for a motion to be proposed and if threshold is lower then return error
`TresholdToLow`. Also in `propose()`extrinsic, add a minimum motion duration of
1 day and return error `InvalidProposalDuration` if optional duration is not set
in the 1 day - 30 days interval.
