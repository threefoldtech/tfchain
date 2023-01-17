# Service/Consumer contracts

It is now possible to create generic contract between two TFChain users (without restriction of account type) for some service and bill for it.


## How does it work?

The initial scenario is when two parties, a service provider an a consumer of the service, want to use TFChain to automatically handle the billing/payment process for an agreement they want to make.
Multiple use cases can benefit from this feature, here are some examples of applications:

//TODO give some examples of applications

Initial requirements:
- Both service and consumer need to have their respective twin created
- Consumer account needs to be funded (lack of funds will simply result in the contract cancelation)

NB: A twin is automatically created when user first register to TFGrid via TFConnect app or Dashboard. For devnet purpose, see [here](docs/create_farm.md#step-5-create-a-twin) how to create a twin on TFChain.

In the following steps we detail the sequence of extrinsic that need to be called for executing such contract.
 

## Step 1: Create the contract and get its unique ID

The contract creation can be iniciated by both service or consumer using the following extrinsic:

~~~rust
service_contract_create(
    service_account: AccountId32,
    consumer_account: AccountId32,
)
~~~

Once executed the service contract is `Created` between the two parties.

⚠️ Important: during the execution of the contract creation, an event `ServiceContractCreated(service_contract)` is triggered with the contract object.
This object contain a unique ID (`service_contract_id`) which is essential to extract for being able to continue the flow

Be aware that calling the extrinsic a second time will create a new contract with a new ID.


## Step 2: Fill contract

Once created, the contract must be filled with its relative `per hour` fees (only service can set fees):

~~~rust
service_contract_set_fees(
    service_contract_id: u64,
    base_fee: u64,
    variable_fee: u64,
)
~~~

and also filled with some metadata with the description of the service for example (only service or consumer can set metadata):

~~~rust
service_contract_set_metadata(
    service_contract_id: u64,
    metadata: Bytes,
)
~~~

The agreement will be automatically considered `Ready` when both metadata and fees are set (`metadata` not empty and `base_fee` greater than zero).
Note that whenever this condition is not reached both extrinsics can still be called to modify agreement


## Step 3: Both parties approve contract

Now having the agreement ready the contract can be submited for approval.
One can approve the agreement using this extrinsic:

~~~rust
service_contract_approve(
    service_contract_id: u64,
)
~~~

and reject it using this following one:

~~~rust
service_contract_reject(
    service_contract_id: u64,
)
~~~

The contract need to be explicitly `Approved` by both service and consumer to be ready for billing.
Before reaching this state, if one of the parties decides to call the rejection extrinsic, it will instantly lead to the cancelation of the contract (and its permanent removal).


## Step 4: Bill for the service

Once the contract is accepted by both it can be billed.
Only the service can bill the consumer using the following extrincic:

~~~rust
service_contract_bill(
    service_contract_id: u64,
    variable_amount: u64,
    metadata: Bytes,
)
~~~

⚠️ Important: because a service should not charge the user if it doesn't work, it is required that bills be send in less than 1 hour intervals.
Any bigger interval will result in a bounded 1 hour bill (in other words, extra time will not be billed).
It is the service responsability to bill on right frequency!

When the bill is received, the chain calculates the bill amount based on the agreement values as follows: 

~~~
amount = base_fee * T / 3600 + variable_amount 
~~~

where `T` is the elapsed time, in seconds and bounded by 3600 (see above), since last effective billing operation occured.

Note that if `variable_amount` is too high (i.e `variable_amount >  variable_fee * T / 3600`) the billing extrinsic will fail.
The `variable_fee` value in the contract is interpreted as being "per hour" and acts as a protection mecanism to avoid consumer draining.
Indeed, as it is technically possible for the service to send a bill every second, there would be no gain for that (unless overloading the chain uselessly).
So it is also the service responsability to set a suitable `variable_amount` according to the billing frequency!

Also be aware that if the consumer is out of funds the billing will fail AND the contract will be canceled.

Then, if all goes well the consumer pays for the due amount calculated from the bill (see detail above).
In practice the amount is transferred from the consumer twin account to the service twin account.


## Step 5: Cancel the contract

At every moment of the flow since the contract is created it can be canceled (and definitively removed) by calling the following:

~~~rust
service_contract_cancel(
    service_contract_id: u64,
)
~~~
