# 16. Add condition to DAO motion approval

Date: 2023-11-07

## Status

Accepted

## Context

See [here](https://github.com/threefoldtech/tfchain/issues/803) for more
details.

## Decision

Stick to specs initial idea by adding new condition
`number of votes >= threshold` for proposal to be approved. If by the end of the
vote the minimal amount of votes is not reached, the proposal fails due to
insufficient interest.
