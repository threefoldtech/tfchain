# 23. Billing Refactor in pallet-smart-contract Module

Date: 2024-08-18

## Status

Accepted

## Context

The billing logic within the pallet-smart-contract module of the TFChain project faced several issues, including critical bugs, inefficiencies in handling validators, and inadequate fund reservation mechanisms. The goal of this refactor was to address these issues while enhancing the overall reliability and robustness of the billing process.

## Decision

The following architectural decisions were made to improve the billing logic:

1. Refactoring Billing Logic
Enhanced Tracking: Improved tracking mechanisms were introduced for contract payments, especially during grace periods and when handling overdue payments. The new ContractPaymentState was introduced to accurately manage contract payment states and resolve liquidity issues.
Overdraft Handling: Partial payments are now allowed, where users can cover part of their billing due if they lack sufficient funds for the full amount. Overdraft are tracked separately.
Fund Reservation: The use of balance locks/freezes was replaced with a more reliable reservation system. Reserved amounts are not part of the free balance, reducing issues related to fund availability for reward distribution.
2. Improved Off-Chain Worker Logic
Runtime Verification: The is_next_block_author function was removed, with verification now occurring at runtime. This change ensures transaction fees are reliably waived for validators and still prevents duplicate transactions.
3. New Events Introduced
ContractGracePeriodElapsed: Indicates that a contract's grace period has ended.
ContractPaymentOverdrawn: Logs when a contract payment is overdrawn.
RewardDistributed: Tracks when rewards have been distributed.
4. Certified vs DIY Capacity
The system now correctly charges certified capacity at a higher rate (25% more) than DIY capacity.

## Consequences

Increased Robustness: The billing process is now more robust and less prone to errors, particularly in scenarios involving partial payments and fund reservations.
Better Visibility: The introduction of new events and improved logging provides better visibility into the billing and payment processes, aiding in debugging and monitoring.
Backward Compatibility: While significant refactoring has been done, efforts were made to ensure backward compatibility where possible, especially concerning contract migration.
