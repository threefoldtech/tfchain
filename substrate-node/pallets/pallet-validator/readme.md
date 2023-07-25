# Validator addition module

A pallet that manages external parties to join Threefold DAO & become a consensus validator (aura/gran).

See: [flow diagram](./validator_request_flow.md)

## Overview

The Validator addition module provides functions for:

- Create a validator request
- Activating a validator node
- Changing a validator node account
- Bonding a validator account to prove that account is indeed in control of the valiadtor node
- Approving a validator to be added as a council member and to participate in consensus
- Removing a validator node

## Terminology

- Validator request: A request to add a validator node to the Threefold DAO
- Validator node account: The account that is used to sign blocks and participate in consensus
- Stash account: The account that is used to bond tokens to the validator node account

See [spec](./spec.md) for more details.

## Interface

Dispatchable functions of this pallet.

- `create_validator_request`: Create a validator request
- `activate_validator_node`: Activate a validator node
- `change_validator_node_account`: Change the validator node account
- `bond`: Bond a validator node account
- `approve_validator`: Approve a validator node to be added as a council member and to participate in consensus, can only by called by a council member
- `remove_validator_node`: Remove a validator node, can only by called by a council member