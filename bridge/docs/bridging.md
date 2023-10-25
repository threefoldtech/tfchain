# Bridging TFT between Stellar and TF Chain

## Usage

This document will explain how you can transfer TFT from TF Chain to Stellar and back.

## Prerequisites

- Threefold Connect application or any other Stellar wallet
- A running bridge and bridge wallet address

## Stellar to TF Chain

Transfer the TFT from your Stellar wallet to bridge wallet address that you configured. A depositfee of 1 TFT will be taken, so make sure you send a larger amount as 1 TFT.

### Transfer to TF Chain

We also enabled deposits to TF Grid objects. Following objects can be deposited to:

- Twin
- Farm
- Entity
- Node

To deposit to any of these objects, a memo text in format `object_objectID` must be passed on the deposit to the bridge wallet. Example: `twin_1`. 

To deposit to a TF Grid object, this object **must** exists. If the object is not found on chain, a refund is issued.

## TF Chain to Stellar

Browse to https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/extrinsics , select tftBridgeModule and extrinsic: `swap_to_stellar`. Provide your stellar target address and amount and sign it with your account holding the tft balance.
Again, a withdrawfee of 1 TFT will be taken, so make sure you send a larger amount as 1 TFT.

The amount withdrawn from TF Chain will be sent to your Stellar wallet.

Example: ![swap_to_stellar](swap_to_stellar.png)

## Deeper look at how the TFChain Bridge works

### STELLAR -> TFCHAIN (Lock-and-Mint flow)
In this section, we look into the details of transferring TFT from a Stellar Account to a TFChain Account. 

1. A transaction is received on the bridge Stellar account and witnessed by a bridge validator..

2. The transaction information undergoes validation. If it fails, a refund is issued (sent back to the stellar source account). now, We assume that the validation has passed, but we will examine the refund flow in the next section.

3. A mint is proposed by calling `propose_or_vote_mint_transaction` extrinsic on the TFTBridgeModule in tfchain. This extrinsic inserts a new `MintTransaction` in `MintTransactions` storage that includes the `amount`, `target`, `block`, `votes`, and emits a `MintTransactionProposed` event. The mint is considered processed by the bridge side at this point.

4. Other bridge validators also votes for that proposal by calling same extrinsic. If the `MintTransaction` exists in `MintTransactions` storage already, the extrinsic increments the `votes` count for the specified `MintTransaction` and emit `MintTransactionVoted` event.

5. From the TFChain side, if the majority (more than the half) of bridge validators agree on the transaction, tokens are minted to the target address. This check happens every time the `propose_or_vote_mint_transaction` extrinsic is executed by validator call. Then, the transaction is removed from bridge pallet `MintTransactions` storage and added to `ExecutedMintTransactions`. Finally, a `MintCompleted` event is emitted.

#### Overview of the TFChain Minting events
1. `tftBridgeModule.MintTransactionProposed`: A bridge validator has proposed a mint transaction upon witness a stellar deposit.
2. `tftBridgeModule.MintTransactionVoted`: Other bridge validators witness same stellar deposit and voted for the mint proposal.
3. `tftBridgeModule.MintCompleted`: Enough bridge validators' votes was collected and the tokens minted to the target address successfully.

#### When a Refund-on-Stellar occurs?

A refund on Stellar occurs when one of the following conditions is met:

- The deposited amount is lower than the deposit fee.
- The memo message is empty.
- The transaction contains more than one payment.
- The memo is not formatted correctly.
- The grid type is not supported (not one of grid, farm, node, or entity) or not found.

### STELLAR -> TFCHAIN (Refund-on-Stellar flow)
In this section, we look into the details of what happens when the a Stellar deposit can not be processed sue to a validation problem.

1. A transaction is received on the bridge Stellar account and witnessed by a bridge validator.

2. The transaction information undergoes validation. here we assume it failed because of one of the violations mentioned in the previous section, so a refund flow is initiated by the bridge by calling TFTBridgeModule `create_refund_transaction_or_add_sig` extrinsic to propose a `RefundTransaction` and store the details in `RefundTransactions` storage map alongside with bridge validator signature. `RefundTransactionsignatureAdded` and `RefundTransactionCreated` events.

3. Other bridge validators also provides their signature for that transaction proposal by calling same extrinsic.

4. If the majority (more than the half) of bridge validators provided their signature for a refund transaction, a `RefundTransactionReady` event is emitted as well. This check happens every time the `create_refund_transaction_or_add_sig` extrinsic is executed by validator call.

