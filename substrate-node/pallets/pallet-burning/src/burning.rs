use crate::*;
use frame_support::{
    ensure,
    traits::{Currency, OnUnbalanced},
};
use sp_runtime::DispatchResult;
use sp_std::{vec, vec::Vec};

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

        let burn = types::Burn {
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
