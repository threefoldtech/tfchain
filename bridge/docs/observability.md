# Bridge Observability
The bridge is event-driven distributed system, it can be challenging to understand how requests or transfers move through the system and where bottlenecks may occur.
This is where techniques like distributed tracing, structured logs, and structured events comes in to allows developers and operators to monitor and understand the behavior of complex systems.

By using these techniques, developers and operators can gain a more complete understanding of how their applications are performing in production and quickly identify issues when they arise.

## Structured events
This section describes the structure and contents of the bridge logs.
It defines a common set of field names and data types, as well as descriptions and examples of how to use them.

Bridge logs are structured and represented in JSON format.
It uses a common structure for logs, making it easier to ingest, correlate, search, and aggregate on individual fields within your logs.

For interfaces designed for humans, they can simply display the "message" key and hide the other metadata, making the log easy to ready without sacrificing the metadata.

The logs entries serve as events (represents an immutable event that occurred in the past), and adopting past-tense verb naming schema.

### Events:
#### Operational Events:
These events to audit how the bridge system is operating.

##### Bridge availability events
- `bridge_init_aborted`: The bridge failed to initiated mostly due to misconfiguration.  
- `bridge_started`: The bridge started successfully.
- `bridge_unexpectedly_exited`: The bridge panicked.  
- `bridge_stopped`: The bridge stopped normally mostly for maintenance or new version enrollment.

##### Persistency events
- `stellar_cursor_saved`: The bridge saved the account operation cursor.

##### Stellar monitor events
- `transactions_fetched`: The bridge fetched transactions data from stellar network. Should be received periodically and can be used as a mark message to detect availability issues. Also include stellar cursor position.
- `fetch_transactions_failed`: The bridge failed to fetch transactions data from stellar network.

##### TFChain monitor events
- `block_events_fetched`: The bridge fetched transactions data from TFChain network for a block. Should be received every block and can be used as a mark message to detect availability issues. Also include the TFChain height.
- `fetch_finalized_Heads_failed`: The bridge failed to fetch transactions data from TFChain network.

#### Business Events:
These events are for business intelligence and always tied to a user transfer, and include a `trace_id` field.

you can use `trace_id` to correlate multiple log entries and track an transfer from one chain to the other. it is simply a stellar tx ID or TFChain burn ID depend on the side the transfer initiated from.

For example, if a customer is complaining that their deposit never bridged, you could filter logs using `trace_id` field with the id of the customer deposit to get overview of all events related to this deposit id and see what went wrong.

##### Cross-chain transfer events
- `transfer_initiated`: The bridge initiated a cross chain transfer (after receiving a deposit from stellar side or burn event from TFChain side).
- `transfer_completed`: The bridge has completed a cross chain transfer (either by bridging tokens to the other chain or issuing a refund if something went wrong).
- `transfer_failed`: a withdraw can not be completed and refund is not possible as well.

##### Cross-chain transfer phases
##### Mint related
- `mint_skipped`: a mint request skipped by the bridge instance as it has already been minted. 
- `mint_proposed`: a mint has proposed or voted by the bridge instance. 
- `mint_completed`: a mint has completed and received on the target TFChain account.

##### Refund related
- `event_refund_tx_ready_received`: The bridge instance has received TFChain `RefundTransactionReady` event which means all bridge validators signed a refund transaction. 
- `event_refund_tx_expired_received`: The bridge instance has received TFChain  `RefundTransactionExpired` event.
- `refund_skipped`: a refund request skipped by the bridge instance as it has already been refunded.
- `refund_proposed`: a refund has proposed or signed by the bridge instance.
- `refund_completed`: a refund has completed and received on the target stellar account.

##### Withdraw related 
- `event_burn_tx_created_received`: The bridge instance has received TFChain `BurnTransactionCreated` event.
- `event_burn_tx_ready_received`: The bridge instance has received TFChain `BurnTransactionReady` event which means all bridge validators signed a withdraw transaction.
- `event_burn_tx_expired_received`: The bridge instance has received TFChain `BurnTransactionExpired` event.
- `withdraw_skipped`: a refund request skipped by the bridge instance as it has already been refunded.
- `withdraw_proposed`: a withdraw has proposed or signed by the bridge instance. 
- `withdraw_completed`: a withdraw has completed and received on the target stellar account.

