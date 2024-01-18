# 18. Allow renting standby node

Date: 2024-01-09

## Status

Accepted

## Context

See [here](https://github.com/threefoldtech/tfchain/issues/923) for more details.

## Decision

We want to be able to create a rent contract on node even if it is in standby phase.
Moreover, user should be billed for this contract only during online periods.

In `pallet-smart-contract`:

*   Remove the `node_power.is_down()` restriction when trying to create a rent contract in `create_rent_contract()` extrinsic. restriction
*   Modify rent contract billing logic by allowing billing only if the node is online (`PowerState` = `Up`). To skip the billing during the standby period we update the contract lock when the node power state is switched to `Up`.
