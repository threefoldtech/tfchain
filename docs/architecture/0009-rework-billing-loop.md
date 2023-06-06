# 9. Rework smart contract billing loop

Date: 2023-05-23

## Status

Accepted

## Context

See [here](https://github.com/threefoldtech/tfchain/issues/701) for more details.

## Decision

(1) Each contract (node/rent/name contracts) is inserted in billing loop only once, when contract is created.
There is no more exception for node contracts with no public ips and no used resources on chain.

(2) When a contract is to be billed during the billing loop execution (via the offchain worker), bill it ONLY if there is effectively some IP/SU/CU/NU consumed. Technically this comes down to trigger `bill_contract()` extrinsic only when there is some used ip / resources or consumption reports linked to this contract.