# 13. Remove sudo functionality

Date: 2023-07-10

## Status

Accepted

## Context

We want to remove the sudo functionality from the chain. The main reason being security and not having a "root" key that can do anything on the chain.

Anything meaning: transfers, creating new accounts, changing the chain configuration, etc.

## Decision

Remove Sudo functionality from the chain and allow the `Council` (a group of ppl) to have elevated privileges to do a set of actions.

These actions are:

- Upgrading the runtime
- Changing the chain configuration
- Adding / Removing new council members
- ..

An important note here is that the council origin is not "root", so the council can only make actions that are allowed by specific pallets.

For example see `type SetCodeOrigin: EnsureOrigin<Self::RuntimeOrigin>;` in RuntimeUpgrade pallet. This origin is defined in the runtime (runtime/lib.rs) and can only be called by the council.
