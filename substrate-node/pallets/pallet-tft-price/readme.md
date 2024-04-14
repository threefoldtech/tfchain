# TFT Price Module

A pallet that serves as a price oracle for TFT on Tfchain. It fetches the price on Stellar (TFT/USD) every configurable amount of blocks and stores it in the storage. An Average price is calculated based on the last 1440 prices (24 hours window if you fetch every 1 minute).

The price is fetched by an offchain worker that runs after a block is created. This offchain worker relies on the `aura` key in the keystore to sign the transaction that stores the price in the storage. This means that only valid block creators can sign this transaction. Given this configuration this pallet will only work in an Aura / Grandpa based chains.

## Terminology

- TFT: Threefold Token
- Price: TFT price stored in mUSD (milli USD or 1/1000 USD)

## Genesis configuration

- `min_tft_price`: Minimum TFT price in mUSD, used to prevent the TFT price from going too low.
- `max_tft_price`: Maximum TFT price in mUSD, used to prevent the TFT price from going too high.

## Interface

Dispatchable functions of this pallet.

- `set_price`: Set the price, can only be called by a block creator.
- `set_min_tft_price`: Set the minimum TFT price, can only be called by a configurable origin.
- `set_max_tft_price`: Set the maximum TFT price, can only be called by a configurable origin.