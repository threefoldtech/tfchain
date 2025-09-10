# 24. Twin Ownership Transfer in pallet-tfgrid

Date: 2025-09-03

## Status

Accepted

## Context

Operators occasionally need to transfer ownership of an existing Twin to a different account (e.g., organization handover, compromised keys, or account restructuring).
Previously there was no safe, on-chain mediated process to move Twin ownership while preserving accounting invariants and preventing hijacks.

## Decision

Introduce a simple two-step transfer protocol implemented in `pallet-tfgrid` that:

- Requires the current (old) owner to initiate and specify the intended new account.
- Enforces that the new account has accepted Terms & Conditions and does not already own a Twin.
- Requires explicit acceptance by the new account.
- Allows the current owner to cancel a pending request at any time.
- Repatriates all reserved balance from old owner to new owner on acceptance.

### Dispatchables

- `request_twin_transfer(origin=old_account, new_account)`
  - Origin is the current (old) owner.
  - Validates preconditions and creates a pending transfer.
  - Emits `TwinTransferRequested { request_id, twin_id, from, to }`.

- `accept_twin_transfer(origin=new_account, request_id)`
  - Origin must be the intended new owner.
  - Moves reserved balance from old to new as reserved, updates Twin owner and indexes, and completes the request.
  - Emits `TwinOwnershipTransferred { request_id, twin_id, from, to }` and `TwinUpdated(Twin)`.

- `cancel_twin_transfer(origin=old_account, request_id)`
  - Origin must be the current (old) owner.
  - Cancels and removes the pending request.
  - Emits `TwinTransferCanceled { request_id, twin_id, from, to }`.

### Storage

- `TwinTransferRequests: RequestId -> TwinTransferRequest` (twin_id, from, to, created_at)
- `PendingTransferByTwin: TwinId -> RequestId` (enforces one pending request per Twin)
- `TwinTransferRequestID: u64` (monotonic counter)

### Types

- `TwinTransferRequest` includes `from: AccountId`, `to: AccountId`, and `created_at: BlockNumber` to record when the request was created.
  - There is no expiry logic in v1.
  - Future cleaners (on_finalize/offchain) can use `created_at` to remove stale items if desired.

### Errors and Semantics

- Request flow is capped at one request per twin. If a request already exists, `request_twin_transfer` returns a single error:
  - `TwinTransferPendingExists` ("cancel the existing request first")
- Accept flow has no expiry checks; presence of a matching request and correct signer are sufficient.
- Cancel flow always succeeds for the old owner:
  - Removes the request and emits `TwinTransferCanceled`.

## Security Considerations

- Current owner cannot push a transfer without new account cooperation (accept step by new account is required).
- New account must have signed T&C and must not own another Twin (prevents multi-ownership and aligns with usage rules).
- Owner can cancel any time to unblock.
- On acceptance, reserved balance is repatriated using `repatriate_reserved(..., BalanceStatus::Reserved)`; failures are tolerated but the pallet attempts best-effort transfer before ownership move.

## Consequences

- Clear, auditable transfer trail via events.
- Compatible with existing Twin lifecycle; no changes to Twin schema.

## Backwards Compatibility & Migration

- New storage items are additive.
- No migration of existing state required.

## References

- Implementation: `substrate-node/pallets/pallet-tfgrid/src/twin_transfer.rs`
- Extrinsics wiring: `substrate-node/pallets/pallet-tfgrid/src/lib.rs` (call_index 40, 41, 42)
