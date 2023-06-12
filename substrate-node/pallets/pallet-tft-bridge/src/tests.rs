use crate::{mock::*, Error};
use frame_support::{
    assert_noop, assert_ok,
    traits::{LockableCurrency, OnFinalize, OnInitialize, WithdrawReasons},
};
use frame_system::RawOrigin;
use sp_runtime::traits::SaturatedConversion;
use sp_runtime::DispatchError;

#[test]
fn add_validator_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(TFTBridgeModule::add_bridge_validator(
            RawOrigin::Root.into(),
            bob()
        ));
        assert_ok!(TFTBridgeModule::add_bridge_validator(
            RawOrigin::Root.into(),
            alice()
        ));
    });
}

#[test]
fn add_validator_non_root_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            TFTBridgeModule::add_bridge_validator(RuntimeOrigin::signed(alice()), bob()),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn removing_validator_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(TFTBridgeModule::add_bridge_validator(
            RawOrigin::Root.into(),
            bob()
        ));
        assert_ok!(TFTBridgeModule::remove_bridge_validator(
            RawOrigin::Root.into(),
            bob()
        ));
    });
}

#[test]
fn proposing_mint_transaction_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(TFTBridgeModule::add_bridge_validator(
            RawOrigin::Root.into(),
            alice()
        ));

        assert_ok!(TFTBridgeModule::propose_or_vote_mint_transaction(
            RuntimeOrigin::signed(alice()),
            "some_tx".as_bytes().to_vec(),
            bob(),
            2
        ));
    });
}

#[test]
fn proposing_mint_transaction_without_being_validator_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            TFTBridgeModule::propose_or_vote_mint_transaction(
                RuntimeOrigin::signed(alice()),
                "some_tx".as_bytes().to_vec(),
                bob(),
                2
            ),
            Error::<TestRuntime>::ValidatorNotExists
        );
    });
}

#[test]
fn mint_flow() {
    new_test_ext().execute_with(|| {
        prepare_validators();

        assert_ok!(TFTBridgeModule::propose_or_vote_mint_transaction(
            RuntimeOrigin::signed(alice()),
            "some_tx".as_bytes().to_vec(),
            bob(),
            750000000
        ));

        assert_ok!(TFTBridgeModule::propose_or_vote_mint_transaction(
            RuntimeOrigin::signed(bob()),
            "some_tx".as_bytes().to_vec(),
            bob(),
            750000000
        ));
        let mint_tx = TFTBridgeModule::mint_transactions("some_tx".as_bytes().to_vec()).unwrap();
        assert_eq!(mint_tx.votes, 2);

        assert_ok!(TFTBridgeModule::propose_or_vote_mint_transaction(
            RuntimeOrigin::signed(eve()),
            "some_tx".as_bytes().to_vec(),
            bob(),
            750000000
        ));
        let executed_mint_tx =
            TFTBridgeModule::executed_mint_transactions("some_tx".as_bytes().to_vec()).unwrap();
        assert_eq!(executed_mint_tx.votes, 3);

        let b = TFTBridgeModule::get_usable_balance(&bob());
        let balances_as_u128: u128 = b.saturated_into::<u128>();
        assert_eq!(balances_as_u128, 2750000000);

        if let Some(fee_account) = TFTBridgeModule::fee_account() {
            let b = Balances::free_balance(&fee_account);
            let balances_as_u128: u128 = b.saturated_into::<u128>();
            assert_eq!(balances_as_u128, 500000000);
        }
    });
}

