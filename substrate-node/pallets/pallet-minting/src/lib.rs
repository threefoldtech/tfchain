#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::DispatchResult;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::convert::TryInto;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        MintTransactionCreated(),
    }

    #[pallet::error]
    pub enum Error<T> {
        NotEnoughBalanceToMint,
    }

    #[pallet::storage]
    #[pallet::getter(fn mints)]
    pub type Mints<T> = StorageValue<_, u64, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn minting_frequency)]
    pub type MintingFrequency<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub minting_frequency: u64,
    }

    // The default value for the genesis config type.
    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self {
                minting_frequency: 600,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            MintingFrequency::<T>::put(self.minting_frequency);
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(100_000_000)]
        pub fn mint(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let _ = ensure_signed(origin)?;
            Self::mint_tft()?;
            Ok(Pays::No.into())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn mint_tft() -> DispatchResult {
        Self::deposit_event(Event::MintTransactionCreated());
        Ok(())
    }
}