##### Bridge vault account related
- `payment_received` : This event represents successful payment to the bridge account (a deposit).
- `stellar_transaction_submitted` : This event represents successful transaction from the bridge account (a refund or a withdraw).

### Metrics:

This events describes a numeric measurement taken at given point in time.
Metric events are often collected on a predictable frequency.

#### Bridge vault account related
- `wallet_balance`: This event describes a tft balance locked on the bridge and collected once a minute.

### Log schema:

#### Base fields
The base field set contains all fields which are at the root of the events. These fields are common across all types of events.

| Field | Type | Description | Required |
| --- | --- | --- | --- |
| level | string | yes | The level or severity of the log event |
| version | number | yes |Log schema version |
| source | object | yes | Information about the bridge instance |
| event_action | string | yes | The action captured by the event |
| event_kind | string | yes | Can gives high-level information about what type of information the event contains, without being specific to the contents of the event. One of `Event`, `Alert`, `Error`, `metric` |
| metadata | object | no | Only present for logs that contain meta data about the event |
| message | string | yes | The log message |
| time | time | yes | The timestamp of the log event |
| error | string | no | Only present for logs that contain an exception or error. The message of the exception or error |
| category | string | no | One of `availability`, `persistency`, `stellar_monitor`, `tfchain_monitor`, `transfer`, `mint`, `refund`, `withdraw`, `vault` |
| trace_id | string | no | Only present on business events. a unique identifier that is assigned to a trace, which represents a complete cross-chain transfer flow. |

#### Categorization Fields
##### event_kind field
The value of this field can be used to inform how these kinds of events should be handled.

- alert: This value indicates an event such as an alert or notable event.

- event: This value is the most general and most common value for this field. It is used to represent events that indicate that something happened.

- error: This value indicates that an error occurred during the operation of the bridge.

- metric: This events describes a numeric measurement taken at given point in time.

##### category field
The value of this represents the "big buckets" of event categories

- availability
- persistency
- stellar_monitor
- tfchain_monitor
- transfer
- mint
- refund
- withdraw
- vault

#### source object
the source field set contains all fields which are included in the source object, it is common across all types of events except of `bridge_init_aborted` error event.

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| Instance_public_key | string | yes | Instance public key which you can use to filter logs by instance |
| Bridge_wallet_address | string | yes | The bridge account address which you can use to filter logs by bridge |
| Stellar_network | string | yes | Stellar network name which you can use to filter logs by environment |
| Tfchain_url | string | yes | The url of the substrate rpc node which you can use to filter logs by environment |

#### Event-specific fields:

##### bridge_init_aborted

- kind: error

- category: availability

<table>
    <thead>
        <tr><th colspan="4"><div>bridge_init_aborted Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td colspan="4"><div>No metadata</div></td>
        </tr>            
    </tbody>
</table>

##### bridge_started

- kind: event

- category: availability

<table>
    <thead>
        <tr><th colspan="4"><div>bridge_started Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td><code>rescan_flag</code></td>
            <td>bool</td>
            <td>yes</td>
            <td>The value of the bridge configuration flag which used to instruct the bridge to scan the vault account from the earliest known operation. </td>
        </tr>
        <tr>
            <td><code>deposit_fee</code></td>
            <td>number</td>
            <td>yes</td>
            <td>The bridge fees to charge per deposit which fetched from TFChain. </td>
        </tr>            
    </tbody>
</table>

##### bridge_unexpectedly_exited

- kind: error

- category: availability

<table>
    <thead>
        <tr><th colspan="4"><div>bridge_unexpectedly_exited Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td colspan="4"><div>No metadata</div></td>
        </tr>            
    </tbody>
</table>

##### bridge_stopped

- kind: event

- category: availability

<table>
    <thead>
        <tr><th colspan="4"><div>bridge_stopped Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td colspan="4"><div>No metadata</div></td>
        </tr>            
    </tbody>
</table>

##### stellar_cursor_saved

- kind: event

- category: persistency