#[test]
fn burn_approval_retries_works() {
    new_test_ext().execute_with(|| {
        prepare_validators();
        run_to_block(1);

        assert_ok!(TFTBridgeModule::swap_to_stellar(
            RuntimeOrigin::signed(bob()),
            "GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z"
                .as_bytes()
                .to_vec(),
            2000000000
        ));

        // Advance 41 blocks, so the expiration interval gets triggered twice
        // Should return 2 expire events
        run_to_block(42);

        // We can still approve a burn transaction later on
        assert_ok!(TFTBridgeModule::propose_burn_transaction_or_add_sig(
            RuntimeOrigin::signed(alice()),
            1,
            "GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z"
                .as_bytes()
                .to_vec(),
            1500000000,
            "some_sig".as_bytes().to_vec(),
            "some_stellar_pubkey".as_bytes().to_vec(),
            1
        ));
        assert_ok!(TFTBridgeModule::propose_burn_transaction_or_add_sig(
            RuntimeOrigin::signed(bob()),
            1,
            "GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z"
                .as_bytes()
                .to_vec(),
            1500000000,
            "bob_sig".as_bytes().to_vec(),
            "bob_stellar_pubkey".as_bytes().to_vec(),
            1
        ));
        let burn_tx = TFTBridgeModule::burn_transactions(1);
        assert_eq!(burn_tx.signatures.len(), 2);

        assert_ok!(TFTBridgeModule::propose_burn_transaction_or_add_sig(
            RuntimeOrigin::signed(eve()),
            1,
            "GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z"
                .as_bytes()
                .to_vec(),
            1500000000,
            "some_other_eve_sig".as_bytes().to_vec(),
            "eve_stellar_pubkey".as_bytes().to_vec(),
            1
        ));
        let executed_burn_tx = TFTBridgeModule::burn_transactions(1);
        assert_eq!(executed_burn_tx.signatures.len(), 3);

        // // Test that the expected events were emitted
        // let our_events = System::events()
        //     .into_iter()
        //     .map(|r| r.event)
        //     .filter_map(|e| {
        //         if let Event::pallet_tft_bridge(inner) = e {
        //             Some(inner)
        //         } else {
        //             None
        //         }
        //     })
        //     .collect::<Vec<_>>();

        // for e in our_events.iter() {
        //     println!("event: {:?}", e);
        // }
        // let expected_events: std::vec::Vec<RawEvent<AccountId, BlockNumber>> = vec![
        //     RawEvent::BurnTransactionExpired(
        //         1,
        //         "GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z".as_bytes().to_vec(),
        //         1500000000,
        //     ),
        //     RawEvent::BurnTransactionReady(1),
        // ];
        // // 1st event should be an expire event
        // assert_eq!(our_events[1], expected_events[0]);
        // // 2nd event should be an expire event
        // assert_eq!(our_events[2], expected_events[0]);
        // // 6th event should be burn tx ready event
        // assert_eq!(our_events[6], expected_events[1]);
    });
}

#[test]
fn swap_to_stellar_valid_address_workds() {
    new_test_ext().execute_with(|| {
        assert_ok!(TFTBridgeModule::swap_to_stellar(
            RuntimeOrigin::signed(bob()),
            "GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z"
                .as_bytes()
                .to_vec(),
            2000000000
        ));
    });
}

#[test]
fn swap_to_stellar_non_valid_address_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            TFTBridgeModule::swap_to_stellar(
                RuntimeOrigin::signed(bob()),
                "some_invalid_text".as_bytes().to_vec(),
                2000000000
            ),
            Error::<TestRuntime>::InvalidStellarPublicKey
        );
    });
}

#[test]
fn proposing_burn_transaction_works() {
    new_test_ext().execute_with(|| {
        prepare_validators();

        assert_ok!(TFTBridgeModule::swap_to_stellar(
            RuntimeOrigin::signed(bob()),
            "GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z"
                .as_bytes()
                .to_vec(),
            2000000000
        ));

        assert_ok!(TFTBridgeModule::propose_burn_transaction_or_add_sig(
            RuntimeOrigin::signed(alice()),
            1,
            "GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z"
                .as_bytes()
                .to_vec(),
            1500000000,
            "some_sig".as_bytes().to_vec(),
            "some_stellar_pubkey".as_bytes().to_vec(),
            1
        ));
    });
}

#[test]
fn proposing_burn_transaction_if_no_burn_was_made_fails() {
    new_test_ext().execute_with(|| {
        prepare_validators();

        assert_noop!(
            TFTBridgeModule::propose_burn_transaction_or_add_sig(
                RuntimeOrigin::signed(alice()),
                1,
                "GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z"
                    .as_bytes()
                    .to_vec(),
                2,
                "some_sig".as_bytes().to_vec(),
                "some_stellar_pubkey".as_bytes().to_vec(),
                1
            ),
            Error::<TestRuntime>::BurnTransactionNotExists
        );
    });
}

