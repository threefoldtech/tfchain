#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_runtime::traits::StaticLookup;
use crate::Pallet as ValidatorModule;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::{EventRecord, Pallet as System, RawOrigin};
use pallet_membership::Pallet as CouncilMembership;

benchmarks! {
    // create_validator_request()
    create_validator_request {
        let caller: T::AccountId = whitelisted_caller();
        let validator_node_account: T::AccountId = account("Alice", 0, 0);
        let stash_account: T::AccountId = account("Bob", 0, 1);
        let description = b"description".to_vec();
        let tf_connect_id = b"tf_connect_id".to_vec();
        let info = b"validator_candidate_info".to_vec();
    }: _(
        RawOrigin::Signed(caller.clone()),
        validator_node_account.clone(),
        stash_account.clone(),
        description.clone(),
        tf_connect_id.clone(),
        info.clone()
    )
    verify {
        let validator_created = _get_validator::<T>(types::ValidatorRequestState::Created);
        assert_eq!(
            ValidatorModule::<T>::validator_requests(caller.clone()),
            Some(validator_created.clone()),
        );
        assert_last_event::<T>(Event::ValidatorRequestCreated(caller, validator_created).into());
    }

    // activate_validator_node()
    activate_validator_node {
        _create_validator_and_approve::<T>();
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Signed(caller.clone()))
    verify {
        let validator_validating = _get_validator::<T>(types::ValidatorRequestState::Validating);
        assert_eq!(
            ValidatorModule::<T>::validator_requests(caller),
            Some(validator_validating.clone()),
        );
        assert_last_event::<T>(Event::ValidatorActivated(validator_validating).into());
    }

    // change_validator_node_account()
    change_validator_node_account {
        _create_validator_and_start_validating::<T>();
        let caller: T::AccountId = whitelisted_caller();
        let validator_node_account: T::AccountId = account("Ferdie", 0, 2);
    }: _(RawOrigin::Signed(caller.clone()), validator_node_account.clone())
    verify {
        let mut validator_validating = _get_validator::<T>(crate::types::ValidatorRequestState::Validating);
        validator_validating.validator_node_account = validator_node_account.clone();
        assert_eq!(
            ValidatorModule::<T>::validator_requests(caller),
            Some(validator_validating),
        );
        assert_last_event::<T>(Event::NodeValidatorChanged(validator_node_account).into());
    }

    // bond()
    bond {
        let caller: T::AccountId = whitelisted_caller();
        let caller_lookup = T::Lookup::unlookup(caller.clone());
        let stash: T::AccountId = account("Bob", 0, 1);
    }: _(RawOrigin::Signed(stash.clone()), caller_lookup)
    verify {
        assert_eq!(
            ValidatorModule::<T>::bonded(stash.clone()),
            Some(caller),
        );
        assert_last_event::<T>(Event::Bonded(stash).into());
    }

    // approve_validator()
    approve_validator {
        _create_validator::<T>();
        let caller: T::AccountId = whitelisted_caller();
        let caller_lookup = T::Lookup::unlookup(caller.clone());

        let council_members = CouncilMembership::<T, _>::members();
        assert!(council_members.len() > 0);
        let council_member = council_members.into_iter().next().unwrap();
    }: _(RawOrigin::Signed(council_member.clone()), caller_lookup)
    verify {
        let validator_approved = _get_validator::<T>(types::ValidatorRequestState::Approved);
        assert_eq!(
            ValidatorModule::<T>::validator_requests(caller.clone()),
            Some(validator_approved.clone()),
        );
        assert!(CouncilMembership::<T, _>::members().contains(&caller));
        assert_last_event::<T>(Event::ValidatorRequestApproved(validator_approved).into());
    }

    // remove_validator()
    remove_validator {
        _create_validator_and_approve::<T>();
        let caller: T::AccountId = whitelisted_caller();
        let caller_lookup = T::Lookup::unlookup(caller.clone());

        let council_members = CouncilMembership::<T, _>::members();
        assert!(council_members.len() > 0);
        let council_member = council_members.into_iter().next().unwrap();
    }: _(RawOrigin::Signed(council_member.clone()), caller_lookup.clone())
    verify {
        assert!(ValidatorModule::<T>::validator_requests(caller.clone()).is_none());
        assert!(!CouncilMembership::<T, _>::members().contains(&caller));
    }

    // Calling the `impl_benchmark_test_suite` macro inside the `benchmarks`
    // block will generate one #[test] function per benchmark
    impl_benchmark_test_suite!(ValidatorModule, crate::mock::new_test_ext(), crate::mock::TestRuntime)
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    let events = System::<T>::events();
    let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}

fn _get_validator<T: Config>(
    state: crate::types::ValidatorRequestState,
) -> crate::types::Validator<T::AccountId> {
    crate::types::Validator {
        validator_node_account: account("Alice", 0, 0),
        stash_account: account("Bob", 0, 1),
        description: b"description".to_vec(),
        tf_connect_id: b"tf_connect_id".to_vec(),
        info: b"validator_candidate_info".to_vec(),
        state,
    }
}

fn _create_validator<T: Config>() {
    assert_ok!(ValidatorModule::<T>::create_validator_request(
        RawOrigin::Signed(whitelisted_caller()).into(),
        account("Alice", 0, 0),
        account("Bob", 0, 1),
        b"description".to_vec(),
        b"tf_connect_id".to_vec(),
        b"validator_candidate_info".to_vec(),
    ));
}

fn _create_validator_and_approve<T: Config>() {
    _create_validator::<T>();

    let caller: T::AccountId = whitelisted_caller();
    let caller_lookup = T::Lookup::unlookup(caller.clone());

    let council_members = CouncilMembership::<T, _>::members();
    assert!(council_members.len() > 0);
    let council_member = council_members.into_iter().next().unwrap();

    assert_ok!(ValidatorModule::<T>::approve_validator(
        RawOrigin::Signed(council_member).into(),
        caller_lookup,
    ));
}

fn _create_validator_and_start_validating<T: Config>() {
    _create_validator_and_approve::<T>();

    let caller: T::AccountId = whitelisted_caller();

    assert_ok!(ValidatorModule::<T>::activate_validator_node(
        RawOrigin::Signed(caller).into(),
    ));
}
