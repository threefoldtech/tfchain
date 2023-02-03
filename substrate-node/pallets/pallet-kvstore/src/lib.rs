//! A pallet for Threefold key-value store
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{ensure, pallet_prelude::*, traits::IsType};
    use frame_system::{ensure_signed, pallet_prelude::*};
    use sp_std::convert::TryInto;
    use sp_std::prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A user has set their entry
        EntrySet(T::AccountId, Vec<u8>, Vec<u8>),
        /// A user has read their entry, leaving it in storage
        EntryGot(T::AccountId, Vec<u8>, Vec<u8>),
        /// A user has read their entry, removing it from storage
        EntryTaken(T::AccountId, Vec<u8>, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The requested user has not stored a value yet
        NoValueStored,
        KeyIsTooLarge,
        ValueIsTooLarge,
    }

    #[pallet::storage]
    #[pallet::getter(fn key_value_store)]
    pub type TFKVStore<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        Vec<u8>,
        Vec<u8>,
        ValueQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set the value stored at a particular key
        #[pallet::call_index(0)]
        #[pallet::weight(100_000_000)]
        pub fn set(
            origin: OriginFor<T>,
            key: Vec<u8>,
            value: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            // A user can only set their own entry
            ensure!(key.len() <= 512, Error::<T>::KeyIsTooLarge);
            ensure!(value.len() <= 2048, Error::<T>::ValueIsTooLarge);
            let user = ensure_signed(origin)?;
            <TFKVStore<T>>::insert(&user, &key, &value);
            Self::deposit_event(Event::EntrySet(user, key, value));
            Ok(().into())
        }

        /// Read the value stored at a particular key, while removing it from the map.
        /// Also emit the read value in an event
        #[pallet::call_index(1)]
        #[pallet::weight(100_000_000)]
        pub fn delete(origin: OriginFor<T>, key: Vec<u8>) -> DispatchResultWithPostInfo {
            // A user can only take (delete) their own entry
            let user = ensure_signed(origin)?;

            ensure!(
                <TFKVStore<T>>::contains_key(&user, &key),
                Error::<T>::NoValueStored
            );
            let value = <TFKVStore<T>>::take(&user, &key);
            Self::deposit_event(Event::EntryTaken(user, key, value));
            Ok(().into())
        }
    }
}