5. The bridge will handle the event and query TFChain storage for the `RefundTransaction` details and the validators’ signatures. It will create a multi-signatures Stellar transaction with a [MEMO](https://developers.stellar.org/docs/encyclopedia/memos) of `RETURN` type containing the hash of the refunded transaction and submit it to Stellar network. If submitted successfully, it will call `set_refund_transaction_executed` extrinsic (which removes the `RefundTransaction` from the `RefundTransactions` storage and adds it to `ExecutedRefundTransactions`) then emit `RefundTransactionProcessed` event.


#### Overview of the TFChain Refund events
1. `tftBridgeModule.RefundTransactionCreated`: A bridge validator has proposed a Refund-on-Stellar transaction upon witness a stellar deposit with invalid or missing cross-chain transfer information.
2. `tftBridgeModule.RefundTransactionsignatureAdded`: Other bridge validators witness same stellar deposit and voted for the refund proposal.
3. `tftBridgeModule.RefundTransactionReady`: Enough validators signatures was collected and stored so from now it is possible to submit the proposed stellar refund transaction.
4. `tftBridgeModule.RefundTransactionProcessed`: A bridge validator has called `set_refund_transaction_executed` extrinsic with a proof that the proposed stellar refund transaction was executed successfully on stellar network.

### TFCHAIN -> STELLAR (Burn-and-Withdraw flow)
now, we look into the details of transferring TFT from a TFChain Account to a Stellar.

1. To withdraw your asset back to Stellar, the TFTBridgeModule's `swap_to_stellar` extrinsic in TFChain must be called with an amount to burn (on the TFChain side) and a Stellar account ID to receive the equivalent TFT amount (on the Stellar network side).

2. The call validates the target Stellar account ID and ensures that you have enough balance in the source account. If so, it burns the amount and transfer fees to feeAccount, increments the `BurnTransactionId` in the TFTBridgeModule storage, store data about the transaction with empty signatures placeholder, adds it to `BurnTransactions` with the `burnId` as key, and emits `BurnTransactionCreated` event. This event contains `burn_id`, `source` account, `target` Stellar address, and burn `amount`.

3. The bridge validators are listening for this event. They extract the `burnId` and other transaction parameters, validate the Stellar address (tokens could refunded/minted back on TFChain at this step if validation failed), then construct signed stellar transaction and extract the signature (note, the transaction can not be submitted yet to stellar network). They then call `propose_burn_transaction_or_add_sig` extrinsic which fill their signatures and the bridge account sequence number in the `BurnTransaction` in storage that matches specified `burnId`. when this call executed, the `BurnTransactionSignatureAdded` event is emitted. 

4. If the majority (more than the half) of bridge validators provided their signature for a transaction, a `BurnTransactionReady` event is emitted as well. This check happens every time the `propose_burn_transaction_or_add_sig` extrinsic is executed by validator call.

5. The bridge will handle the event and query TFChain storage for the `BurnTransaction` details and the validators’ signatures. It will create a multi-signatures Stellar transaction and submit it to Stellar network. If submitted successfully, it will call `set_burn_transaction_executed` extrinsic (which removes the `BurnTransaction` from the `BurnTransactions` storage and adds it to `ExecutedBurnTransactions`) then emit `BurnTransactionProcessed` event.

#### Overview of the TFChain Burning events
1. `tftBridgeModule.BurnTransactionCreated`: A swap from TFChain to stellar was initiated by a call to `swap_to_stellar` extrinsic.
2. `tftBridgeModule.BurnTransactionSignatureAdded`: Other bridge validators handled `BurnTransactionCreated` TFChain event and submit their signature for the proposed stellar transaction.
3. `tftBridgeModule.BurnTransactionReady`: Enough validators signatures was collected and stored so from now it is possible to submit the proposed stellar withdraw transaction.
4. `tftBridgeModule.BurnTransactionProcessed`: A bridge validator has called `set_burn_transaction_executed` extrinsic with a proof that the proposed stellar withdraw transaction was executed successfully on stellar network.

#### When a Refund-on-TFChain occurs?

A refund on TFChain is initiated when either of the following conditions is met:

- Account information cannot be retrieved from the Stellar network.
- The account has no trust line to TFT tokens or has a deleted one (TFT balance limit is `0`).

### TFChain Retry mechanism

We didn't mentioned yet a few TFChain event related to the flows discussed above, these events are:
- `tftBridgeModule.BurnTransactionExpired`
- `tftBridgeModule.RefundTransactionExpired`

These expired events are typically the result of an outage of one or more bridge validators. We will explain why.

TFChain has a retry mechanism built into its runtime that takes into account possible bridge validator outages. If a certain number of TFChain blocks pass without a `BurnTransaction` or `RefundTransaction` being noticed and signed by the majority of bridge validators, the stored transaction signatures are reset and a `BurnTransactionExpired` or `RefundTransactionExpired` event is emitted.

These events will continue to occur until the unavailable bridge validators come back online and handle the expired events as it gets re-emitted.
