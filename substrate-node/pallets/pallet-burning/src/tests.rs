use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::SaturatedConversion;

#[test]
fn test_burn() {
    new_test_ext().execute_with(|| {
        assert_ok!(BurningModule::burn_tft(
            Origin::signed(alice()),
            900000000000,
            "some_message".as_bytes().to_vec()
        ));

        let b = Balances::free_balance(alice());
        let balances_as_u128: u128 = b.saturated_into::<u128>();
        assert_eq!(balances_as_u128, 100000000000);
    });
}

#[test]
fn test_burn_to_much_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            BurningModule::burn_tft(
                Origin::signed(alice()),
                1200000000000,
                "some_message".as_bytes().to_vec()
            ),
            Error::<TestRuntime>::NotEnoughBalanceToBurn
        );
    });
}
