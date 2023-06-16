#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as ValidatorSet;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::{EventRecord, Pallet as System, RawOrigin};

benchmarks! {
    // add_validator()
    add_validator {
        let mut validators = ValidatorSet::<T>::validators();
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Root, caller.clone())
    verify {
        validators.push(caller.clone());
        assert_eq!(
            ValidatorSet::<T>::validators(),
            validators
        );
        assert_last_event::<T>(Event::ValidatorAdditionInitiated(caller).into());
    }

    // remove_validator()
    remove_validator {
        let validators = ValidatorSet::<T>::validators();
        let caller: T::AccountId = whitelisted_caller();
        assert_ok!(ValidatorSet::<T>::add_validator(
            RawOrigin::Root.into(),
            caller.clone(),
        ));
    }: _(RawOrigin::Root, caller.clone())
    verify {
        assert_eq!(
            ValidatorSet::<T>::validators(),
            validators
        );
        assert_last_event::<T>(Event::ValidatorRemovalInitiated(caller).into());
    }

    // add_validator_again()
    add_validator_again {
        let mut validators = ValidatorSet::<T>::validators();
        let caller: T::AccountId = whitelisted_caller();
        assert_ok!(ValidatorSet::<T>::add_validator(
            RawOrigin::Root.into(),
            caller.clone(),
        ));
        assert_ok!(ValidatorSet::<T>::remove_validator(
            RawOrigin::Root.into(),
            caller.clone(),
        ));
    }: _(RawOrigin::Signed(caller.clone()), caller.clone())
    verify {
        validators.push(caller.clone());
        assert_eq!(
            ValidatorSet::<T>::validators(),
            validators
        );
        assert_last_event::<T>(Event::ValidatorAdditionInitiated(caller).into());
    }

    // Calling the `impl_benchmark_test_suite` macro inside the `benchmarks`
    // block will generate one #[test] function per benchmark
    impl_benchmark_test_suite!(ValidatorSet, crate::mock::new_test_ext(), crate::mock::TestRuntime)
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    let events = System::<T>::events();
    let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}
