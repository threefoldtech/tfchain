# TFChain Bridge

TFChain bridge is a Chain-To-Chain bridge which mainly designed to support the movement of assets between two blockchains, TFChain and Stellar.
It is composed as a daemon that runs in the background scanning a Stellar vault address for deposits and TFChain Events for withdrawals and executes upon them. This allows cross-chain transfers between Stellar <> TFChain.

## what is a blockchain bridge?

Blockchain bridges connect two different blockchains, similar to how real-world bridges connect two different locations. Without a bridge, blockchains are siloed environments that cannot communicate with each other because each network has its own set of rules, governance mechanisms, native assets, and data that are incompatible with the other blockchains. However, with a bridge between two blockchains, it becomes possible to transfer crypto-assets and arbitrary data between them. Bridges are key for interoperability in the crypto ecosystem and are necessary to make different blockchain networks compatible with each other.

## Cross-Chain Mechanism

Bridges can be categorized by a number of characteristics. These include how they transfer information across chains which consider the most important factor.

Our bridge between Stellar TFT and TFChain TFT use a mechanism known as “locking” or “burning,” followed by either minting or withdrawing, respectively. let's describe how the mechanism works by using an example.
- the user begins by depositing the Stellar TFT version into a designated stellar address owned by the bridge and specifying the recipient on TFChain. This step is referred to as “locking.”.
- the bridge initiate a flow to “mints” or issues a version of the deposited asset on TFChain and credits it to the recipient account.
- When the user wants to move back to Stellar TFT, the TFChain token is simply “burned.” This allows the underlying asset on Stellar to be redeemed and sent to the specified recipient address.

## Development

In this document we will explain how the bridge works and how you can setup a local instance to develop against.
The local instance will consist of a connection between a tfchain that runs in development mode and Stellar Testnet.

See [architecture](./architecture.md) for more information on how the bridge works.

## Setup

### Development setup

Refer to [development](./development.md) for more information on how to setup a development instance.

### Production setup

Refer to [production](./production.md) for more information on how to setup a production instance.

### Bridging

When you have setup the bridge in either development or production mode you can start bridging.

See [bridging](./bridging.md) for more information on how to bridge.

## Deeper look at how the TFChain Bridge works

### STELLAR -> TFCHAIN (Lock-and-Mint flow)
In this section, we look into the details of transferring TFT from a Stellar Account to a TFChain Account. 

1. A transaction is received on the bridge Stellar account and witnessed by a bridge validator..

2. The transaction information undergoes validation. If it fails, a refund is issued (sent back to the stellar source account). now, We assume that the validation has passed, but we will examine the refund flow in the next section.

3. A mint is proposed by calling `propose_or_vote_mint_transaction` extrinsic on the TFTBridgeModule in tfchain. This extrinsic inserts a new `MintTransaction` in `MintTransactions` storage that includes the `amount`, `target`, `block`, `votes`, and emits a `MintTransactionProposed` event. The mint is considered processed by the bridge side at this point.

4. Other bridge validators also votes for that proposal by calling same extrinsic. If the `MintTransaction` exists in `MintTransactions` storage already, the extrinsic increments the `votes` count for the specified `MintTransaction` and emit `MintTransactionVoted` event.

5. From the TFChain side, if the majority (more than the half) of bridge validators agree on the transaction, tokens are minted to the target address. This check happens every time the `propose_or_vote_mint_transaction` extrinsic is executed by validator call. Then, the transaction is removed from bridge pallet `MintTransactions` storage and added to `ExecutedMintTransactions`. Finally, a `MintCompleted` event is emitted.

#### Overview of the TFChain Minting events
1. tftBridgeModule.MintTransactionProposed: A bridge validator has proposed a mint transaction upon witness a stellar deposit.
2. tftBridgeModule.MintTransactionVoted: Other bridge validators witness same stellar deposit and voted for the mint proposal.
3. tftBridgeModule.MintCompleted: Enough bridge validators' votes was collected and the tokens minted to the target address successfully.

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
1. tftBridgeModule.RefundTransactionCreated: A bridge validator has proposed a Refund-on-Stellar transaction upon witness a stellar deposit with invalid or missing cross-chain transfer information.
2. tftBridgeModule.RefundTransactionsignatureAdded: Other bridge validators witness same stellar deposit and voted for the refund proposal.
3. tftBridgeModule.RefundTransactionReady: Enough validators signatures was collected and stored so from now it is possible to submit the proposed stellar refund transaction.
4. tftBridgeModule.RefundTransactionProcessed: A bridge validator has called `set_refund_transaction_executed` extrinsic with a proof that the proposed stellar refund transaction was executed successfully on stellar network.

