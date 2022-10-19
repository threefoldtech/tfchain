use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn test_create_validator_request_works() {
    new_test_ext().execute_with(|| create_validator());
}

#[test]
fn test_activate_validator_node_works() {
    new_test_ext().execute_with(|| {
        create_validator_and_approve();

        assert_ok!(ValidatorModule::activate_validator_node(Origin::signed(10),));
    });
}

#[test]
fn test_change_validator_node_account_works() {
    new_test_ext().execute_with(|| {
        create_validator_and_approve();

        assert_ok!(ValidatorModule::change_validator_node_account(
            Origin::signed(10),
            13,
        ));
    });
}

#[test]
fn test_bond_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(ValidatorModule::bond(Origin::signed(10), 12,));
    });
}

#[test]
fn test_approve_validator_works() {
    new_test_ext().execute_with(|| create_validator_and_approve());
}

#[test]
fn test_remove_validator_council_works() {
    new_test_ext().execute_with(|| {
        create_validator_and_approve();

        assert_ok!(ValidatorModule::remove_validator(Origin::signed(1), 10,));
    });
}

#[test]
fn test_remove_validator_validator_works() {
    new_test_ext().execute_with(|| {
        create_validator_and_approve();

        assert_ok!(ValidatorModule::remove_validator(Origin::signed(10), 10,));
    });
}

#[test]
fn test_remove_validator_unauthorized_fails() {
    new_test_ext().execute_with(|| {
        create_validator_and_approve();

        assert_noop!(
            ValidatorModule::remove_validator(Origin::signed(4), 10,),
            Error::<TestRuntime>::BadOrigin
        );
    });
}

fn create_validator() {
    assert_ok!(ValidatorModule::create_validator_request(
        Origin::signed(10),
        11,
        12,
        b"description".to_vec(),
        b"tf_connect_id".to_vec(),
        b"validator_candidate_info".to_vec(),
    ));
}

fn create_validator_and_approve() {
    create_validator();

    assert_ok!(ValidatorModule::approve_validator(Origin::signed(1), 10,));
}
