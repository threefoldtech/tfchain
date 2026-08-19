# 27. Migrate a Node Contract to Another Node in the Same Farm

Date: 2026-08-18

## Status

Accepted

## Context

A farmer cannot power a node down while it holds a single active contract:
`change_power_target(Down)` requires `node_has_no_active_contracts`
(`pallet-tfgrid/src/node.rs:373-378`). The only way to empty a node today is to
cancel its contracts, which is terminal — the contract is gone, and with it any
path for the tenant to return under the same identity.

That blocks hardware consolidation. On mainnet at spec 157, emptying ten machines
in Freefarm means relocating 74 contracts across 29 twins, none of them the
operator; retiring the 2012 generation means 122. Asking those twins to
recreate contracts by hand is not available to a hosting provider.

Two alternatives were rejected. **Cancel and recreate** is the problem, not the
fix. **Off-chain coordination** offload the responsibility to the tenant, which
is not feasible.

### What the chain does and does not guarantee

**Migration moves the booking, not the bytes.** The chain stores a contract ID, a
deployment hash and small metadata; the workload lives on the node. Once the
contract leaves `ActiveNodeContracts[source]`, that node's ZOS reconciles, finds a
local deployment the chain no longer lists for it, and deprovisions it — and
`zmount.Deprovision` calls `DiskDelete`. Nothing is copied between nodes, and no
on-chain mechanism could copy it.

So **preserving data is the caller's responsibility, before calling.** The team's
intended sequence for the consolidation is: pause the deployment on the source,
copy it to S3, call `migrate_node_contract` — at which point tearing down the
source is safe and expected — then create the deployment on the destination. The
chain neither enforces nor observes any of that.

What migration preserves that cancellation does not: the contract ID, billing
continuity, the reward payee, reserved public IPs, and a live contract to deploy
against. That is the whole case for the feature — it turns an irreversible
operator action into a recoverable one.

## Decision

### New extrinsic (pallet-smart-contract)

| Extrinsic | Origin | Call index |
| --- | --- | --- |
| `migrate_node_contract(contract_id, node_id, deployment_hash)` | `RestrictedOrigin` (root or 3/5 council) | 22 |

`deployment_hash` is `Option<HexHash>`; `None` keeps the current hash. The hash
covers the full deployment sent to the node and legitimately changes for network
workloads but not for VMs. On-chain `deployment_data` is small metadata, unaffected
by a relocation, and is therefore not a parameter.

**Council-only for v1.** The consolidation is cross-tenant by construction, so
restricting v1 removes the whole authorization matrix (owner vs farmer, and who may
rewrite a deployment hash). **Owner and farmer paths must arrive as a new
`call_index(23)`, never by widening 22's origin check** — adding a dispatchable is
exempt from a `transaction_version` bump (`sp_version/src/lib.rs:209`), altering an
existing one's semantics is not (`:204-206`).

That rule cuts against this changeset's own report-handler fix, so state it plainly
rather than let this PR become precedent: `transaction_version` stays at 2 because
the call index and parameter encoding are byte-identical (so no signed payload
decodes differently), the change is strictly in the submitter's favour, and both
calls are ZOS telemetry with no offline-signing consumer. A future semantics change
lacking those three properties needs the bump.

### Preconditions enforced on-chain

1. Contract exists and is a node contract.
2. State is `Created`. Every target contract, and all 2,372 in Freefarm, are
   `Created`; the chain holds exactly one `GracePeriod` node contract, and
   relocating a contract due to auto-delete in 14 days only moves where it dies.
3. Destination differs from the source.
4. Same **farm**. No same-country check — farm membership is the only relationship
   the chain models, and it is what keeps public-IP reservations coherent, since
   `reserve_ip`/`free_ip` resolve the farm *through* the node.
5. Neither node is in `NodeV3BillingOptOut`, and neither has an
   `ActiveRentContractForNode`. Both would silently reprice the tenant: opting out
   waives billing, and a rent contract zeroes CPU/RAM/disk cost (`cost.rs:78`).
6. Destination is not standby and not dedicated. The **source** node's power state
   is deliberately unchecked — migrating off a machine you are about to shut down
   is the point.
7. The destination's `(node_id, deployment_hash)` key is strictly free. Stricter
   than `create_node_contract`, which permits overwriting a `Deleted` entry: there
   is no restore semantic here, and that contract's eventual `remove_contract`
   would unconditionally delete the key we just claimed.

### Behaviour

**Bills first**, settling at the source node's cost basis — billing reads the
source's certification and resolves the payee from the source farm — then re-reads
the contract, because billing can mutate or remove it. The two post-billing
branches answer deliberately differently:

- **grace → `Err`** (the live path): an underfunded twin transitions to
  `GracePeriod` inside this call. Nothing was removed, the contract still pins the
  node, and `Ok` would be a silent no-op indistinguishable from success. The
  rolled-back cycle is redone by the offchain worker.
- **gone → `Ok`** (currently unreachable, kept as a guard): `bill_contract` only
  deletes once an *existing* grace period elapses, and precondition 2 already
  refused anything but `Created`.

**The `ContractIDByNodeIDAndHash` removal is guarded by an equality check.**
`update_node_contract` never enforced hash uniqueness where `create_node_contract`
does, so a key may already point at a *different* live contract; an unguarded
remove would destroy that contract's index entry. This is a real regression guard
with a test, not defensive padding.

Then the contract moves between the two `ActiveNodeContracts` vectors, and
`ContractPaymentState.last_updated_seconds` is stamped unconditionally —
`bill_contract` skips this on its zero-amount early return, and the
`deployment_hash` parameter can change what the destination deploys.

