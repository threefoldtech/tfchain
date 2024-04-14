# 6. Allow attaching solution provider ID to existing contracts

Date: 2023-03-10

## Status

Accepted

## Context

See [here](https://github.com/threefoldtech/tfchain/issues/630) for more details.

## Decision

We added a call on tfchain:

- `SmartContractModule.attach_solution_provider_id(contract_id, solution_provider_id)`

This call adds a solution provider ID to a contract, only the owner of the contract can call this function.
You cannot override an existing solution provider ID.
