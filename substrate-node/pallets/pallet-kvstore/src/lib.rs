//! A pallet for Threefold key-value store
#![cfg_attr(not(feature = "std"), no_std)]

pub mod kvstore;

pub use pallet::*;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use super::weights::WeightInfo;
    use frame_support::{pallet_prelude::*, traits::IsType};
    use frame_system::{ensure_signed, pallet_prelude::*};
    use sp_std::convert::TryInto;
    use sp_std::prelude::*;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: crate::weights::WeightInfo;
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
        #[pallet::weight(<T as Config>::WeightInfo::set())]
        pub fn set(
            origin: OriginFor<T>,
            key: Vec<u8>,
            value: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            // A user can only set their own entry
            let user = ensure_signed(origin)?;
            Self::_set(user, key, value)
        }

        /// Read the value stored at a particular key, while removing it from the map.
        /// Also emit the read value in an event
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::delete())]
        pub fn delete(origin: OriginFor<T>, key: Vec<u8>) -> DispatchResultWithPostInfo {
            // A user can only take (delete) their own entry
            let user = ensure_signed(origin)?;
            Self::_delete(user, key)
        }
    }
}
