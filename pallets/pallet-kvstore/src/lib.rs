//! A pallet for Threefold key-value store
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::ensure_signed;
use sp_std::vec::Vec;

#[cfg(test)]
mod tests;
pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

decl_storage! {
    trait Store for Module<T: Config> as TFKVStore {
        pub TFKVStore get(fn key_value_store):
         double_map hasher(blake2_128_concat) T::AccountId,  hasher(blake2_128_concat) Vec<u8> => Vec<u8>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
    {
        /// A user has set their entry
        EntrySet(AccountId, Vec<u8>, Vec<u8>),

        /// A user has read their entry, leaving it in storage
        EntryGot(AccountId, Vec<u8>, Vec<u8>),

        /// A user has read their entry, removing it from storage
        EntryTaken(AccountId, Vec<u8>, Vec<u8>),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        /// The requested user has not stored a value yet
        NoValueStored,
        KeyIsTooLarge,
        ValueIsTooLarge,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {

        // Initialize errors
        type Error = Error<T>;

        // Initialize events
        fn deposit_event() = default;

        /// Set the value stored at a particular key
        #[weight = 100]
        fn set(origin, key: Vec<u8>, value: Vec<u8>) -> DispatchResult {
            // A user can only set their own entry
            ensure!(key.len()	<= 512, Error::<T>::KeyIsTooLarge);
            ensure!(value.len() <= 2048, Error::<T>::ValueIsTooLarge);
            let user = ensure_signed(origin)?;
            <TFKVStore<T>>::insert(&user, &key, &value);
            Self::deposit_event(RawEvent::EntrySet(user, key, value));
            Ok(())
        }

        /// Read the value stored at a particular key, while removing it from the map.
        /// Also emit the read value in an event
        #[weight = 10_000]
        fn delete(origin, key: Vec<u8>) -> DispatchResult {
            // A user can only take (delete) their own entry
            let user = ensure_signed(origin)?;

            ensure!(<TFKVStore<T>>::contains_key(&user, &key), Error::<T>::NoValueStored);
            let value = <TFKVStore<T>>::take(&user, &key);
            Self::deposit_event(RawEvent::EntryTaken(user, key, value));
            Ok(())
        }

    }
}