<table>
    <thead>
        <tr><th colspan="4"><div>stellar_cursor_saved Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td><code>cursor</code></td>
            <td>number</td>
            <td>yes</td>
            <td>the Cursor is an integer that points to a specific location in a collection of horizon responses. </td>
        </tr>         
    </tbody>
</table>

##### transactions_fetched

- kind: event

- category: stellar_monitor

<table>
    <thead>
        <tr><th colspan="4"><div>transactions_fetched Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td><code>cursor</code></td>
            <td>number</td>
            <td>yes</td>
            <td>the Cursor is an integer that points to a specific location in a collection of horizon responses. </td>
        </tr>
        <tr>
            <td><code>count</code></td>
            <td>number</td>
            <td>yes</td>
            <td>The count of the fetched transactions from horizon.</td>
        </tr>         
    </tbody>
</table>

##### fetch_transactions_failed

- kind: alert

- category: stellar_monitor

<table>
    <thead>
        <tr><th colspan="4"><div>fetch_transactions_failed Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td><code>cursor</code></td>
            <td>number</td>
            <td>yes</td>
            <td>the Cursor is an integer that points to a specific location in a collection of horizon responses. </td>
        </tr>            
    </tbody>
</table>

##### block_events_fetched

- kind: event

- category: tfchain_monitor

<table>
    <thead>
        <tr><th colspan="4"><div>block_events_fetched Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td><code>height</code></td>
            <td>number</td>
            <td>yes</td>
            <td>The block height of a TFChain</td>
        </tr>            
    </tbody>
</table>

##### fetch_finalized_Heads_failed

- kind: alert

- category: tfchain_monitor

<table>
    <thead>
        <tr><th colspan="4"><div>fetch_finalized_Heads_failed Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td colspan="4"><div>No metadata</div></td>
        </tr>            
    </tbody>
</table>

##### transfer_initiated

- kind: event

- category: transfer

<table>
    <thead>
        <tr><th colspan="4"><div>transfer_initiated Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td><code>type</code></td>
            <td>string</td>
            <td>yes</td>
            <td>The type of this transfer. One of `burn` (from tfchain side) or `deposit` (from stellar side). </td>
        </tr>              
    </tbody>
</table>

##### transfer_completed

- kind: event

- category: transfer

<table>
    <thead>
        <tr><th colspan="4"><div>transfer_completed Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td><code>outcome</code></td>
            <td>string</td>
            <td>yes</td>
            <td>One of `refunded` or `bridged`. </td>
        </tr>            
    </tbody>
</table>

##### transfer_failed

- kind: alert

- category: transfer

<table>
    <thead>
        <tr><th colspan="4"><div>transfer_failed Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td><code>reason</code></td>
            <td>string</td>
            <td>yes</td>
            <td>The reason behind the failure of this transfer. </td>
        </tr>
        <tr>
            <td><code>type</code></td>
            <td>string</td>
            <td>yes</td>
            <td>The type of this transfer. One of `burn` (from tfchain side) or `deposit` (from stellar side). </td>
        </tr>            
    </tbody>
</table>

##### mint_skipped

- kind: event

- category: mint

<table>
    <thead>
        <tr><th colspan="4"><div>mint_skipped Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td colspan="4"><div>No metadata</div></td>
        </tr>            
    </tbody>
</table>

##### mint_proposed

- kind: event

- category: mint

<table>
    <thead>
        <tr><th colspan="4"><div>mint_proposed Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td><code>amount</code></td>
            <td>number</td>
            <td>yes</td>
            <td>deposited amount to be minted on tfchain. </td>
        </tr>  
        <tr>
            <td><code>tx_id</code></td>
            <td>number</td>
            <td>yes</td>
            <td>The stellar deposit tx ID. </td>
        </tr>
        <tr>
            <td><code>to</code></td>
            <td>string</td>
            <td>yes</td>
            <td>The tfchain target account address. </td>
        </tr>           
    </tbody>
</table>

##### mint_completed

- kind: event

- category: mint

<table>
    <thead>
        <tr><th colspan="4"><div>mint_completed Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td colspan="4"><div>No metadata</div></td>
        </tr>            
    </tbody>
</table>

##### event_refund_tx_ready_received

