#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as TFTPriceModule;
use frame_benchmarking::benchmarks;
use frame_system::{pallet_prelude::BlockNumberFor, EventRecord, Pallet as System, RawOrigin};
use pallet_session::Pallet as Session;

benchmarks! {
    where_clause {
        where
        T: pallet_session::Config<ValidatorId = <T as frame_system::Config>::AccountId>,
    }

    // set_prices()
    set_prices {
        let block = BlockNumberFor::<T>::from(1u32);
        let price = 500;

        let validators = Session::<T>::validators();
        assert!(validators.len() > 0);
        let validator = validators.into_iter().next().unwrap();

    }: _(RawOrigin::Signed(validator), price, block)
    verify {
        assert_eq!(TFTPriceModule::<T>::tft_price(), price);
        assert_eq!(TFTPriceModule::<T>::average_tft_price(), price);
        assert_last_event::<T>(Event::AveragePriceStored(price).into());
    }

    // set_min_tft_price()
    set_min_tft_price {
        let price = 20;
    }: _(RawOrigin::Root, price)
    verify {
        assert_eq!(TFTPriceModule::<T>::min_tft_price(), price);
    }

    // set_max_tft_price()
    set_max_tft_price {
        let price = 2000;
    }: _(RawOrigin::Root, price)
    verify {
        assert_eq!(TFTPriceModule::<T>::max_tft_price(), price);
    }

    // Calling the `impl_benchmark_test_suite` macro inside the `benchmarks`
    // block will generate one #[test] function per benchmark
    impl_benchmark_test_suite!(TFTPriceModule, crate::mock::ExternalityBuilder::build(), crate::mock::TestRuntime)
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    let events = System::<T>::events();
    let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}
