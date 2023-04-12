# 3. Third party service contract

Date: 2022-10-17

## Status

Accepted

## Context

See [here](https://github.com/threefoldtech/tfchain/issues/445) for more details.

## Decision

The third party service contract flow is described [here](../../substrate-node/pallets/pallet-smart-contract/third-party_service_contract.md#flow).

## Consequences

### the good

- It is now possible to create generic contract between two `TFChain` users (without restriction of account type) for some service and bill for it.

### the worrying

- Keep eyes on potential abuses and be prepared to handle all the malicious cases that can show up.