**`NodeContractResources` is deliberately NOT cleared,** and the tenant is not
over-billed by the move. The entry is contract-keyed, so it survives the migration
and billing continues at the same quantity on the same schedule — the inline
`bill_contract` settles the source period and stamps the clock (`billing.rs:199,369`),
so the next cycle bills from that instant forward. No overlap, no gap, no double
charge; the loop index is `contract_id % billing_frequency`, which the move does not
change. The one real deviation is the certification multiplier, covered below.

Clearing the entry is what would break this. `calculate_resources_cost_units_usd`
(`cost.rs:71-89`) derives the whole node-contract cost from it, so zeroing it sends
`bill_contract` down its zero-amount early return (`billing.rs:276-286`) — no
overdraft, so no grace, so no 14-day auto-delete, leaving a free immortal contract
pinning the destination against `node_has_no_active_contracts`.

### Ordering is forced by ZOS: migrate first, then deploy

A deployment cannot be pre-staged. `validate()`
(`zosbase/pkg/provision/engine.go:616-626`) rejects any deployment whose contract
does not already name that node and whose `ChallengeHash()` does not match the
contract's on-chain hash; the only bypass is `boot()` reinstalling from local
storage. So the destination cannot accept the workload until the extrinsic has
landed — which is what the `deployment_hash` parameter exists for. Without it the
flow would need `update_node_contract`, which is owner-only, so a council-driven
migration could not fix the hash at all.

### No new event

`ContractUpdated(Contract<T>)` already carries the whole contract, hence the new
`node_id` and hash, and the indexer already writes both
(`tfchain_graphql/src/mappings/contracts.ts:227,231`). So tfchain_graphql and grid
proxy need no changes, and the Go client needs only a call wrapper. Source-node
cleanup needs no chain change either: `ContractEventHandler.sync()` runs hourly,
compares local deployments against `ActiveNodeContracts[node]`, and deprovisions
what the chain no longer lists.

**Prompt teardown was considered and is not worth building.** A ZOS node could act on
`ContractUpdated` directly rather than waiting for `sync()`, since every node
already decodes the complete `EventRecords` per block locally. But it accelerates
nothing the operator needs — `change_power_target(Down)` is gated on
`ActiveNodeContracts`, which is empty the instant the extrinsic lands — and the
measured benefit is the negligible stream saving above. If it is ever built:

- whoever builds it must confirm the operating procedure still copies data off the
source *before* calling, because same-block teardown removes any margin for a
procedure that does not.
- A dedicated `NodeContractMigrated` would let ZOS filter before pushing to its local
stream, but `ContractUpdated` fires well under once per hour chain-wide, so the
saving is about one discarded stream entry every few hours. It remains purely
additive if audit ever wants it.

### New errors

Appended at the end of the enum — `Error` variants have no explicit index and are
SCALE-encoded by declaration order: `NodeNotInSameFarm`, `ContractAlreadyOnNode`,
`NodeIsOptedOutOfV3Billing`, `ContractNotInCreatedState`.

`NodeNotAuthorizedToComputeReport` is left in place and marked reserved; the
companion fix below removed its last construction site, and deleting it would
renumber every variant below.

### Companion fix: report handlers skip a stale entry, consistently

Best read as a consistency sweep. Both handlers already tolerate a report naming a
contract that does not exist — `if let Some(contract)` in
`_report_contract_resources`, two `contains_key` guards in `_compute_reports` —
skipping it silently and for free, since both end in `Ok(Pays::No)`. A report
naming a contract that exists but sits on **another** node is the same class of
mistake, yet was handled the opposite way: `ensure!` rejecting the whole extrinsic,
and charging for it. That asymmetry was the anomaly. Both now `continue` and log.

The abort path stops being reachable exactly when it would start to matter: it was
unreachable before, because cancellation removes the contract and the
missing-contract guards catch it. `migrate_node_contract` is the first operation
that leaves a *live* contract pointing elsewhere, and left as it was, one migrated
contract would have frozen resource and NRU reporting for every other contract on
the source node.

Charging for a skipped entry was considered and rejected: it would re-introduce the
asymmetry in the other direction. If the anti-spam posture is revisited, both cases
should move together.

## Consequences

- No storage migration. `Contract` unchanged, `CONTRACT_VERSION` 4, `StorageVersion`
  V12, `transaction_version` 2. No indexer, grid proxy or ZOS change required.
- **An emptied `ActiveNodeContracts` is not permission to power the machine off.**
  It empties the instant the extrinsic lands and says nothing about where the
  workload is. Nothing in the pallet can detect the difference.
- **Certification is per-node and legitimately mixed within a farm**, driving a
  +25% Certified multiplier, so a council-approved move can reprice a tenant. Worse across the
  reporting gap: subsequent cycles read the *destination's* certification
  (`billing.rs:161-175`) against the *source's* still-stored footprint — new price ×
  old quantity, either direction, until the destination reports. Choosing
  certification-matched destinations makes the multiplier 1.0 and removes it. If the
  signed path is added it needs a directional guard.
- **A council motion has a weight ceiling.** `MaxProposalWeight` is 50% of max block
  weight, bounding a motion at roughly 900–1,800 contracts.

## Operational gate

Conditions on the first mainnet migration, not on merging.

1. **Data is copied off the source before the extrinsic is called.** The chain does
   not check this and cannot; deprovisioning follows the move automatically.
2. **Destinations are certification-matched**, removing the repricing exposure
   rather than documenting it.
3. **Batch per tenant group**, never all at once.
4. **Power the source down only after confirming the destination provisioned** —
   never on `ActiveNodeContracts` being empty.
