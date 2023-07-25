# Threefold Key-value Store Pallet

The Key-value Store Pallet is a module that enables storing key-value pairs on the blockchain, where the key is a unique identifier and the value is the data payload.

Some use cases for this module are storing user profiles, preferences, or settings; storing metadata or configuration data for other modules or dapps; storing encrypted or hashed data for privacy or security purposes. The Key-value Store Pallet provides a simple and efficient way to store and access arbitrary data on the blockchain, without relying on any intermediaries or centralized servers.

## Overview

The Key-value Store Pallet provides [dictionary-like data storage](https://paritytech.github.io/substrate/master/frame_support/storage/trait.StorageDoubleMap.html#) and functions for:
- Storing key-value pairs.
- Retrieving the value associated with the specified key.
- Removing a value associated with the specified key.

The key value store uses under hood a map with two keys

*(Key1, Key2)  ->  Value*

**Key1** is provided by the runtime set to the account that signed the extrinsic. **Key2** and **Value** are user provided

You can think of the key1 as namespace that make it easier to come up with unique names which distinguished from other accounts' names.      

The key value store pallet impose restrictions on the size of the keys and values.  see [Assumptions](#assumptions) section.

## Terminology
- Key-value store: A key-value store, or key-value database is a simple database that uses an associative array (think of a map or dictionary) as the fundamental data model where each key is associated with one and only one value in a collection. This relationship is referred to as a key-value pair.

## Implementations
NA

## Interface
### Dispatchable Functions
- `set(key: Vec<u8>, value: Vec<u8>)` store value for a pair of keys (key1, key2) in the store. The first key is automatically set to the account ID of the signer of the transaction, while the second key is given by the user. This way, you can assign ownership of a key to the account that created it, and also use freely any name for a key that might be already taken by someone else. If the key pair already exists in the store, the value is overwritten with the new one. Emits `EntrySet` event.

- `delete(key: Vec<u8>)` remove a value stored under a pair of keys (key1, key2) from the store. The first key is automatically set to the account ID of the sender of the transaction, while the second key is given by the user. so by design, only the owner of a key pair can delete it from the store. Emits `EntryTaken` event.


### Events
- `EntrySet(T::AccountId, Vec<u8>, Vec<u8>)` The value for the specified key has been stored/updated.
- `EntryGot(T::AccountId, Vec<u8>, Vec<u8>)` The value for the specified key has been queried. (This Event Defined But Not Used By the runtime)
- `EntryTaken(T::AccountId, Vec<u8>, Vec<u8>)` The value stored under the specified key has been removed.

All events included account id, the user key name and the value.

### Errors
- `NoValueStored` The double key (Account ID, Provided Key) is not in the `StorageDoubleMap`
- `KeyIsTooLarge` The key length exceed the maximum length. see [Assumptions](#assumptions) section.
- `ValueIsTooLarge` The key length exceed the maximum length. see [Assumptions](#assumptions) section.

## Config
```rust
pub trait Config: Config {
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as Config>::RuntimeEvent>;
    type WeightInfo: WeightInfo;
}
```

The main purpose of this trait is to act as an interface between this pallet and the runtime in which it is embedded in. A type, function, or constant in this trait is essentially left to be configured by the runtime that includes this pallet.

Consequently, a runtime that wants to include this pallet must implement this trait.

### Required Associated Types
```rust
type RuntimeEvent: From<Event<Self>> + IsType<<Self as Config>::RuntimeEvent>
```

The overarching event type.

```rust
type WeightInfo: WeightInfo
```

Weight information for extrinsics in this pallet.

## Usage
The following example shows how to use the key value store pallet in your runtime:
```rust
// Import the key value store pallet
pub use pallet_kvstore;

// Add it to your runtime configuration
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        // ...
        KeyValueStore: pallet_kvstore::{Pallet, Call, Storage, Event<T>},
    }
);

// Implement the config trait for the key value store pallet
impl pallet_kvstore::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_kvstore::weights::SubstrateWeight<Runtime>;
}
```
## Genesis config
NA

## Assumptions
- The key length must not exceed 512 bytes.
- The value length must not exceed 2048 bytes.
- The user is responsible for ensuring the uniqueness of the key within their account, otherwise the previous value associated with the key will be replaced or updated.
- The stored information is publicly accessible and queryable. This means that anyone can see your data without your permission. The user must not store any sensitive information unencrypted, such as personal details, passwords, credit card numbers, or confidential information. Doing so can have serious consequences. Please use encryption tools to protect your sensitive data before storing it on this store.