#[test]
fn proposing_burn_transaction_without_being_validator_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            TFTBridgeModule::propose_burn_transaction_or_add_sig(
                RuntimeOrigin::signed(alice()),
                1,
                "GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z"
                    .as_bytes()
                    .to_vec(),
                2,
                "some_sig".as_bytes().to_vec(),
                "some_stellar_pubkey".as_bytes().to_vec(),
                1
            ),
            Error::<TestRuntime>::ValidatorNotExists
        );
    });
}

#[test]
fn burn_more_than_balance_plus_fee_fails() {
    new_test_ext().execute_with(|| {
        prepare_validators();

        let b = TFTBridgeModule::get_usable_balance(&bob());
        let balances_as_u128: u128 = b.saturated_into::<u128>();
        assert_eq!(balances_as_u128, 2500000000);

        assert_noop!(
            TFTBridgeModule::swap_to_stellar(
                RuntimeOrigin::signed(bob()),
                "GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z"
                    .as_bytes()
                    .to_vec(),
                2500000001
            ),
            Error::<TestRuntime>::NotEnoughBalanceToSwap
        );
    });
}

#[test]
fn burn_locked_tokens_fails() {
    new_test_ext().execute_with(|| {
        prepare_validators();

        let free_balance = Balances::free_balance(&bob());
        assert_eq!(free_balance, 2500000000);

        let locked_balance = 1000000000;
        let id: u64 = 1;
        Balances::set_lock(
            id.to_be_bytes(),
            &bob(),
            locked_balance,
            WithdrawReasons::all(),
        );

        let usable_balance = TFTBridgeModule::get_usable_balance(&bob());
        assert_eq!(usable_balance, 1500000000);

        assert_noop!(
            TFTBridgeModule::swap_to_stellar(
                RuntimeOrigin::signed(bob()),
                "GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z"
                    .as_bytes()
                    .to_vec(),
                usable_balance + 1
            ),
            Error::<TestRuntime>::NotEnoughBalanceToSwap
        );
    });
}

#[test]
fn burn_flow() {
    new_test_ext().execute_with(|| {
        prepare_validators();

        let b = TFTBridgeModule::get_usable_balance(&bob());
        let balances_as_u128: u128 = b.saturated_into::<u128>();
        assert_eq!(balances_as_u128, 2500000000);

        assert_ok!(TFTBridgeModule::swap_to_stellar(
            RuntimeOrigin::signed(bob()),
            "GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z"
                .as_bytes()
                .to_vec(),
            2000000000
        ));

        // amount that needs to be burned is:
        // 2000000000 - fee (500000000)

        assert_ok!(TFTBridgeModule::propose_burn_transaction_or_add_sig(
            RuntimeOrigin::signed(alice()),
            1,
            "GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z"
                .as_bytes()
                .to_vec(),
            1500000000,
            "alice_sig".as_bytes().to_vec(),
            "alice_stellar_pubkey".as_bytes().to_vec(),
            1
        ));

        assert_ok!(TFTBridgeModule::propose_burn_transaction_or_add_sig(
            RuntimeOrigin::signed(bob()),
            1,
            "GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z"
                .as_bytes()
                .to_vec(),
            1500000000,
            "bob_sig".as_bytes().to_vec(),
            "bob_stellar_pubkey".as_bytes().to_vec(),
            1
        ));
        let burn_tx = TFTBridgeModule::burn_transactions(1);
        assert_eq!(burn_tx.signatures.len(), 2);

        assert_ok!(TFTBridgeModule::propose_burn_transaction_or_add_sig(
            RuntimeOrigin::signed(eve()),
            1,
            "GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z"
                .as_bytes()
                .to_vec(),
            1500000000,
            "some_other_eve_sig".as_bytes().to_vec(),
            "eve_stellar_pubkey".as_bytes().to_vec(),
            1
        ));
        let executed_burn_tx = TFTBridgeModule::burn_transactions(1);
        assert_eq!(executed_burn_tx.signatures.len(), 3);

        let b = TFTBridgeModule::get_usable_balance(&bob());
        let balances_as_u128: u128 = b.saturated_into::<u128>();
        assert_eq!(balances_as_u128, 500000000);

        if let Some(fee_account) = TFTBridgeModule::fee_account() {
            let b = Balances::free_balance(&fee_account);
            let balances_as_u128: u128 = b.saturated_into::<u128>();
            assert_eq!(balances_as_u128, 500000000);
        }
    });
}

