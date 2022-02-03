//! # Validator Pallet
//!
//! The Validator Pallet provides functionality for tfchain DAO validators.
//!

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support;


#[frame_support::pallet]
pub mod pallet {
 
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

    #[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

	}

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // The pallet's runtime storage items.
    #[pallet::storage]
    #[pallet::getter(fn validators)]
    pub type Validators<T: Config> = StorageValue<_, Vec<T::AccountId>>;


	#[pallet::event]
	pub enum Event<T: Config> {
		// New validator added.
		ValidatorAdded(T::AccountId),

		// Validator removed.
		ValidatorRemoved(T::AccountId),

	}
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub validators: Vec<T::AccountId>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				validators: Vec::new(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			Pallet::<T>::initialize_validators(&self.validators);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
    }

    impl<T: Config> Pallet<T> {
        fn initialize_validators(validators: &[T::AccountId]) {
            if !validators.is_empty() {
                assert!(
                    <Validators<T>>::get().is_none(),
                    "Validators are already initialized!"
                );
                <Validators<T>>::put(validators);
            }
        }
    }

}

