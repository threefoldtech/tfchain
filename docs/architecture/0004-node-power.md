# 4: Node Power management

Date: 2023-01-26

## Status

Accepted

## Context

See [here](https://github.com/threefoldtech/farmerbot/issues/1) for more details.

## Decision

Node Power State and Power target were added to `pallet-tfgrid` to orchestrate the Farmerbot taking actions to do power management.
A farmer(bot) can choose to power down nodes by calling `change_power_target(node_id, power_target(Up/Down))`.
When a node actually is going to shut down because of the changed power target, it will set it's power state to `down` on chain by calling `change_power_state(PowerState(Up/Down))`.
Likewise, when the farmer(bot) changes back the power target of the node to Up, the node will change it's power state to `up` when it's fully booted again.

When creating contracts (rent/node) the power state is checked before the contract is created, thus preventing contracts to be created on Nodes that are shutdown physically.

## Consequences

### the good

- Contracts (rent/node) cannot be created anymore on nodes that have power state `Down`
- Power management is and opt-in feature with the farmerbot

### the worrying

- If the farmer(bot) sets a node's power target to `Down` and the node is in isolated network, the only way it can be turned on again is by setting the power target to `Up` and physically rebooting the node.
