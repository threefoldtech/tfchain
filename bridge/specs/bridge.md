# TFT Tfchain Stellar bridge

## Stellar TFT to Tfchain TFT

The bridge monitors a central stellar account that is goverened by the threefoldfoundation. When a user sends an amount of TFT to that stellar account, the bridge will pick up this transaction.

### Destination

In the memo text of this transaction is the Tfchain address of the receiver.

**TODO: how to convert/encode an SS58 address so it fits in the Stellar memo text.**

## Components

There are 3 main components to get the bridge working.

- Central stellar bridge wallet (requires multisig in order to be secure)
- The `pallet-tft-bridge` (the runtime functionality)
- Validator daemons

Note: There must always be as many validator daemons as there are signers on the stellar bridge wallet account.

## Pallet TFT Bridge

Is the main runtime functionality, distributes consensus for minting / burning tokens on TF Chain and manages signature passaround for the validators in order to execute a multisig transaction on the central Stellar bridge wallet.

Contains following extrinsic to call:

#### Admin setup
- `add_validator(accountId)` (root call for the admin to setup the bridge)
- `remove_validator(accountId)` (root call for the admin to setup the bridge)
- `set_fee_account(accountId) (root call for the admin to setup the bridge fee wallet on tfchain)`
- `set_deposit_fee(amount) (root call for the admin to setup the bridge desposit on tfchain)`
- `set_burn_fee() (root call for the admin to setup the bridge burn fee on tfchain)`
#### Stellar -> TF Chain (minting on TF Chain)
- `propose_or_vote_mint_transaction(transaction, target, amount)`
#### TF Chain -> Stellar (burning on TF Chain)
- `propose_burn_transaction_or_add_sig(transaction, target, amount, signature, stellarAccountAddress)`
#### User callable extrinsic
- `swap_to_stellar(target, amount)`

## Validator daemon

Is a signer of the multisig Stellar bridge wallet. This code does the following things:

- Monitors central Stellar bridge wallet for incoming transactions
- Monitors events on chain

More will be explained below

## Stellar swap to TF Chain flow

A user looks up the Stellar bridge wallet address for a swap to TF Chain. He then sends a transaction with some amount to the bridge wallet. 

Now the validator daemons monitoring the bridge wallet will see an incoming transaction and execute a `propose_or_vote_mint_transaction` on TF Chain. If the transaction has already been executed before, this will fail.

If more then *(number of validators / 2) + 1* voted that this mint transaction is valid, then the chain will execute a `mint` on the target address for the specified amount of tokens.

## TF Chain swap to Stellar

A user will call `swap_to_stellar(optional target, amount)`. The validator daemons will pick up a an event containing information about the swap to stellar. The validators will call `propose_burn_transaction_or_add_sig` with the information about the stellar transaction.

If more then *(number of validators / 2) + 1* signatures are present on the burn transaction, the transaction is considered valid and the chain will emit an event that the transaction is ready to be submitted.

The daemons will see this transaction ready event and retrieve the transcation object from storage. They will submit the transaction, along with all the signatures to the stellar network. Only one validator will succeed in this operation, the other validators will get an error, but they can ignore that because stellar has a double spend protection mechanism.