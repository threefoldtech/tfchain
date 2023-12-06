#![cfg_attr(not(feature = "std"), no_std)]

pub mod burning;
pub mod types;
pub mod weights;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::weights::WeightInfo;
    use crate::types::Burn;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, OnUnbalanced, ReservableCurrency},
    };
    use frame_system::{ensure_signed, pallet_prelude::*};
    use sp_std::vec::Vec;

    // balance type using reservable currency type
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::NegativeImbalance;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Currency type for this pallet.
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
        /// Handler for the unbalanced decrement when slashing (burning collateral)
        type Burn: OnUnbalanced<NegativeImbalanceOf<Self>>;
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        BurnTransactionCreated(T::AccountId, BalanceOf<T>, BlockNumberFor<T>, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        NotEnoughBalanceToBurn,
    }

    #[pallet::storage]
    #[pallet::getter(fn burns)]
    pub type Burns<T: Config> =
        StorageValue<_, Vec<Burn<T::AccountId, BalanceOf<T>, BlockNumberFor<T>>>, OptionQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::burn_tft())]
        pub fn burn_tft(
            origin: OriginFor<T>,
            amount: BalanceOf<T>,
            message: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let target = ensure_signed(origin)?;
            Self::_burn_tft(target, amount, message)?;
            Ok(().into())
        }
    }
}