#[test]
fn burn_flow_expired() {
    new_test_ext().execute_with(|| {
        prepare_validators();

        run_to_block(1);

        let b = TFTBridgeModule::get_usable_balance(&bob());
        let balances_as_u128: u128 = b.saturated_into::<u128>();
        assert_eq!(balances_as_u128, 2500000000);

        assert_ok!(TFTBridgeModule::swap_to_stellar(
            RuntimeOrigin::signed(alice()),
            "GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z"
                .as_bytes()
                .to_vec(),
            750000000
        ));

        // amount that needs to be burned is:
        // 750000000 - fee (500000000)

        assert_ok!(TFTBridgeModule::propose_burn_transaction_or_add_sig(
            RuntimeOrigin::signed(alice()),
            1,
            "GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z"
                .as_bytes()
                .to_vec(),
            250000000,
            "alice_sig".as_bytes().to_vec(),
            "alice_stellar_pubkey".as_bytes().to_vec(),
            1
        ));

        assert_ok!(TFTBridgeModule::propose_burn_transaction_or_add_sig(
            RuntimeOrigin::signed(bob()),
            1,
            "GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z"
                .as_bytes()
                .to_vec(),
            250000000,
            "bob_sig".as_bytes().to_vec(),
            "bob_stellar_pubkey".as_bytes().to_vec(),
            1
        ));
        let burn_tx = TFTBridgeModule::burn_transactions(1);
        assert_eq!(burn_tx.signatures.len(), 2);

        run_to_block(102);
        let burn_tx = TFTBridgeModule::burn_transactions(1);
        assert_eq!(burn_tx.signatures.len(), 0);

        // let expired_burn_tx = TFTBridgeModule::expired_burn_transactions(1);
        // assert_eq!(expired_burn_tx.signatures.len(), 2);

        // // Test that the expected events were emitted
        // let our_events = System::events()
        //     .into_iter()
        //     .map(|r| r.event)
        //     .filter_map(|e| {
        //         if let Event::pallet_tft_bridge(inner) = e {
        //             Some(inner)
        //         } else {
        //             None
        //         }
        //     })
        //     .collect::<Vec<_>>();

        // let expected_events: std::vec::Vec<RawEvent<AccountId, BlockNumber>> =
        //     vec![RawEvent::BurnTransactionExpired(
        //         1,
        //         "GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z".as_bytes().to_vec(),
        //         250000000,
        //     )];
        // assert_eq!(our_events[4], expected_events[0]);

        let burn_tx = TFTBridgeModule::burn_transactions(1);
        assert_eq!(burn_tx.signatures.len(), 0);
        assert_eq!(burn_tx.sequence_number, 0);
    });
}

#[test]
fn burn_fails_if_less_than_withdraw_fee_amount() {
    new_test_ext().execute_with(|| {
        prepare_validators();

        assert_noop!(
            TFTBridgeModule::swap_to_stellar(
                RuntimeOrigin::signed(alice()),
                "GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z"
                    .as_bytes()
                    .to_vec(),
                490000000
            ),
            Error::<TestRuntime>::AmountIsLessThanWithdrawFee
        );
    });
}

fn prepare_validators() {
    TFTBridgeModule::add_bridge_validator(RawOrigin::Root.into(), alice()).unwrap();
    TFTBridgeModule::add_bridge_validator(RawOrigin::Root.into(), bob()).unwrap();
    TFTBridgeModule::add_bridge_validator(RawOrigin::Root.into(), eve()).unwrap();
    TFTBridgeModule::add_bridge_validator(RawOrigin::Root.into(), ferdie()).unwrap();

    TFTBridgeModule::set_fee_account(RawOrigin::Root.into(), ferdie()).unwrap();
    TFTBridgeModule::set_deposit_fee(RawOrigin::Root.into(), 500000000).unwrap();
    TFTBridgeModule::set_withdraw_fee(RawOrigin::Root.into(), 500000000).unwrap();
}

fn run_to_block(n: u64) {
    while System::block_number() < n {
        TFTBridgeModule::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        TFTBridgeModule::on_initialize(System::block_number());
    }
}
