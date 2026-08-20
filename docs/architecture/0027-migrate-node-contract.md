# 27. Migrate a Node Contract to Another Node in the Same Farm

Date: 2026-08-18

## Status

Accepted

## Context

A farmer cannot power a node down while it holds a single active contract:
`change_power_target(Down)` requires `node_has_no_active_contracts`. The only way
to empty a node today is to cancel its contracts, which is terminal — the contract
is gone, and with it any path for the tenant to return under the same identity.

That blocks hardware consolidation. On mainnet at spec 157, emptying ten machines
in Freefarm means relocating 74 contracts across 29 twins, none of them the
operator. Asking those twins to recreate contracts by hand is not available to a
hosting provider.

Two alternatives were rejected. **Cancel and recreate** is the problem, not the
fix. **Off-chain coordination** cannot work: the contract ID is the identity ZOS,
the billing loop and the indexer all key on, and nothing off-chain can move it.

### What migration moves, and what it does not

**It moves the booking, not the bytes.** The chain stores a contract ID, a
deployment hash and small metadata; the workload lives on the node. Once the
contract leaves the source node's `ActiveNodeContracts`, that node's ZOS
reconciles, finds a local deployment the chain no longer lists for it, and
deprovisions it — deleting the disks. Nothing is copied between nodes, and no
on-chain mechanism could copy it.

So **preserving data is the caller's responsibility, before calling.** The team's
intended sequence is: pause the deployment on the source, copy it to S3, call
`migrate_node_contract` — at which point tearing down the source is safe and
expected — then create the deployment on the destination. The chain neither
enforces nor observes any of that.

What migration preserves that cancellation does not: the contract ID, billing
continuity, the reward payee, reserved public IPs, and a live contract to deploy
against. That is the whole case for the feature — it turns an irreversible
operator action into a recoverable one.

## Decisions

### 1. A new extrinsic, restricted to council

| Extrinsic | Origin | Call index |
| --- | --- | --- |
| `migrate_node_contract(contract_id, node_id, deployment_hash)` | `RestrictedOrigin` (root or 3/5 council) | 22 |

The consolidation is cross-tenant by construction, so restricting v1 removes the
whole authorization matrix — owner versus farmer, and who may rewrite a deployment
hash — rather than answering it.

`deployment_hash` is `Option<HexHash>`; `None` keeps the current hash. It exists
because the hash covers the full deployment sent to the node and legitimately
changes for network workloads, though not for VMs. On-chain `deployment_data` is
small metadata unaffected by a relocation, so it is not a parameter.

**Forward constraint: owner and farmer paths must arrive as a new `call_index(23)`,
never by widening 22's origin check.** Adding a dispatchable is exempt from a
`transaction_version` bump; altering an existing one's semantics is not.

### 2. Same farm, and deliberately no same-country rule

Farm membership is the only relationship the chain models, and it is what keeps
public-IP reservations coherent, since `reserve_ip`/`free_ip` resolve the farm
*through* the node. A same-country rule would encode a relationship the chain
cannot verify.

### 3. `Created` contracts only

Every target contract is `Created`, and the chain holds exactly one `GracePeriod`
node contract. Relocating a contract that will auto-delete in 14 days only moves
where it dies.

### 4. Opted-out and rented nodes excluded, on both sides

Both would silently reprice the tenant: opting out of v3 billing waives billing
entirely, and a rent contract zeroes CPU/RAM/disk cost. Excluding them costs
nothing in the farms this was built for and removes two repricing paths.

### 5. Bill before moving

Billing settles at the **source** node's cost basis — it reads the source's
certification and resolves the payee from the source farm — so the source period
is closed before the contract changes hands. The contract is then re-read, because
billing can mutate or remove it, and the two branches that follow answer
deliberately differently: a contract pushed into grace returns `Err`, a contract
already deleted returns `Ok`. The reasoning for that asymmetry, and for the
guarded `ContractIDByNodeIDAndHash` removal and the unconditional clock stamp,
lives in comments at each site in `_migrate_node_contract`, where the next person
to touch them will actually read it.

### 6. Reuse `ContractUpdated`; add no event

`ContractUpdated` already carries the whole contract, hence the new `node_id` and
hash, and the indexer already writes both. So tfchain_graphql and grid proxy need
no changes, and the Go client needs only a call wrapper. Source-node cleanup needs
no chain change either: `ContractEventHandler.sync()` runs hourly, compares local
deployments against `ActiveNodeContracts[node]`, and deprovisions what the chain no
longer lists.

