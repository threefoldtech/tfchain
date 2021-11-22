use crate::{AccountId, Balances};
use frame_support::traits::{Currency, Imbalance, OnUnbalanced};
use crate::Runtime;

type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;

/// Logic for the stakingpool to receive the fees.
pub struct ToStakingPot<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance> for ToStakingPot<R>
where
    R: pallet_balances::Config,
    <R as frame_system::Config>::AccountId: From<AccountId>,
    <R as frame_system::Config>::AccountId: Into<AccountId>,
    <R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
{
    fn on_nonzero_unbalanced(amount: NegativeImbalance) {
        let numeric_amount = amount.peek();
        let staking_pot = <pallet_staking::Module<Runtime>>::staking_pool_account();
        <pallet_balances::Module<Runtime>>::resolve_creating(&staking_pot, amount);
        <frame_system::Module<Runtime>>::deposit_event(pallet_balances::Event::Deposit(
            staking_pot,
            numeric_amount,
        ));
    }
}

pub struct DealWithFees<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance> for DealWithFees<R>
where
    R: pallet_balances::Config,
    <R as frame_system::Config>::AccountId: From<AccountId>,
    <R as frame_system::Config>::AccountId: Into<AccountId>,
    <R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
{
    fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
        if let Some(mut fees) = fees_then_tips.next() {
            if let Some(tips) = fees_then_tips.next() {
                tips.merge_into(&mut fees);
            }
            <ToStakingPot<R> as OnUnbalanced<_>>::on_unbalanced(fees);
        }
    }
}

