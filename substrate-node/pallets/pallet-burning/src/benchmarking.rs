#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as BurningModule;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::{EventRecord, Pallet as System, RawOrigin};
use sp_runtime::{traits::StaticLookup, SaturatedConversion};

benchmarks! {
    // burn_tft()
    burn_tft {
        let target: T::AccountId = whitelisted_caller();
        let target_lookup = T::Lookup::unlookup(target.clone());
        let amount = BalanceOf::<T>::saturated_from(1000 as u128);
        T::Currency::make_free_balance_be(&target, amount);
        let message = b"some_message".to_vec();
    }: _(RawOrigin::Signed(target.clone()), amount.clone(), message.clone())
    verify {
        let block = T::BlockNumber::from(1 as u32);
        assert_eq!(T::Currency::free_balance(&target).saturated_into::<u128>(), 0 as u128);
        assert_last_event::<T>(Event::BurnTransactionCreated(target, amount, block, message).into());
    }

    // Calling the `impl_benchmark_test_suite` macro inside the `benchmarks`
    // block will generate one #[test] function per benchmark
    impl_benchmark_test_suite! (BurningModule, crate::mock::new_test_ext(), crate::mock::TestRuntime)
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    let events = System::<T>::events();
    let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}
