#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Origin for runtime upgrades
        type SetCodeOrigin: EnsureOrigin<Self::Origin>;
    }

    #[pallet::event]
    pub enum Event<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(100_000_000)]
        pub fn set_code(origin: OriginFor<T>, code: Vec<u8>) -> DispatchResultWithPostInfo {
            T::SetCodeOrigin::ensure_origin(origin)?;
            frame_system::Pallet::<T>::set_code(frame_system::RawOrigin::Root.into(), code)?;
            Ok(Pays::No.into())
        }
    }
}