A dedicated `NodeContractMigrated` would let ZOS filter before pushing to its local
stream, but `ContractUpdated` fires well under once per hour chain-wide, so the
saving is roughly one discarded stream entry every few hours. It stays purely
additive if audit ever wants it.

### 7. Do not build prompt teardown

A ZOS node could act on `ContractUpdated` directly rather than waiting for
`sync()`, since every node already decodes the complete `EventRecords` per block
locally. It is not worth building: it accelerates nothing the operator needs, since
`change_power_target(Down)` is gated on `ActiveNodeContracts`, which empties the
instant the extrinsic lands — and the benefit is the negligible stream saving
above.

If it is ever built, whoever builds it must confirm the operating procedure still
copies data off the source *before* calling, because same-block teardown removes
any margin for a procedure that does not.

### 8. Report handlers skip a stale entry instead of aborting

A consistency sweep more than a behaviour change. Both handlers already tolerated a
report naming a contract that does not exist, skipping it silently and for free. A
report naming a contract that exists but sits on **another** node is the same class
of mistake, yet was handled the opposite way — rejecting the whole extrinsic and
charging for it. That asymmetry was the anomaly.

It was also unreachable until now: cancellation removes the contract, so the
missing-contract guards caught it. `migrate_node_contract` is the first operation
that leaves a *live* contract pointing elsewhere, and left alone, one migrated
contract would have frozen resource and NRU reporting for every other contract on
the source node.

Charging for a skipped entry was considered and rejected — it would re-introduce
the asymmetry in the other direction. If the anti-spam posture is revisited, both
cases should move together.

## What the extrinsic refuses

1. A contract that does not exist, or is not a node contract.
2. A contract not in `Created` state.
3. A destination equal to the source.
4. A destination in a different farm.
5. Either node opted out of v3 billing, or holding a rent contract.
6. A destination that is in standby or dedicated. The **source** node's power state
   is deliberately unchecked — migrating off a machine you are about to shut down
   is the point.
7. A destination whose `(node_id, deployment_hash)` key is taken. Stricter than
   `create_node_contract`, which permits overwriting a `Deleted` entry: there is no
   restore semantic here, and that contract's eventual `remove_contract` would
   delete the key we just claimed.

New errors are appended at the end of the enum, since `Error` variants are
SCALE-encoded by declaration order: `NodeNotInSameFarm`, `ContractAlreadyOnNode`,
`NodeIsOptedOutOfV3Billing`, `ContractNotInCreatedState`.
`NodeNotAuthorizedToComputeReport` is kept and marked reserved — decision 8 removed
its last construction site, and deleting it would renumber every variant below.

## Consequences we accept

- **No storage migration.** `Contract` unchanged, `CONTRACT_VERSION` 4,
  `StorageVersion` V12, `transaction_version` 2. No indexer, grid proxy or ZOS
  change required.
- **An emptied `ActiveNodeContracts` is not permission to power the machine off.**
  It empties the instant the extrinsic lands and says nothing about where the
  workload is. Nothing in the pallet can detect the difference.
- **A council-approved move can reprice a tenant.** Certification is per-node and
  legitimately mixed within a farm, and Certified nodes bill 25% above Diy. Worse
  across the reporting gap: subsequent cycles read the *destination's*
  certification against the *source's* still-stored footprint — new price × old
  quantity, either direction, until the destination reports. Certification-matched
  destinations make the multiplier 1.0 and remove it. If the signed path is added,
  it needs a directional guard.
- **A council motion has a weight ceiling.** `MaxProposalWeight` is 50% of max
  block weight. Against the measured weight, that bounds a motion at about 2,000
  contracts when the nodes involved are near-empty, falling to roughly **340** when
  migrating off a node holding ~1,660 — the cost scales with the length of the
  contract vectors being rewritten, so the busiest nodes are the ones that batch
  worst. Size batches per tenant group and this ceiling is never near.

## Operational gate

Conditions on the first mainnet migration, not on merging.

1. **Data is copied off the source before the extrinsic is called.** The chain does
   not check this and cannot; deprovisioning follows the move automatically.
2. **Destinations are certification-matched**, removing the repricing exposure
   rather than documenting it.
3. **Batch per tenant group**, never all at once.
4. **Power the source down only after confirming the destination provisioned** —
   never on `ActiveNodeContracts` being empty.