- kind: event

- category: refund

<table>
    <thead>
        <tr><th colspan="4"><div>event_refund_tx_ready_received Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td colspan="4"><div>No metadata</div></td>
        </tr>            
    </tbody>
</table>

##### event_refund_tx_expired_received

- kind: alert

- category: refund

<table>
    <thead>
        <tr><th colspan="4"><div>event_refund_tx_expired_received Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td colspan="4"><div>No metadata</div></td>
        </tr>            
    </tbody>
</table>

##### refund_skipped

- kind: event

- category: refund

<table>
    <thead>
        <tr><th colspan="4"><div>refund_skipped Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td colspan="4"><div>No metadata</div></td>
        </tr>            
    </tbody>
</table>

##### refund_proposed

- kind: event

- category: refund

<table>
    <thead>
        <tr><th colspan="4"><div>refund_proposed Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td><code>reason</code></td>
            <td>string</td>
            <td>yes</td>
            <td>The reason for refunding this transaction. </td>
        </tr>            
    </tbody>
</table>

##### refund_completed

- kind: event

- category: refund

<table>
    <thead>
        <tr><th colspan="4"><div>refund_completed Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td colspan="4"><div>No metadata</div></td>
        </tr>            
    </tbody>
</table>

##### event_burn_tx_created_received

- kind: event

- category: withdraw

<table>
    <thead>
        <tr><th colspan="4"><div>event_burn_tx_created_received Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td colspan="4"><div>No metadata</div></td>
        </tr>            
    </tbody>
</table>

##### event_burn_tx_ready_received

- kind: event

- category: withdraw

<table>
    <thead>
        <tr><th colspan="4"><div>event_burn_tx_ready_received Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td colspan="4"><div>No metadata</div></td>
        </tr>            
    </tbody>
</table>

##### event_burn_tx_expired_received

- kind: alert

- category: withdraw

<table>
    <thead>
        <tr><th colspan="4"><div>event_burn_tx_expired_received Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td colspan="4"><div>No metadata</div></td>
        </tr>            
    </tbody>
</table>

##### withdraw_skipped

- kind: event

- category: withdraw

<table>
    <thead>
        <tr><th colspan="4"><div>withdraw_skipped Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td colspan="4"><div>No metadata</div></td>
        </tr>            
    </tbody>
</table>

##### withdraw_proposed

- kind: event

- category: withdraw

<table>
    <thead>
        <tr><th colspan="4"><div>withdraw_proposed Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td><code>amount</code></td>
            <td>number</td>
            <td>yes</td>
            <td>Burned amount. </td>
        </tr>  
        <tr>
            <td><code>tx_id</code></td>
            <td>number</td>
            <td>yes</td>
            <td>The burn ID. </td>
        </tr>
        <tr>
            <td><code>to</code></td>
            <td>string</td>
            <td>yes</td>
            <td>The stellar target account address. </td>
        </tr>            
    </tbody>
</table>

##### withdraw_completed

- kind: event

- category: withdraw

<table>
    <thead>
        <tr><th colspan="4"><div>withdraw_completed Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td colspan="4"><div>No metadata</div></td>
        </tr>            
    </tbody>
</table>

##### payment_received

- kind: event

- category: vault

<table>
    <thead>
        <tr><th colspan="4"><div>payment_received Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td><code>from</code></td>
            <td>string</td>
            <td>yes</td>
            <td>The stellar source account address</td>
        </tr>
        <tr>
            <td><code>amount</code></td>
            <td>decimal</td>
            <td>yes</td>
            <td>Deposit amount</td>
        </tr>
        <tr>
            <td><code>tx_hash</code></td>
            <td>string</td>
            <td>yes</td>
            <td>transaction hash</td>
        </tr>
        <tr>
            <td><code>ledger_close_time</code></td>
            <td>time</td>
            <td>yes</td>
            <td>transaction time</td>
        </tr>                   
    </tbody>
</table>

##### stellar_transaction_submitted

- kind: event

- category: vault

<table>
    <thead>
        <tr><th colspan="4"><div>stellar_transaction_submitted Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td><code>result_tx_id</code></td>
            <td>string</td>
            <td>yes</td>
            <td>The stellar id of the bridge executed transaction. </td>
        </tr>            
    </tbody>