### TFCHAIN -> STELLAR (Burn-and-Withdraw flow)
now, we look into the details of transferring TFT from a TFChain Account to a Stellar.

1. To withdraw your asset back to Stellar, the TFTBridgeModule's `swap_to_stellar` extrinsic in TFChain must be called with an amount to burn (on the TFChain side) and a Stellar account ID to receive the equivalent TFT amount (on the Stellar network side).

2. The call validates the target Stellar account ID and ensures that you have enough balance in the source account. If so, it burns the amount and transfer fees to feeAccount, increments the `BurnTransactionId` in the TFTBridgeModule storage, store data about the transaction with empty signatures placeholder, adds it to `BurnTransactions` with the `burnId` as key, and emits `BurnTransactionCreated` event. This event contains `burn_id`, `source` account, `target` Stellar address, and burn `amount`.

3. The bridge validators are listening for this event. They extract the `burnId` and other transaction parameters, validate the Stellar address (tokens could refunded/minted back on TFChain at this step if validation failed), then construct signed stellar transaction and extract the signature (note, the transaction can not be submitted yet to stellar network). They then call `propose_burn_transaction_or_add_sig` extrinsic which fill their signatures and the bridge account sequence number in the `BurnTransaction` in storage that matches specified `burnId`. when this call executed, the `BurnTransactionSignatureAdded` event is emitted. 

4. If the majority (more than the half) of bridge validators provided their signature for a transaction, a `BurnTransactionReady` event is emitted as well. This check happens every time the `propose_burn_transaction_or_add_sig` extrinsic is executed by validator call.

5. The bridge will handle the event and query TFChain storage for the `BurnTransaction` details and the validators’ signatures. It will create a multi-signatures Stellar transaction and submit it to Stellar network. If submitted successfully, it will call `set_burn_transaction_executed` extrinsic (which removes the `BurnTransaction` from the `BurnTransactions` storage and adds it to `ExecutedBurnTransactions`) then emit `BurnTransactionProcessed` event.

#### Overview of the TFChain Burning events
1. tftBridgeModule.BurnTransactionCreated: A swap from TFChain to stellar was initiated by a call to `swap_to_stellar` extrinsic.
2. tftBridgeModule.BurnTransactionSignatureAdded: Other bridge validators handled `BurnTransactionCreated` TFChain event and submit their signature for the proposed stellar transaction.
3. tftBridgeModule.BurnTransactionReady: Enough validators signatures was collected and stored so from now it is possible to submit the proposed stellar withdraw transaction.
4. tftBridgeModule.BurnTransactionProcessed: A bridge validator has called `set_burn_transaction_executed` extrinsic with a proof that the proposed stellar withdraw transaction was executed successfully on stellar network.

#### When a Refund-on-TFChain occurs?

A refund on TFChain is initiated when either of the following conditions is met:

- Account information cannot be retrieved from the Stellar network.
- The account has no trust line to TFT tokens or has a deleted one (TFT balance limit is `0`).

### TFChain Retry mechanism

We didn't mentioned yet a few TFChain event related to the flows discussed above, these events are:
- `BurnTransactionExpired`
- `RefundTransactionExpired`
- `MintTransactionExpired`

These expired events are typically the result of an outage of one or more bridge validators. We will explain why.

TFChain has a retry mechanism built into its runtime that takes into account possible bridge validator outages. If a certain number of TFChain blocks pass without a `BurnTransaction` or `RefundTransaction` being noticed and signed by the majority of bridge validators, the stored transaction signatures are reset and a `BurnTransactionExpired` or `RefundTransactionExpired` event is emitted.

The same happens when a `MintTransaction` stalls for a certain number of TFChain blocks without being noticed and voted by the majority of bridge validators, and an `MintTransactionExpired` event is emitted.

These events will continue to occur until the unavailable bridge validators come back online and handle the expired events as it gets re-emitted.
