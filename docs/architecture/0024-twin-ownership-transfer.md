# 24. Twin Ownership Transfer in pallet-tfgrid

Date: 2025-09-03

## Status

Accepted

## Context

Operators occasionally need to transfer ownership of an existing Twin to a different account (e.g., organization handover, compromised keys, or account restructuring). Previously there was no safe, on-chain mediated process to move Twin ownership while preserving accounting invariants and preventing hijacks.

## Decision

Introduce a two-step, time-bound transfer protocol implemented in `pallet-tfgrid` that:

- Requires the prospective new owner to initiate the request (prevents unsolicited hijacks).
- Enforces that the new owner has accepted Terms & Conditions and does not already own a Twin.
- Requires explicit acceptance by the current owner before expiry.
- Repatriates all reserved balance from old owner to new owner on acceptance.

### Dispatchables

- `request_twin_transfer(origin=new_account, twin_id)`
  - Origin is the prospective new account.
  - Validates preconditions and creates a pending transfer with expiry.
  - Emits `TwinTransferRequested { twin_id, old_account, new_account }`.

- `accept_twin_transfer(origin=old_account, request_id)`
  - Origin must be the current (old) owner of the Twin.
  - Requires the request to be pending and unexpired.
  - Moves reserved balance from old to new as reserved, updates Twin owner and indexes, completes the request.
  - Emits `TwinOwnershipTransferred { twin_id, old_account, new_account }` and `TwinUpdated(Twin)`.

### Storage

- `TwinTransferRequests: RequestId -> TwinTransferRequest` (status, twin_id, old_account, new_account, expiry)
- `PendingTransferByTwin: TwinId -> RequestId` (enforces one pending request per Twin)
- `TwinTransferRequestID: u64` (monotonic counter)

### Types

- `TransferStatus` enum: `Pending | Completed`

### Expiry

- Requests expire after a fixed window (using HOURS from `tfchain_support::constants::time`). Acceptance after expiry fails.

## Security Considerations

- New owner must initiate request (prevents current owner from pushing ownership without consent of new owner).
- New owner must have signed T&C and must not own another Twin (prevents multi-ownership and aligns with usage rules).
- Current owner must accept while request is pending and before expiry.
- On acceptance, reserved balance is repatriated using `repatriate_reserved(..., BalanceStatus::Reserved)`; failures are tolerated but the pallet attempts best-effort transfer before ownership move.

## Consequences

- Clear, auditable transfer trail via events.
- Compatible with existing Twin lifecycle; no changes to Twin schema.

## Backwards Compatibility & Migration

- New storage items are additive.
- No migration of existing state required.

## References

- Implementation: `substrate-node/pallets/pallet-tfgrid/src/twin_transfer.rs`
- Extrinsics wiring: `substrate-node/pallets/pallet-tfgrid/src/lib.rs` (call_index 40, 41)