</table>

##### wallet_balance

- kind: metric

- category: vault

<table>
    <thead>
        <tr><th colspan="4"><div>wallet_balance Event Properties</div></th></tr>
        <tr>
            <th>Property</th>
            <th>Type</th>
            <th>Required</th>
            <th>Description</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td><code>tft</code></td>
            <td>number</td>
            <td>yes</td>
            <td>The tft amount locked in the bridge vault account. collected once a minute</td>
        </tr>            
    </tbody>
</table>

## Usage examples:

### Example 1

One example, if a customer is complaining that their deposit never bridged, you could filter logs using `trace_id` field with the id of the customer deposit to get overview of all events related to this deposit id and see what went wrong.

    - trace_id = `16d8b06b59aaa5514c645260263e5477bb8aad211502c56cb8849ed5b423d354`

The result would be an array of well defined events, and for a well behave cross-transfer from stellar network to tfchain network it should include these events in the same order:

*payment_received* --> *transfer_initiated* --> *mint_proposed* --> *mint_completed* --> *transfer_completed*

the filtered result would be similar to the one below: 

```
[
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "16d8b06b59aaa5514c645260263e5477bb8aad211502c56cb8849ed5b423d354",
    "event_action": "payment_received",
    "event_kind": "event",
    "category": "vault",
    "metadata": {
      "from": "GD4MUF7FTWOGNREGKMQWC3NOJGBNASEFNEOUJTLNW4FDONV5CEUTGKS4",
      "amount": "5.0000000"
    },
    "tx_hash": "16d8b06b59aaa5514c645260263e5477bb8aad211502c56cb8849ed5b423d354",
    "ledger_close_time": "2023-11-05 17:08:28 +0000 UTC",
    "time": "2023-11-05T19:08:32+02:00",
    "message": "a payment has received on bridge Stellar account"
  },
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "16d8b06b59aaa5514c645260263e5477bb8aad211502c56cb8849ed5b423d354",
    "event_action": "transfer_initiated",
    "event_kind": "event",
    "category": "transfer",
    "metadata": {
      "type": "deposit"
    },
    "time": "2023-11-05T19:08:33+02:00",
    "message": "a transfer has initiated"
  },
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "16d8b06b59aaa5514c645260263e5477bb8aad211502c56cb8849ed5b423d354",
    "event_action": "mint_proposed",
    "event_kind": "event",
    "category": "mint",
    "metadata": {
      "amount": 50000000,
      "tx_id": "16d8b06b59aaa5514c645260263e5477bb8aad211502c56cb8849ed5b423d354",
      "to": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
    },
    "time": "2023-11-05T19:08:36+02:00",
    "message": "a mint has proposed with the target substrate address of 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
  },
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "16d8b06b59aaa5514c645260263e5477bb8aad211502c56cb8849ed5b423d354",
    "event_action": "mint_completed",
    "event_kind": "event",
    "category": "mint",
    "time": "2023-11-05T19:08:49+02:00",
    "message": "found MintCompleted event"
  },
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "16d8b06b59aaa5514c645260263e5477bb8aad211502c56cb8849ed5b423d354",
    "event_action": "transfer_completed",
    "event_kind": "event",
    "category": "transfer",
    "metadata": {
      "outcome": "bridged"
    },
    "time": "2023-11-05T19:08:49+02:00",
    "message": "transfer has completed"
  }
]
```

The `transfer_completed` event’s `outcome` field value of `bridged` indicates that the TFT was successfully transferred. 

### Example 2

For a cross-chain transfer from TFChain to Stellar, the trace_id will be an integer.
    
    - trace_id = `10`

Let’s examine the event actions for this transfer:

*event_burn_tx_created_received* --> *transfer_initiated* --> *mint_proposed* --> *mint_completed* --> *transfer_completed*

This time, the transfer was not completed on the other network and was instead refunded. However, using the `trace_id`, you can still trace the transfer from start to end. The filtered result would be similar to the one below:


