use super::Event as ValidatorEvent;
use crate::{mock::RuntimeEvent as MockEvent, mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use frame_system::{EventRecord, Phase};
use sp_core::H256;

#[test]
fn test_create_validator_request_works() {
    new_test_ext().execute_with(|| {
        create_validator();

        let validator_created = get_validator(crate::types::ValidatorRequestState::Created);

        assert_eq!(
            ValidatorModule::validator_requests(10),
            Some(validator_created.clone()),
        );

        let our_events = System::events();
        assert!(!our_events.is_empty());
        assert_eq!(
            our_events.last().unwrap(),
            &record(MockEvent::ValidatorModule(
                ValidatorEvent::<TestRuntime>::ValidatorRequestCreated(10, validator_created)
            ))
        );
    });
}

#[test]
fn test_create_validator_request_duplicate_fails() {
    new_test_ext().execute_with(|| {
        create_validator();

        assert_noop!(
            ValidatorModule::create_validator_request(
                RuntimeOrigin::signed(10), // same origin
                13,
                14,
                b"other_description".to_vec(),
                b"other_tf_connect_id".to_vec(),
                b"other_validator_candidate_info".to_vec(),
            ),
            Error::<TestRuntime>::DuplicateValidator,
        );
    });
}

#[test]
fn test_approve_validator_works() {
    new_test_ext().execute_with(|| {
        create_validator_and_approve();

        let validator_approved = get_validator(crate::types::ValidatorRequestState::Approved);

        assert_eq!(
            ValidatorModule::validator_requests(10),
            Some(validator_approved.clone()),
        );

        let our_events = System::events();
        assert!(!our_events.is_empty());
        assert_eq!(
            our_events.last().unwrap(),
            &record(MockEvent::ValidatorModule(
                ValidatorEvent::<TestRuntime>::ValidatorRequestApproved(validator_approved)
            ))
        );
    });
}

#[test]
fn test_approve_validator_by_unauthorized_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ValidatorModule::approve_validator(RuntimeOrigin::signed(10), 10,),
            Error::<TestRuntime>::NotCouncilMember,
        );
    });
}

#[test]
fn test_approve_validator_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ValidatorModule::approve_validator(RuntimeOrigin::signed(1), 10,),
            Error::<TestRuntime>::ValidatorNotFound,
        );
    });
}

#[test]
fn test_activate_validator_node_works() {
    new_test_ext().execute_with(|| {
        create_validator_and_approve();

        assert_ok!(ValidatorModule::activate_validator_node(
            RuntimeOrigin::signed(10),
        ));

        let validator_validating = get_validator(crate::types::ValidatorRequestState::Validating);

        assert_eq!(
            ValidatorModule::validator_requests(10),
            Some(validator_validating.clone()),
        );

        let our_events = System::events();
        assert!(!our_events.is_empty());
        assert_eq!(
            our_events.last().unwrap(),
            &record(MockEvent::ValidatorModule(
                ValidatorEvent::<TestRuntime>::ValidatorActivated(validator_validating)
            ))
        );
    });
}

#[test]
fn test_activate_validator_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ValidatorModule::activate_validator_node(RuntimeOrigin::signed(10),),
            Error::<TestRuntime>::ValidatorNotFound,
        );
    });
}

#[test]
fn test_activate_validator_not_approved_fails() {
    new_test_ext().execute_with(|| {
        create_validator();

        assert_noop!(
            ValidatorModule::activate_validator_node(RuntimeOrigin::signed(10),),
            Error::<TestRuntime>::ValidatorNotApproved,
        );
    });
}

#[test]
fn test_activate_validator_already_validating_fails() {
    new_test_ext().execute_with(|| {
        create_validator_and_approve();

        assert_ok!(ValidatorModule::activate_validator_node(
            RuntimeOrigin::signed(10),
        ));

        assert_noop!(
            ValidatorModule::activate_validator_node(RuntimeOrigin::signed(10),),
            Error::<TestRuntime>::ValidatorValidatingAlready,
        );
    });
}

#[test]
fn test_change_validator_node_account_works() {
    new_test_ext().execute_with(|| {
        create_validator_and_approve();

        assert_ok!(ValidatorModule::change_validator_node_account(
            RuntimeOrigin::signed(10),
            13,
        ));

        let mut validator_approved = get_validator(crate::types::ValidatorRequestState::Approved);
        validator_approved.validator_node_account = 13;

        assert_eq!(
            ValidatorModule::validator_requests(10),
            Some(validator_approved),
        );
    });
}

