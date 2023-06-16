# Third party service contract

Since we don't want to do work on the chain for every possible 3rd party service, we will keep this as generic as possible. While custom implementations for services might offer small advantages in the flow, the extra effort to develop and most importantly maintain these implementations is not worth it compared to a proper generic flow which would technically also be reusable.

## Contract structure

A contract will work simple client - server principle (i.e. the "buyer" and the "seller", a "consumer" of a service and one who "offers" the service). Both parties are identified by a twin id to fit in the current flow (we could use a generic address as well here). Contract is laid out as such

- consumer twin id
- service twin id
- base fee, this is the fixed amount (in mUSD) which will be billed hourly
- variable fee, this is the maximum amount (in mUSD) which can be billed on top of the base fee (for variable consumption metrics, to be defined by the service)
- metadata, a field which just holds some bytes. The service can use this any way it likes (including having stuff set by the user). We limit this field to some size now, suggested 64 bytes (2 public keys generally)

Additionally, we also keep track of some metadata, which will be:

- accepted by consumer
- accepted by service
- last bill received (we keep track of this to make sure the service does not send overlapping bills)

## Billing

Once a contract is accepted by both the consumer and the service, the chain can start accepting "bill reports" from the service for the contract. Only the twin of the service can send these, as specified in the contract. The bill contains the following:

- Variable amount (in mUSD) which is billed. The chain checks that this is less than or equal to the variable amount as specified in the contract, if it is higher the bill is rejected for overcharging. Technically the service could send a report every second to drain the user. To protect against this, the max variable amount in the contract is interpreted as being "per hour", and the value set in the contract is divided by 3600, multiplied by window size, to find the actual maximum which can be billed by the contract.
- Window, this is the amount of time (in seconds) covered since last contract. The chain verifies that `current time - window >= contract.last_bill`, such that no bills overlap to avoid overcharging. Combined with the previous limit to variable amount this prevents the service from overcharging the user.
- Some optional metadata, this will again just be some bytes (the service decides how this can be interpreted). For now we'll limit this to 50 bytes or so.

## Chain calls

### Callable by anyone

- `create_contract(consumer twin ID, service twin ID)`: Creates the contract and sets the id's. Base fee and variable fee are left at 0

### Callable by consumer or service

- `set_metadata(data)`: Sets the custom metadata on the contract. This can be done by either the client of the service, depending on how it is interpreted (as specified by the service). For now, we will assume that setting metadata is a one off operation. As a result, if metadata is already present when this is called, an error is thrown (i.e. only the first call of this function can succeed).

### Callable by service

- `set_fees(base, variable)`: Sets the base fee and variable fee on the contract (both in mUSD)
- `reject_by_service()`: Rejects the contract, deleting it.
- `approve_by_service()`: Sets the `service_accepted` flag on the contract. After this, no more modifications to fees or metadata can be done

### Callable by user

- `reject_by_consumer()`: Rejects the contract, deleting it.
- `approve_by_consumer()`: Sets the `consumer_accepted` flag on the contract. After this, no more modifications to fees or metadata can be done

## Flow

We start of by creating a contract. This can technically be done by anyone, but in practice will likely end up being done by either the service or the consumer (depending on what the service expects). This will be followed by the service or consumer setting the metadata (again depending on how the service expects things to be), and the service setting a base fee + variable fee. Note that part of the communication can and should be off chain, the contract is only the finalized agreement. When the fees and metadata are set, both the consumer and service need to explicitly approve the contract, setting the appropriate flag on the contract. Note that as soon as either party accepted (i.e. either flag is set), the fees and metadata cannot be changed anymore. It is technically possible for consumers to accept a contract as soon as it is created, thereby not giving the service a chance to set the fees. Though this basically means the contract is invalid and the service should just outright reject it.

Once the contract is accepted by both the consumer and the service, it can be billed (i.e. bills send before both flags are set must be rejected). Because a service should not charge the user if it doesn't work, we will require that bills be send every hour, by limiting the window size to 3600. Anything with a bigger window is rejected. This way if the service is down (for some longer period), it for sure can't bill for the time it was down. When the bill is received, the chain calculates `contract.base_fee * bill.window / 3600 + variable fee` (keeping in mind the constraint for variable fee as outlined above), and this amount is transferred from the consumer twin account to the service twin account.

We will not implement a grace period for this right now, as the service should define on an individual basis how this is handled. If needed in the future this can of course change.