```
[
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "10",
    "event_action": "event_burn_tx_created_received",
    "event_kind": "event",
    "category": "withdraw",
    "time": "2023-11-05T20:16:31+02:00",
    "message": "found BurnTransactionCreated event"
  },
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "10",
    "event_action": "transfer_initiated",
    "event_kind": "event",
    "category": "transfer",
    "metadata": {
      "type": "burn"
    },
    "time": "2023-11-05T20:16:31+02:00",
    "message": "a transfer has initiated"
  },
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "10",
    "event_action": "mint_proposed",
    "event_kind": "event",
    "category": "mint",
    "metadata": {
      "amount": 40000000,
      "tx_id": "10",
      "to": "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
    },
    "time": "2023-11-05T20:16:36+02:00",
    "message": "a mint has proposed with the target substrate address of 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
  },
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "10",
    "event_action": "mint_completed",
    "event_kind": "event",
    "category": "mint",
    "time": "2023-11-05T20:16:50+02:00",
    "message": "found MintCompleted event"
  },
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "10",
    "event_action": "transfer_completed",
    "event_kind": "event",
    "category": "transfer",
    "metadata": {
      "outcome": "refunded"
    },
    "time": "2023-11-05T20:16:50+02:00",
    "message": "transfer has completed"
  }
]
```

Notably, the `transfer_completed` event’s `outcome` field value of refunded indicates that the TFT was refunded to the source account.

### Example 3

Here is another example of a cross-chain transfer from TFChain to Stellar, where the events show that the transfer was successful.
    
    - trace_id = `13`

*event_burn_tx_created_received* --> *transfer_initiated* --> *withdraw_proposed* --> *event_burn_tx_ready_received* --> *stellar_transaction_submitted* --> *withdraw_completed* --> *transfer_completed*

For a more simplified view, you can filter events by the transfer category to display only the start and end events of the transfer in question.

```
[
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "13",
    "event_action": "event_burn_tx_created_received",
    "event_kind": "event",
    "category": "withdraw",
    "time": "2023-11-05T20:57:08+02:00",
    "message": "found BurnTransactionCreated event"
  },
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "13",
    "event_action": "transfer_initiated",
    "event_kind": "event",
    "category": "transfer",
    "metadata": {
      "type": "burn"
    },
    "time": "2023-11-05T20:57:08+02:00",
    "message": "a transfer has initiated"
  },
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "13",
    "event_action": "withdraw_proposed",
    "event_kind": "event",
    "category": "withdraw",
    "metadata": {
      "amount": 40000000,
      "tx_id": "13",
      "to": "GBK4SQ5HUMWKMSYVEAFPPO4W27YRPGHE4CGQOKEFQ3WGPTSNURZPISO3"
    },
    "time": "2023-11-05T20:57:12+02:00",
    "message": "a withdraw has proposed with the target stellar address of GBK4SQ5HUMWKMSYVEAFPPO4W27YRPGHE4CGQOKEFQ3WGPTSNURZPISO3"
  },
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "13",
    "event_action": "event_burn_tx_ready_received",
    "event_kind": "event",
    "category": "withdraw",
    "time": "2023-11-05T20:57:25+02:00",
    "message": "found BurnTransactionReady event"
  },
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "13",
    "event_action": "stellar_transaction_submitted",
    "event_kind": "event",
    "category": "vault",
    "metadata": {
      "result_tx_id": "777f561a4b91928f4679ad182be2178a29d5f0a3ee28a0461708d183d0a00a7d"
    },
    "time": "2023-11-05T20:57:32+02:00",
    "message": "the transaction submitted to the Stellar network, and its unique identifier is 777f561a4b91928f4679ad182be2178a29d5f0a3ee28a0461708d183d0a00a7d"
  },
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "13",
    "event_action": "withdraw_completed",
    "event_kind": "event",
    "category": "withdraw",
    "time": "2023-11-05T20:57:32+02:00",
    "message": "the withdraw has proceed"
  },
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "13",
    "event_action": "transfer_completed",
    "event_kind": "event",
    "category": "transfer",
    "metadata": {
      "outcome": "bridged"
    },
    "time": "2023-11-05T20:57:32+02:00",
    "message": "the transfer has completed"
  }
]
```

### Example 4

