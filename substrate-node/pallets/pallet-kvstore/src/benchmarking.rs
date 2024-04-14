#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as TFKVStoreModule;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::{EventRecord, Pallet as System, RawOrigin};

benchmarks! {
    // set()
    set {
        let caller: T::AccountId = whitelisted_caller();
        let key = b"name".to_vec();
        let value = b"nametest".to_vec();
    }: _(RawOrigin::Signed(caller.clone()), key.clone(), value.clone())
    verify {
        assert_eq!(TFKVStoreModule::<T>::key_value_store(caller.clone(), key.clone()), value.clone());
        assert_last_event::<T>(Event::EntrySet(caller, key, value).into());
    }

    // delete()
    delete {
        let caller: T::AccountId = whitelisted_caller();
        let key = b"Address".to_vec();
        let value = b"Cairo".to_vec();
        assert_ok!(TFKVStoreModule::<T>::set(
            RawOrigin::Signed(caller.clone()).into(),
            key.clone(),
            value.clone()
        ));
    }: _(RawOrigin::Signed(caller.clone()), key.clone())
    verify {
        assert_eq!(TFKVStoreModule::<T>::key_value_store(caller.clone(), key.clone()), b"".to_vec());
        assert_last_event::<T>(Event::EntryTaken(caller, key, value).into());
    }

    // Calling the `impl_benchmark_test_suite` macro inside the `benchmarks`
    // block will generate one #[test] function per benchmark
    impl_benchmark_test_suite!(TFKVStoreModule, crate::tests::ExternalityBuilder::build(), crate::tests::TestRuntime)
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    let events = System::<T>::events();
    let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}
