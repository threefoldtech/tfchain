# 7. Allow stash account for twin

Date: 2023-03-10

## Status

Accepted

## Context

See [here](https://github.com/threefoldtech/tfchain/issues/636) for more details.

## Decision

Allow a twin to specify an address of an optional "savings/stash account".
We added on tfchain:

- An optional storage map on pallet-tfgrid `tfgridModule.twin_bonded_account(u32) Option<AccountId32>` to map a twin_id and a savings account.
- A call on pallet-tfgrid `tfgridModule.bound_twin_account(twin_id)` where the twin id and the savings account are inserted into the storage; the savings account should initiate the bond between the twin and the account address.

Moreover, we take into account this new optional stash balance to calculate the discount level that can be applied on a node contract in pallet-smart-contract.
