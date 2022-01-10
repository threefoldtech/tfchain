#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    traits::{Currency, OnUnbalanced, ReservableCurrency, Vec},
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::DispatchResult;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

// balance type using reservable currency type
type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::NegativeImbalance;

pub trait Config: system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

    /// Currency type for this pallet.
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

    /// Handler for the unbalanced decrement when slashing (burning collateral)
    type Burn: OnUnbalanced<NegativeImbalanceOf<Self>>;
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Config>::AccountId,
        BlockNumber = <T as system::Config>::BlockNumber,
        BalanceOf = BalanceOf<T>,
    {
        BurnTransactionCreated(AccountId, BalanceOf, BlockNumber, Vec<u8>),
    }
);

decl_error! {
    /// Error for the vesting module.
    pub enum Error for Module<T: Config> {
        NotEnoughBalanceToBurn,
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct Burn<AccountId, BalanceOf, BlockNumber> {
    pub target: AccountId,
    pub amount: BalanceOf,
    pub block: BlockNumber,
    pub message: Vec<u8>,
}

decl_storage! {
    trait Store for Module<T: Config> as TFTBridgeModule {
        pub Burns get(fn burns): Vec<Burn<T::AccountId, BalanceOf<T>, T::BlockNumber>>;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        #[weight = 10_000]
        fn burn_tft(origin, amount: BalanceOf<T>, message: Vec<u8>){
            let target = ensure_signed(origin)?;
            Self::_burn_tft(target, amount, message)?;
        }
    }
}

impl<T: Config> Module<T> {
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

        let block = <frame_system::Module<T>>::block_number();

        // Save historical burns
        let mut burns = Burns::<T>::get();
        burns.push(Burn {
            target: target.clone(),
            amount,
            block,
            message: message.clone(),
        });
        Burns::<T>::put(burns);

        // Desposit event
        Self::deposit_event(RawEvent::BurnTransactionCreated(
            target, amount, block, message,
        ));

        Ok(())
    }
}
