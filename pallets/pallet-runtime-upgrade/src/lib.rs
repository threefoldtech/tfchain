#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    weights::{Pays},
    decl_module, decl_event, traits::{EnsureOrigin, Vec},
    dispatch::DispatchResultWithPostInfo,
};

pub trait Config: frame_system::Config {
    type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;
    /// Origin for runtime upgrades
    type SetCodeOrigin: EnsureOrigin<Self::Origin>;
}

decl_event!(
	pub enum Event {}
);


decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        #[weight = 10]
        pub fn set_code(origin, code: Vec<u8>) -> DispatchResultWithPostInfo {
            T::SetCodeOrigin::ensure_origin(origin)?;

            frame_system::Pallet::<T>::set_code(frame_system::RawOrigin::Root.into(), code)?;

            Ok(Pays::No.into())
        }
    }
}