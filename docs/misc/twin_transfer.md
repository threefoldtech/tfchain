# Twin Ownership Transfer

This document explains the twin ownership transfer flow in the `pallet-tfgrid` pallet and the main events and errors.

## Overview

- The new owner initiates the transfer via `request_twin_transfer(twin_id)`.
- The current owner accepts via `accept_twin_transfer(request_id)`.
- On success, ownership of the twin moves to the new account, reserved balances are repatriated, and indices are updated.

## Dispatchables

- request_twin_transfer(origin, twin_id)
  - Origin (signer) is the new account.
  - Emits `TwinTransferRequested { twin_id, old_account, new_account }`.
- accept_twin_transfer(origin, request_id)
  - Origin must be the current (old) owner of the twin.
  - Emits `TwinOwnershipTransferred { twin_id, old_account, new_account }` and `TwinUpdated(Twin)`.

## Preconditions

- Twin must exist.
- New account must have accepted Terms & Conditions (`user_accept_tc`).
- New account must not already own a twin.
- Only one pending transfer per twin.
- Acceptance must happen before expiry (request has an expiry block window).

## Common Errors

- UserDidNotSignTermsAndConditions: new account did not accept T&C.
- TwinTransferNewAccountHasTwin: new account already has a twin.
- TwinTransferPendingExists: a pending transfer already exists for this twin.
- TwinTransferRequestNotFound: request ID does not exist.
- TwinTransferRequestAlreadyCompleted: request already completed/cannot be accepted again.
- TwinTransferRequestExpired: acceptance after expiry is rejected.
- UnauthorizedToUpdateTwin: accept extrinsic not signed by current owner.

## Events

- TwinTransferRequested
- TwinOwnershipTransferred
- TwinUpdated

## Notes

- Reserved balance of the old owner is repatriated to the new owner as reserved during acceptance.
- A simple integration test is available under `substrate-node/tests/integration_tests.robot` ("Test Twin Transfer Flow").
