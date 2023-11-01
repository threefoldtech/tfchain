# Bridging TFT between Stellar and TF Chain

## Usage

This document will explain how you can transfer TFT from TF Chain to Stellar and back.

## Prerequisites

*   Threefold Connect application or any other Stellar wallet
*   A running bridge and bridge wallet address

## Stellar to TF Chain

Transfer the TFT from your Stellar wallet to bridge wallet address that you configured. A depositfee of 1 TFT will be taken, so make sure you send a larger amount as 1 TFT.

### Transfer to TF Chain

We also enabled deposits to TF Grid objects. Following objects can be deposited to:

*   Twin
*   Farm
*   Entity
*   Node

To deposit to any of these objects, a memo text in format `object_objectID` must be passed on the deposit to the bridge wallet. Example: `twin_1`.

To deposit to a TF Grid object, this object **must** exists. If the object is not found on chain, a refund is issued.

## TF Chain to Stellar

Browse to https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Ftfchain.grid.tf#/extrinsics (for mainnet), select tftBridgeModule and extrinsic: `swap_to_stellar()`. Provide your stellar target address and amount and sign it with your account holding the tft balance.
Again, a withdrawfee of 1 TFT will be taken, so make sure you send a larger amount as 1 TFT.

The amount withdrawn from TF Chain will be sent to your Stellar wallet.

Example: ![swap\_to\_stellar](swap_to_stellar.png)

## Deeper look at how the TFChain Bridge works

### STELLAR -> TFCHAIN (Lock-and-Mint flow)

In this section, we look into the details of transferring TFT from a Stellar Account to a TFChain Account.

1.  A transaction is received on the bridge Stellar account (aka. vault) and monitored by bridge validators (deamons).

2.  Each time such transaction event is reported by a bridge validator it undergoes some validation. If it fails, a refund is issued (sent back to the stellar source account). Here we assume that the validation has passed, but we will examine the refund flow in the next section.

3.  The first bridge validator reporting the transaction will propose a mint by calling `propose_or_vote_mint_transaction()` extrinsic on the TFTBridgeModule in TFChain. This extrinsic inserts a new `MintTransaction` in `MintTransactions` storage that includes the `amount`, `target`, `block`, `votes`, and emits a `MintTransactionProposed` event. The mint is considered processed by the bridge side at this point.

4.  Other bridge validators that report the transaction later will only add their votes for that proposal by calling same TFChain extrinsic. Since the `MintTransaction` already exists in `MintTransactions` storage, the extrinsic will increment the `votes` count for the specified `MintTransaction` and emit `MintTransactionVoted` event.

5.  From the TFChain side, if the majority (more than the half) of bridge validators agree on the transaction, tokens are minted to the target address. This check happens every time the `propose_or_vote_mint_transaction()` extrinsic is executed by validator call. Then, the transaction is removed from bridge pallet `MintTransactions` storage and added to `ExecutedMintTransactions`. Finally, a `MintCompleted` event is emitted.

#### Overview of the TFChain Minting events

1.  `tftBridgeModule.MintTransactionProposed`: A bridge validator proposed a mint transaction after being the first to report a stellar deposit.
2.  `tftBridgeModule.MintTransactionVoted`: Other bridge validators reported same stellar deposit and voted for the mint proposal.
3.  `tftBridgeModule.MintCompleted`: Enough bridge validator votes was collected and the tokens was successfully minted to the target address.

#### When a Refund-on-Stellar occurs?

A refund on Stellar occurs when one of the following conditions is met:

*   The deposited amount is lower than the deposit fee.
*   The memo message is empty.
*   The transaction contains more than one payment.
*   The memo is not formatted correctly.
*   The grid type is not supported (not one of grid, farm, node, or entity) or not found.

### STELLAR -> TFCHAIN (Refund-on-Stellar flow)

In this section, we look into the details of what happens when the a Stellar deposit can not be processed due to a validation problem.

1.  A transaction is received on the bridge Stellar account (aka. vault) and monitored by bridge validators (deamons).

2.  Each time such transaction event is reported by a bridge validator it undergoes some validation. Here, we assume that the validation has failed because of one of the violations mentioned in the previous section so a refund flow is initiated.

3.  The first bridge validator reporting a violation will initiate the refund by calling TFTBridgeModule `create_refund_transaction_or_add_sig()` extrinsic to propose a `RefundTransaction`, to store the details in `RefundTransactions` storage map alongside with its signature and to emit `RefundTransactionsignatureAdded` and `RefundTransactionCreated` events.

4.  Other bridge validators that report the transaction later also provides their signature for that refund transaction proposal by calling same extrinsic.

5.  If the majority (more than the half) of bridge validators provided their signature for a refund transaction, a `RefundTransactionReady` event is emitted as well. This check happens every time the `create_refund_transaction_or_add_sig()` extrinsic is executed by validator call.