#[test]
fn test_change_validator_node_account_validating_works() {
    new_test_ext().execute_with(|| {
        create_validator_and_approve();

        assert_ok!(ValidatorModule::activate_validator_node(
            RuntimeOrigin::signed(10),
        ));

        assert_ok!(ValidatorModule::change_validator_node_account(
            RuntimeOrigin::signed(10),
            13,
        ));

        let mut validator_validating =
            get_validator(crate::types::ValidatorRequestState::Validating);
        validator_validating.validator_node_account = 13;

        assert_eq!(
            ValidatorModule::validator_requests(10),
            Some(validator_validating),
        );

        let our_events = System::events();
        assert!(!our_events.is_empty());
        assert_eq!(
            our_events[our_events.len() - 3],
            record(MockEvent::ValidatorModule(
                ValidatorEvent::<TestRuntime>::NodeValidatorRemoved(11)
            ))
        );
        assert_eq!(
            our_events[our_events.len() - 1],
            record(MockEvent::ValidatorModule(
                ValidatorEvent::<TestRuntime>::NodeValidatorChanged(13)
            ))
        );
    });
}

#[test]
fn test_change_validator_node_account_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ValidatorModule::change_validator_node_account(RuntimeOrigin::signed(10), 13,),
            Error::<TestRuntime>::ValidatorNotFound,
        );
    });
}

#[test]
fn test_bond_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(ValidatorModule::bond(RuntimeOrigin::signed(12), 10,));

        assert_eq!(ValidatorModule::bonded(12), Some(10),);

        let our_events = System::events();
        assert!(!our_events.is_empty());
        assert_eq!(
            our_events.last().unwrap(),
            &record(MockEvent::ValidatorModule(
                ValidatorEvent::<TestRuntime>::Bonded(12)
            ))
        );
    });
}

#[test]
fn test_bond_already_bounded_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(ValidatorModule::bond(RuntimeOrigin::signed(12), 10,));

        assert_noop!(
            ValidatorModule::bond(RuntimeOrigin::signed(12), 10,),
            Error::<TestRuntime>::AlreadyBonded,
        );
    });
}

#[test]
fn test_remove_validator_works() {
    new_test_ext().execute_with(|| {
        create_validator_and_approve();

        assert_ok!(ValidatorModule::remove_validator(
            RuntimeOrigin::signed(1),
            10,
        ));

        assert_eq!(ValidatorModule::validator_requests(10), None,);
    });
}

#[test]
fn test_remove_validator_by_himself_works() {
    new_test_ext().execute_with(|| {
        create_validator_and_approve();

        assert_ok!(ValidatorModule::remove_validator(
            RuntimeOrigin::signed(10),
            10,
        ));

        assert_eq!(ValidatorModule::validator_requests(10), None,);
    });
}

#[test]
fn test_remove_validator_by_unauthorized_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ValidatorModule::remove_validator(RuntimeOrigin::signed(4), 10,),
            Error::<TestRuntime>::BadOrigin,
        );
    });
}

#[test]
fn test_remove_validator_not_found_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ValidatorModule::remove_validator(RuntimeOrigin::signed(1), 10,),
            Error::<TestRuntime>::ValidatorNotFound,
        );
    });
}

fn record(event: RuntimeEvent) -> EventRecord<RuntimeEvent, H256> {
    EventRecord {
        phase: Phase::Initialization,
        event,
        topics: vec![],
    }
}

fn get_validator(state: crate::types::ValidatorRequestState) -> crate::types::Validator<u64> {
    crate::types::Validator {
        validator_node_account: 11,
        stash_account: 12,
        description: b"description".to_vec(),
        tf_connect_id: b"tf_connect_id".to_vec(),
        info: b"validator_candidate_info".to_vec(),
        state,
    }
}

fn create_validator() {
    assert_ok!(ValidatorModule::create_validator_request(
        RuntimeOrigin::signed(10),
        11,
        12,
        b"description".to_vec(),
        b"tf_connect_id".to_vec(),
        b"validator_candidate_info".to_vec(),
    ));
}

fn create_validator_and_approve() {
    create_validator();

    assert_ok!(ValidatorModule::approve_validator(
        RuntimeOrigin::signed(1),
        10,
    ));
}
