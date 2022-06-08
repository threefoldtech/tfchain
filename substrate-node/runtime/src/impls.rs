use crate::*;
use frame_support::traits::{Currency, OnUnbalanced};

type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;

pub struct DealWithFees<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance> for DealWithFees<R>
where
    R: pallet_balances::Config,
    <R as frame_system::Config>::AccountId: From<AccountId>,
    <R as frame_system::Config>::AccountId: Into<AccountId>,
    <R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
{
    fn on_nonzero_unbalanced(amount: NegativeImbalance) {
        Balances::resolve_creating(&Authorship::author(), amount);
    }
}
