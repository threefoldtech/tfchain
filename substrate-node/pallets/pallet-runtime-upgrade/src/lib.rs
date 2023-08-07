#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Origin for runtime upgrades
        type SetCodeOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        // Give same weight as set_code() wrapped extrinsic from frame_system
        #[pallet::weight((T::BlockWeights::get().base_block, DispatchClass::Operational))]
        pub fn set_code(origin: OriginFor<T>, code: Vec<u8>) -> DispatchResultWithPostInfo {
            T::SetCodeOrigin::ensure_origin(origin)?;
            frame_system::Pallet::<T>::set_code(frame_system::RawOrigin::Root.into(), code)?;
            Ok(Pays::No.into())
        }
    }
}
