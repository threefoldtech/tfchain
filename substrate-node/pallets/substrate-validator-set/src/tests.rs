//! Tests for the Validator Set pallet.

#![cfg(test)]

use crate::mock::*;
use crate::mock::{authorities, new_test_ext, Session, TestRuntime, ValidatorSet};
use crate::Error;
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use sp_runtime::testing::UintAuthorityId;
use sp_runtime::DispatchError;

#[test]
fn simple_setup_should_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(
            authorities(),
            vec![UintAuthorityId(1), UintAuthorityId(2), UintAuthorityId(3)]
        );
        assert_eq!(ValidatorSet::validators(), vec![1u64, 2u64, 3u64]);
        assert_eq!(Session::validators(), vec![1, 2, 3]);
    });
}

#[test]
fn add_validator_updates_validators_list() {
    new_test_ext().execute_with(|| {
        assert_ok!(ValidatorSet::add_validator(RawOrigin::Root.into(), 4));
        assert_eq!(ValidatorSet::validators(), vec![1u64, 2u64, 3u64, 4u64])
    });
}

#[test]
fn remove_validator_updates_validators_list() {
    new_test_ext().execute_with(|| {
        assert_ok!(ValidatorSet::remove_validator(RawOrigin::Root.into(), 2));
        assert_eq!(ValidatorSet::validators(), vec![1u64, 3u64]);
    });
}

#[test]
fn add_validator_fails_with_invalid_origin() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ValidatorSet::add_validator(RuntimeOrigin::signed(1), 4),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn remove_validator_fails_with_invalid_origin() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ValidatorSet::remove_validator(RuntimeOrigin::signed(1), 4),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn duplicate_check() {
    new_test_ext().execute_with(|| {
        assert_ok!(ValidatorSet::add_validator(RawOrigin::Root.into(), 4));
        assert_eq!(ValidatorSet::validators(), vec![1u64, 2u64, 3u64, 4u64]);
        assert_noop!(
            ValidatorSet::add_validator(RawOrigin::Root.into(), 4),
            Error::<TestRuntime>::Duplicate
        );
    });
}
