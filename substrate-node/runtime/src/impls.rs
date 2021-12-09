use crate::Runtime;
use crate::{AccountId, Balances, get_staking_pool_account};
use frame_support::traits::{Currency, Imbalance, OnUnbalanced};

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
        let staking_pot = get_staking_pool_account();
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
    fn on_nonzero_unbalanced(amount: NegativeImbalance) {
        <ToStakingPot<R> as OnUnbalanced<_>>::on_nonzero_unbalanced(amount);
    }
}


/// Recover slashed funds
pub struct DealWithSlash<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance> for DealWithSlash<R>
where
    R: pallet_balances::Config,
    <R as frame_system::Config>::AccountId: From<AccountId>,
    <R as frame_system::Config>::AccountId: Into<AccountId>,
    <R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
{
    fn on_nonzero_unbalanced(amount: NegativeImbalance) {
        let numeric_amount = amount.peek();
        let slashing_beneficiary = <pallet_staking::Module<Runtime>>::slashing_beneficiary();
        <pallet_balances::Module<Runtime>>::resolve_creating(
            &slashing_beneficiary,
            amount,
        );
        <frame_system::Module<Runtime>>::deposit_event(pallet_balances::Event::Deposit(
            slashing_beneficiary,
            numeric_amount,
        ));
    }
}
