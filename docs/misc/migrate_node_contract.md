# Migrating a Node Contract

See [ADR 0027](../architecture/0027-migrate-node-contract.md) for why this exists
and what was decided.

## Overview

`migrate_node_contract` moves a live node contract to another node **in the same
farm** without cancelling it. The contract keeps its id, billing state, reported
resources and reserved public IPs, so the tenant is left with something to deploy
against — which cancelling is not.

It moves the **booking, not the workload**. Nothing is copied between nodes. Once
the contract leaves the source node's `ActiveNodeContracts`, that node deprovisions
its local copy on the next hourly reconciliation and deletes the disks. Getting the
data off the source beforehand is the caller's job.

## Dispatchable

```
migrate_node_contract(contract_id: u64, node_id: u32, deployment_hash: Option<HexHash>)
```

Origin: root, or a 3/5 council motion. Pass `None` for `deployment_hash` to keep the
current one.

## Preconditions checked on chain

- the contract exists, is a node contract, and is in `Created` state
- destination is a different node, in the **same farm**
- neither node is opted out of v3 billing, nor under a rent contract
- destination is not in standby and not dedicated
- destination does not already hold a contract with the same deployment hash

The **source** node's power state is not checked — migrating off a machine you are
about to shut down is the point.

## Not checked on chain

Read this section before planning a wave.

- **Destination capacity.** Neither this call nor `create_node_contract` verifies
  the destination can host the workload; ZOS decides that at deployment time. So an
  oversubscribed destination is discovered *after* the contract has moved and the
  source has been told to let go. Check real free memory on the destination, not
  just reported usage.
- **That the data was preserved.** See Overview.
- **That anything is actually running.** `ActiveNodeContracts` for the source is
  empty the instant the call lands. That is also what `change_power_target(Down)`
  gates on, so the chain will report the machine safe to sleep while the workload
  is still on it and not yet rebuilt anywhere else.

## Before you call

1. Get the data off the source.
2. Prefer a destination whose **node certification matches the source**. Certified
   nodes bill 25% above Diy, and until the destination files its first resource
   report the tenant is billed at the destination's rate against the source's
   footprint. Matching certification makes that 1.0, and a farm whose nodes are all
   one certification cannot hit this at all.
3. Confirm the contract still exists, still belongs to that twin and still sits on
   the source — plans go stale, contracts get cancelled continuously.
4. Confirm the contract has a `ContractPaymentState`; without one `bill_contract`
   fails and the migration cannot proceed.

## Submitting through the council

Propose → vote → close, as in [council.md](council.md). Notes specific to this call:

- Use `utility.batch` **per tenant group**, not `batch_all` across everything.
  `batch_all` is atomic, so one underfunded twin rolls back every other migration in
  the batch along with its settlement.
- A motion is capped by `MaxProposalWeight` (50% of max block weight), which bounds
  it at roughly 900–1,800 contracts. The weight scales with how many contracts the
  source and destination nodes already hold.

## Common errors

| Error | Meaning |
| --- | --- |
| `ContractNotExists` | wrong id, or already cancelled |
| `InvalidContractType` | not a node contract (name or rent) |
| `ContractNotInCreatedState` | in grace or deleted — or it entered grace during this call's own billing, which means the twin is underfunded |
| `ContractAlreadyOnNode` | source and destination are the same |
| `NodeNotInSameFarm` | cross-farm move; not supported |
| `NodeIsOptedOutOfV3Billing` | either side is opted out |
| `NodeNotAvailableToDeploy` | destination is rented, standby, or dedicated |
| `ContractIsNotUnique` | destination already has a contract with this deployment hash — pass a fresh hash |
| `ContractPaymentStateNotExists` | contract is already unbillable; cannot migrate |
| `BadOrigin` | not submitted as root or through a council motion |

## Verifying

After the call:

- `Contracts(contract_id).node_id` is the destination
- the contract is gone from `ActiveNodeContracts(source)` and present in
  `ActiveNodeContracts(destination)`
- a `ContractUpdated` event carries the new node id and hash

## After the call

Deploy on the destination. The order is forced — a node rejects a deployment whose
contract does not already name it, so the destination cannot be prepared in advance.

The source deprovisions its stale copy on its next reconciliation (hourly).

**Power the source down only after confirming the destination is provisioned and
serving** — never on `ActiveNodeContracts` being empty.
