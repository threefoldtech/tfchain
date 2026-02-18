# 25. V3 Billing Opt-Out for Node Migration

Date: 2026-02-18

## Status

Accepted

## Context

During the migration from v3 to Mycelium, farmers need a way to signal that their nodes are entering a migration window. Existing workloads on these nodes should not be billed during this period, as the node is transitioning infrastructure.

## Decision

### New Storage Items (pallet-tfgrid)

- **`NodeV3BillingOptOut`**: A `StorageMap<node_id → opted_out_at (Unix seconds)>` tracking which nodes have opted out. Presence in this map is the sole indicator of opt-out status. No storage migration is required as this is an additive change.
- **`AllowedTwinAdmins`**: A `StorageValue<Vec<AccountId>>` listing accounts authorized to deploy on opted-out nodes.

### New Extrinsics (pallet-tfgrid)

| Extrinsic | Origin | Call Index |
|---|---|---|
| `opt_out_of_v3_billing(node_id)` | Farmer (signed) | 43 |
| `add_twin_admin(account)` | Council (`RestrictedOrigin`) | 44 |
| `remove_twin_admin(account)` | Council (`RestrictedOrigin`) | 45 |

Opt-out is one-way and permanent — there is no opt-back-in extrinsic. The `NodeV3BillingOptOut` entry is cleaned up automatically when the node is deleted.

### Deployment Guards (pallet-smart-contract)

`_create_node_contract` and `_create_rent_contract` both check `NodeV3BillingOptOut` before allowing deployment. If the target node has opted out, the caller's `AccountId` must be present in `AllowedTwinAdmins`, otherwise the call fails with `OnlyTwinAdminCanDeployOnThisNode`.

### Billing Suppression (pallet-smart-contract)

In `bill_contract`, a `should_waive_migration_billing` flag is computed by checking `NodeV3BillingOptOut` for the contract's node. When true:

- **`Created` state**: early return, no billing work performed.
- **`GracePeriod` state**: cost is zeroed; `manage_contract_state` runs normally, allowing the contract to be restored to `Created` once the user tops up to cover pre-opt-out overdraft.
- **`Deleted` state**: cost is zeroed; cleanup proceeds normally.

This is distinct from `should_waive_standby_rent` (standby power state, rent contracts only, emits `RentWaived`). The migration billing waiver is silent — no event is emitted because no billing is expected after opt-out.

### New Events (pallet-tfgrid)

- `NodeV3BillingOptedOut { node_id, opted_out_at }` — emitted on successful opt-out.
- `TwinAdminAdded(AccountId)` — emitted when an admin is added.
- `TwinAdminRemoved(AccountId)` — emitted when an admin is removed.

### New Errors

**pallet-smart-contract**

- `OnlyTwinAdminCanDeployOnThisNode` — returned when a non-admin attempts to deploy on an opted-out node.

**pallet-tfgrid**

- `NodeV3BillingOptOutAlreadyEnabled` — returned when `opt_out_of_v3_billing` is called on a node that has already opted out.
- `AlreadyTwinAdmin` — returned when `add_twin_admin` is called for an account already in the admin list.
- `NotTwinAdmin` — returned when `remove_twin_admin` is called for an account not in the admin list, or when the list is empty.

## Consequences

- **No storage migration**: all new storage items are additive.
- **Free migration window**: existing workloads on opted-out nodes accumulate no new charges. Users with pre-existing overdraft (contracts in `GracePeriod`) can still top up to restore their workloads, after which subsequent billing cycles are free.
- **Access control**: only council-approved twin admins can deploy new workloads on opted-out nodes.
