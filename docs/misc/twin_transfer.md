# Twin Ownership Transfer

This document explains the twin ownership transfer flow in the `pallet-tfgrid` pallet and the main events and errors.

## Overview

- The current (old) owner initiates the transfer via `request_twin_transfer(new_account)`.
- The prospective new owner accepts via `accept_twin_transfer(request_id)`.
- The current owner may cancel via `cancel_twin_transfer(request_id)`.
- On success, ownership of the twin moves to the new account, reserved balances are repatriated, and indices are updated.

## Dispatchables

- request_twin_transfer(origin=old_owner, new_account)
  - Origin (signer) is the current (old) owner; `twin_id` is derived from the signer.
  - Emits `TwinTransferRequested { request_id, twin_id, from, to }`.
- accept_twin_transfer(origin=new_account, request_id)
  - Origin must be the intended new owner of the twin.
  - Emits `TwinOwnershipTransferred { request_id, twin_id, from, to }` and `TwinUpdated(Twin)`.
- cancel_twin_transfer(origin=old_owner, request_id)
  - Origin must be the current (old) owner of the twin.
  - Emits `TwinTransferCanceled { request_id, twin_id, from, to }`.

## Preconditions

- Twin must exist.
- New account must have accepted Terms & Conditions (`user_accept_tc`).
- New account must not already own a twin.
- Only one pending transfer per twin.

## Common Errors

- UserDidNotSignTermsAndConditions: new account did not accept T&C.
- TwinTransferNewAccountHasTwin: new account already has a twin.
- TwinTransferPendingExists: a pending transfer already exists for this twin.
- TwinTransferRequestNotFound: request ID does not exist.
- UnauthorizedToUpdateTwin: signer is not authorized for this action.

## Events

- TwinTransferRequested { request_id, twin_id, from, to }
- TwinOwnershipTransferred { request_id, twin_id, from, to }
- TwinTransferCanceled { request_id, twin_id, from, to }
- TwinUpdated(Twin)

## Storage

- `TwinTransferRequests: RequestId -> TwinTransferRequest` where `TwinTransferRequest { twin_id, from, to, created_at }`.
- `PendingTransferByTwin: TwinId -> RequestId` enforces at most one pending transfer per twin.

## Notes

- Reserved balance of the old owner is repatriated to the new owner as reserved during acceptance.
