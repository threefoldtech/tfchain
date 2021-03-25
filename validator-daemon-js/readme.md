## Vesting Validator

This document will explain how the vesting automated withdrawals will work in a production setup. 

The vesting validator relies on multiple parts:

- The substrate blockchain with the vesting pallet and tft price pallet enabled.
- The vesting validator daemon (NodeJS).
- The vesting service (dashboard).
- The validator accounts.

## Vesting account

A vesting account is a Stellar Multisignature Account, where the signers are:

- the owner himself
- the validators

This means that in order to transfer funds, a majority of the signers on this account must provide a signature before it can be submitted on the network. When the owner wants to withdraw funds from this "vesting account", he and the validators must provide their signatures. In order to make this an automated system, we will be using Substrate as the decentralised communication channel between these validators.

## Substrate

Pallets:

- [vesting pallet](https://github.com/threefoldtech/substrate-pallets/tree/master/pallet-vesting-validator)
- [tft price pallet](https://github.com/threefoldfoundation/tft-parity/tree/main/pallet-tft-price)

### Vesting Pallet

This is a runtime extension which adds a voting mechanism to "vote" for validity of stellar withdrawal transactions. This pallet has a list of `validators`, these are substrate accounts (ed25519) which can only be added by using the sudo extension. This will make sure only true validators can be added, as there will be an admin for this service which in his turn will add the validator accounts to this runtime storage map.

The vesting pallet gives the functionality to propose a Stellar transaction, anyone can propose a transaction. Once a transaction is proposed it needs (n / 2 + 1) votes (where `n` is the amount of validators) in order to become a `valid` transaction. A majority must vote for validity in shorter terms.

To vote on the validity off a transaction, a "validator" must provide a signature that signs the Stellar Multisig withdrawal transaction. Once enough signatures are provided by the validators, the runtime pallet will see this transaction as "valid".

### TFT Price pallet

This runtime extension will check the price on a regular basis and compute a monthly average price. This price can be retrieved by the validator daemon in order to validate withdrawal transactions.

## Vesting validator daemon (NodeJS).

This is a daemon which needs to run by the "validators". Atleast (n / 2 + 1) (where `n` is the amount of validators) need to run this validator in order to make this system work.
When you start the daemon, you can provide a mnemonic which will load the appropriate accounts.

Accounts for:

- Substrate
- Stellar

In `src/vesting.js` you can find the code that handles following things:

- It checks wether a withdrawal transaction is valid (based on the vesting schedule).
- Signs a transaction and stores the signature back on chain (Substrate).
- Submits a "valid" transaction to the stellar network.

On startup, it monitors the Vesting Pallet (see above). When certain events are emitted from chain, for example (TranasactionProposed(...), TransactionReady(...), TransactionFailed(...), ...) it takes action based on which event is emitted.

### TransactionProposed

This is the entry point of the validation sequence, when a transaction is submitted to chain, the vesting validator decodes the Stellar transaction and does a number of checks on it.

- if the escrow account has enough funds to withdraw
- if the timecondition is met
- if the tft price is met
- if a period has ended
- if the transaction is not a duplicate (replay protection)
- etc..

When it considers this transaction to be valid, it signs the transactions, extracts the signature and posts this signature on chain.

### TransactionsReady

Once enough validators (n / 2 + 1) have posted their signature on the blockchain, a `TransactionsReady` event is emitted from the Vesting Pallet. 
When this happens the validator will gather all signatures that are stored on chain and compose the Stellar transaction.
The transaction will be submitted to the Stellar network and the status of the transaction (success/failed) will be reported.

## Vesting service (dashboard)

In this dashboard the user will vest his tokens and see how much he can unlock if the vesting schedule conditions are met. 
For each vesting account, there should be a button that the user can interact with to withdraw TFT from said vesting account. When this button is pressed, the user should be prompted for his signature. When he provides the signatures, the transaction should be submitted on chain (using the Vesting Pallet) in XDR format.

## Graphql Stack (optional)

We can optionally deploy the graphql and indexer for this setup. There are mappers which map every transaction and saves them to storage so we can query them with ease later.
