use crate::mock::*;
use frame_support::assert_ok;

#[test]
fn test_create_validator_request_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(ValidatorModule::create_validator_request(
            Origin::signed(alice()),
            alice(),
            alice(),
            b"description".to_vec(),
            b"tf_connect_id".to_vec(),
            b"candidate_info".to_vec(),
        ));
    });
}

#[test]
fn test_activate_validator_node_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(true, false);
    });
}

#[test]
fn test_change_validator_node_account_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(true, false);
    });
}

#[test]
fn test_bond_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(true, false);
    });
}

#[test]
fn test_approve_validator_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(true, false);
    });
}

#[test]
fn test_remove_validator_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(true, false);
    });
}
