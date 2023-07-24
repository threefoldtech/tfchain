# TFT Bridge Module

A pallet that serves as a bridge between TFT on Tfchain and TFT on [Stellar](https://stellar.expert/explorer/public/asset/TFT-GBOVQKJYHXRR3DX6NOX2RRYFRCUMSADGDESTDNBDS6CDVLGVESRTAC47-1?asset[]=TFT-GBOVQKJYHXRR3DX6NOX2RRYFRCUMSADGDESTDNBDS6CDVLGVESRTAC47-1)

## Overview

The TFT Bridge module provides functions for:

- Bridging TFT from Tfchain to Stellar
- Creating consensus to bridge TFT from Stellar to Tfchain

## Terminology

- Bridge Validator: A bridge validator is a node that is allowed to propose or vote for mint, burn or refund transactions.
- Withdraw Fee: A fee that is paid when withdrawing TFT from Tfchain to Stellar.
- Deposit Fee: A fee that is paid when depositing TFT from Stellar to Tfchain.
- Mint Transaction: A mint transaction that creates token on Tfchain based on consensus reached by the validators that a certain transfer was created on Stellar.
- Burn Transaction: A burn transaction that withdraws tokens on Stellar based on consensus reached by the validators that a certain swap to Stellar was made.
- Refund Transaction: A refund transaction that refunds tokens on Stellar based on consensus reached by the validators that a certain deposit needs to be refunded.

## Dispatchable Functions

- `add_bridge_validator`: Add a bridge validator, can only be called by a configurable origin.
- `remove_bridge_validator`: Remove a bridge validator, can only be called by a configurable origin.
- `set_fee_accont`: Set the fee account, can only be called by a configurable origin.
- `set_withdraw_fee`: Set the withdraw fee, can only be called by a configurable origin.
- `set_deposit_fee`: Set the deposit fee, can only be called by a configurable origin.
- `swap_to_stellar`: Swaps TFT from Tfchain to Stellar, effectively burning TFT on Tfchain withdrawing TFT on Stellar from a vault acocunt.
- `propose_or_vote_mint_transaction`: Propose or vote for a mint transaction, can only be called by a bridge validator.
- `propose_burn_transaction_or_add_sig`: Propose a burn transaction or add a signature to a burn transaction, can only be called by a bridge validator.
- `set_burn_transaction_executed`: Set a burn transaction as executed, can only be called by a bridge validator.
- `create_refund_transaction_or_add_sig`: Create a refund transaction or add a signature to a refund transaction, can only be called by a bridge validator.
- `set_refund_transaction_executed`: Set a refund transaction as executed, can only be called by a bridge validator.

## Note

See [bridge](../../../bridge/README.md) for more information about the bridge and how it works.