The final example illustrates the expected events when a transfer from Stellar to TFChin fails. We will filter the events using the `trace_id`, which is the deposit transaction ID.

    - trace_id = `7f0406ad7b8d4f0de6dade19eb3979ef93857a56c6daa4bf9f2b0bb22a21d84f`

The `transfer_completed` event contains the `outcome` of the transfer, which is refunded.

Also Upon reviewing the `refund_proposed` event, we found that the `reason` field indicates that the memo was not properly formatted.

```
[
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "7f0406ad7b8d4f0de6dade19eb3979ef93857a56c6daa4bf9f2b0bb22a21d84f",
    "event_action": "payment_received",
    "event_kind": "event",
    "category": "vault",
    "metadata": {
      "from": "GD4MUF7FTWOGNREGKMQWC3NOJGBNASEFNEOUJTLNW4FDONV5CEUTGKS4",
      "amount": "5.0000000"
    },
    "tx_hash": "7f0406ad7b8d4f0de6dade19eb3979ef93857a56c6daa4bf9f2b0bb22a21d84f",
    "ledger_close_time": "2023-11-05 19:05:48 +0000 UTC",
    "time": "2023-11-05T21:05:57+02:00",
    "message": "a payment has received on bridge Stellar account"
  },
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "7f0406ad7b8d4f0de6dade19eb3979ef93857a56c6daa4bf9f2b0bb22a21d84f",
    "event_action": "transfer_initiated",
    "event_kind": "event",
    "category": "transfer",
    "metadata": {
      "type": "deposit"
    },
    "time": "2023-11-05T21:05:57+02:00",
    "message": "a transfer has initiated"
  },
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "7f0406ad7b8d4f0de6dade19eb3979ef93857a56c6daa4bf9f2b0bb22a21d84f",
    "event_action": "refund_proposed",
    "event_kind": "event",
    "category": "refund",
    "metadata": {
      "reason": "memo is not properly formatted"
    },
    "time": "2023-11-05T21:06:00+02:00",
    "message": "a refund has proposed due to memo is not properly formatted"
  },
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "7f0406ad7b8d4f0de6dade19eb3979ef93857a56c6daa4bf9f2b0bb22a21d84f",
    "event_action": "event_refund_tx_ready_received",
    "event_kind": "event",
    "category": "refund",
    "time": "2023-11-05T21:06:12+02:00",
    "message": "found RefundTransactionReady event"
  },
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "7f0406ad7b8d4f0de6dade19eb3979ef93857a56c6daa4bf9f2b0bb22a21d84f",
    "event_action": "stellar_transaction_submitted",
    "event_kind": "event",
    "category": "vault",
    "metadata": {
      "result_tx_id": "161c06d9ebd518beee0147c5e9e8b67c851f1c443b30444aff415668e76b09de"
    },
    "time": "2023-11-05T21:06:16+02:00",
    "message": "the transaction submitted to the Stellar network, and its unique identifier is 161c06d9ebd518beee0147c5e9e8b67c851f1c443b30444aff415668e76b09de"
  },
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "7f0406ad7b8d4f0de6dade19eb3979ef93857a56c6daa4bf9f2b0bb22a21d84f",
    "event_action": "refund_completed",
    "event_kind": "event",
    "category": "refund",
    "time": "2023-11-05T21:06:18+02:00",
    "message": "the transaction has refunded"
  },
  {
    "level": "info",
    "version": 1,
    "source": {
      "Instance_public_key": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Bridge_wallet_address": "GBD3PXJEQOCQ5VR2JSMFNXYBBQF5RDEZP5GMTXDYZWMNZQJHR6HFX3AJ",
      "Stellar_network": "testnet",
      "Tfchain_url": "ws://localhost:9944"
    },
    "trace_id": "7f0406ad7b8d4f0de6dade19eb3979ef93857a56c6daa4bf9f2b0bb22a21d84f",
    "event_action": "transfer_completed",
    "event_kind": "event",
    "category": "transfer",
    "metadata": {
      "outcome": "refunded"
    },
    "time": "2023-11-05T21:06:18+02:00",
    "message": "the transfer has completed"
  }
]
```