6.  The first bridge validator reporting the `RefundTransactionReady` event will handle it and query TFChain storage for the `RefundTransaction` details and the validators’ signatures. It will create a multi-signatures Stellar transaction with a [MEMO](https://developers.stellar.org/docs/encyclopedia/memos) of `RETURN` type containing the hash of the refunded transaction and submit it to Stellar network. If submitted successfully, it will call `set_refund_transaction_executed()` extrinsic (which removes the `RefundTransaction` from the `RefundTransactions` storage and adds it to `ExecutedRefundTransactions`) then emit `RefundTransactionProcessed` event.

#### Overview of the TFChain Refund events

1.  `tftBridgeModule.RefundTransactionCreated`: A bridge validator proposed a Refund-on-Stellar transaction after being the first to report a stellar deposit with invalid or missing cross-chain transfer information.
2.  `tftBridgeModule.RefundTransactionsignatureAdded`: Other bridge validators reported same stellar deposit and provided signature for the refund proposal.
3.  `tftBridgeModule.RefundTransactionReady`: Enough validators signatures were collected and stored so from now it is possible to submit the proposed stellar refund transaction.
4.  `tftBridgeModule.RefundTransactionProcessed`: A bridge validator called `set_refund_transaction_executed()` extrinsic with a proof that the proposed stellar refund transaction was executed successfully on stellar network.

### TFCHAIN -> STELLAR (Burn-and-Withdraw flow)

Now, we look into the details of transferring TFT from a TFChain account to a Stellar account. On TFChain network we burn TFT and on Stellar network we withdraw TFT from TFChain.

1.  To withdraw your asset back to Stellar, the TFTBridgeModule's `swap_to_stellar()` extrinsic in TFChain must be called with an amount to burn (on the TFChain side) and a Stellar account ID to receive the equivalent TFT amount (on the Stellar network side).

2.  The call validates the target Stellar account ID and ensures that you have enough balance in the source account. If so, it burns the amount and transfer fees to feeAccount, increments the `BurnTransactionId` in the TFTBridgeModule storage, stores data about the transaction with empty signatures placeholder, adds it to `BurnTransactions` with the `burnId` as key, and emits `BurnTransactionCreated` event. This event contains `burn_id`, `source` account, `target` Stellar address, and burn `amount`.

3.  The bridge validators are listening to this event. They extract the `burnId` and other transaction parameters, validate the Stellar address (tokens could be refunded/minted back on TFChain at this step if validation failed), then construct signed stellar transaction and extract the signature (note, the transaction can not be submitted yet to stellar network). They then call `propose_burn_transaction_or_add_sig()` extrinsic which fills their signatures and the bridge account sequence number in the `BurnTransaction` in storage that matches specified `burnId`. When this call executed, the `BurnTransactionSignatureAdded` event is emitted.

4.  If the majority (more than the half) of bridge validators provided their signature for the transaction, a `BurnTransactionReady` event is emitted as well. This check happens every time the `propose_burn_transaction_or_add_sig()` extrinsic is executed by validator call.

5.  The bridge will handle the event and query TFChain storage for the `BurnTransaction` details and the validators’ signatures. It will create a multi-signatures Stellar transaction and submit it to Stellar network. If submitted successfully, it will call `set_burn_transaction_executed()` extrinsic (which removes the `BurnTransaction` from the `BurnTransactions` storage and adds it to `ExecutedBurnTransactions`) then emit `BurnTransactionProcessed` event.

#### Overview of the TFChain Burning events

1.  `tftBridgeModule.BurnTransactionCreated`: A swap from TFChain to stellar was initiated by a call to `swap_to_stellar()` extrinsic.
2.  `tftBridgeModule.BurnTransactionSignatureAdded`: A bridge validator handled BurnTransactionCreated TFChain event and submitted its signature for the proposed stellar transaction.
3.  `tftBridgeModule.BurnTransactionReady`: Enough validators signatures were collected and stored so from now it is possible to submit the proposed stellar withdraw transaction.
4.  `tftBridgeModule.BurnTransactionProcessed`: A bridge validator was the first to call `set_burn_transaction_executed()` extrinsic and the proposed stellar withdraw transaction was executed successfully on stellar network.

#### When a Refund-on-TFChain occurs?

A refund on TFChain is initiated when either of the following conditions is met:

*   Account information cannot be retrieved from the Stellar network.
*   The account has no trust line to TFT tokens or has a deleted one (TFT balance limit is `0`).

### TFChain Retry mechanism

We didn't mentioned yet a few TFChain event related to the flows discussed above, these events are:

*   `tftBridgeModule.BurnTransactionExpired`: from TFCHAIN -> STELLAR burn flow, when transaction (burn TFT on TFChain source account and transfer amount from Stellar vault account to Stellar destination account) is stuck in bridge.
*   `tftBridgeModule.RefundTransactionExpired`: from STELLAR -> TFCHAIN refund flow, after TFT were locked into Stellar vault, finally not minted to TFChain, and transaction to get it back to Stellar source account is stuck in bridge.

These expired events are typically the result of an outage of one or more bridge validators. We will explain why.

TFChain has a retry mechanism built into its runtime that takes into account possible bridge validator outages. If a certain number of TFChain blocks pass without a `BurnTransaction` or `RefundTransaction` being noticed and signed by the majority of bridge validators, the stored transaction signatures are reset and a `BurnTransactionExpired` or `RefundTransactionExpired` event is emitted.

These events will continue to occur until the unavailable bridge validators come back online and handle the expired events as it gets re-emitted.
