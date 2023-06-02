#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    ensure,
    traits::{Currency, OnUnbalanced},
};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::DispatchResult;
use sp_std::prelude::*;
use sp_std::vec::Vec;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

pub mod weights;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
pub struct Burn<AccountId, BalanceOf, BlockNumber> {
    pub target: AccountId,
    pub amount: BalanceOf,
    pub block: BlockNumber,
    pub message: Vec<u8>,
}

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::weights::WeightInfo;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, OnUnbalanced, ReservableCurrency},
    };
    use frame_system::{ensure_signed, pallet_prelude::*};
    use sp_std::convert::TryInto;
    use sp_std::vec;

    use crate::{Burn, Vec};

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
        BurnTransactionCreated(T::AccountId, BalanceOf<T>, T::BlockNumber, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        NotEnoughBalanceToBurn,
    }

    #[pallet::storage]
    #[pallet::getter(fn burns)]
    pub type Burns<T: Config> =
        StorageValue<_, Vec<Burn<T::AccountId, BalanceOf<T>, T::BlockNumber>>, OptionQuery>;

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

impl<T: Config> Pallet<T> {
    pub fn _burn_tft(
        target: T::AccountId,
        amount: BalanceOf<T>,
        message: Vec<u8>,
    ) -> DispatchResult {
        let free_balance: BalanceOf<T> = T::Currency::free_balance(&target);
        // Make sure the user has enough balance to cover the withdraw
        ensure!(free_balance >= amount, Error::<T>::NotEnoughBalanceToBurn);

        // Slash the balance & burn
        let imbalance = T::Currency::slash(&target, amount).0;
        T::Burn::on_unbalanced(imbalance);

        let block = <frame_system::Pallet<T>>::block_number();

        let burn = Burn {
            target: target.clone(),
            amount,
            block,
            message: message.clone(),
        };

        // Save historical burns
        match Burns::<T>::get() {
            Some(mut burns) => {
                burns.push(burn);
                Burns::<T>::put(burns);
            }
            None => {
                let burns = vec![burn];
                Burns::<T>::put(burns);
            }
        }

        // Desposit event
        Self::deposit_event(Event::BurnTransactionCreated(
            target, amount, block, message,
        ));

        Ok(())
    }
}
