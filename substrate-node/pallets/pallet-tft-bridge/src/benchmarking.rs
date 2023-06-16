#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as TFTBridgeModule;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::{EventRecord, Pallet as System, RawOrigin};
use pallet_balances::Pallet as Balances;
use sp_runtime::{traits::StaticLookup, SaturatedConversion};

benchmarks! {
    where_clause {
        where
        T: pallet_balances::Config<Balance = BalanceOf<T>>,
    }

    // add_bridge_validator
    add_bridge_validator {
        let validator: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Root, validator.clone())
    verify {
        let validators = TFTBridgeModule::<T>::validator_accounts();
        assert!(validators.contains(&validator));
    }

    // remove_bridge_validator
    remove_bridge_validator {
        let validator: T::AccountId = whitelisted_caller();

        assert_ok!(TFTBridgeModule::<T>::add_bridge_validator(
            RawOrigin::Root.into(),
            validator.clone()
        ));
    }: _(RawOrigin::Root, validator.clone())
    verify {
        let validators = TFTBridgeModule::<T>::validator_accounts();
        assert!(!validators.contains(&validator));
    }

    // set_fee_account
    set_fee_account {
        let fee_account: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Root, fee_account.clone())
    verify {
        assert_eq!(TFTBridgeModule::<T>::fee_account(), Some(fee_account));
    }

    // set_withdraw_fee
    set_withdraw_fee {
        let withdraw_fee = 100;
    }: _(RawOrigin::Root, withdraw_fee)
    verify {
        assert_eq!(TFTBridgeModule::<T>::withdraw_fee(), withdraw_fee);
    }

    // set_deposit_fee
    set_deposit_fee {
        let deposit_fee = 100;
    }: _(RawOrigin::Root, deposit_fee)
    verify {
        assert_eq!(TFTBridgeModule::<T>::deposit_fee(), deposit_fee);
    }

    // swap_to_stellar
    swap_to_stellar {
        _prepare_validators::<T>();

        let caller: T::AccountId = whitelisted_caller();
        let caller_lookup = T::Lookup::unlookup(caller.clone());
        let balance_init_amount = <T as pallet_balances::Config>::Balance::saturated_from(1500000000 as u128);
        Balances::<T>::force_set_balance(RawOrigin::Root.into(), caller_lookup, balance_init_amount).unwrap();

        let target_stellar_address = b"GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z".to_vec();
        let amount = <T as pallet_balances::Config>::Balance::saturated_from(1000000000 as u128);
    }: _(RawOrigin::Signed(caller.clone()), target_stellar_address.clone(), amount)
    verify {
        let burn_id = 1;
        let tx = TFTBridgeModule::<T>::burn_transactions(burn_id);
        assert_last_event::<T>(Event::BurnTransactionCreated(
            burn_id,
            caller,
            tx.target,
            tx.amount,
        ).into());
    }

    // propose_or_vote_mint_transaction
    propose_or_vote_mint_transaction {
        _prepare_validators::<T>();

        let tx_id = b"some_tx".to_vec();
        let target: T::AccountId = whitelisted_caller();
        let amount = 1000000000;

        assert_ok!(TFTBridgeModule::<T>::propose_or_vote_mint_transaction(
            RawOrigin::Signed(account("Bob", 0, 1)).into(),
            tx_id.clone(),
            target.clone(),
            amount
        ));

        assert_ok!(TFTBridgeModule::<T>::propose_or_vote_mint_transaction(
            RawOrigin::Signed(account("Ferdie", 0, 2)).into(),
            tx_id.clone(),
            target.clone(),
            amount
        ));

        let validator: T::AccountId = account("Alice", 0, 0);
    }: _(RawOrigin::Signed(validator), tx_id, target.clone(), amount)
    verify {
        let block = System::<T>::block_number();
        let mint_tx = MintTransaction { amount, target, block, votes: 3 };
        assert_last_event::<T>(Event::MintCompleted(mint_tx).into());
    }

    // propose_burn_transaction_or_add_sig
    propose_burn_transaction_or_add_sig {
        _prepare_validators::<T>();

        let caller: T::AccountId = whitelisted_caller();
        let caller_lookup = T::Lookup::unlookup(caller.clone());
        let balance_init_amount = <T as pallet_balances::Config>::Balance::saturated_from(1500000000 as u128);
        Balances::<T>::force_set_balance(RawOrigin::Root.into(), caller_lookup, balance_init_amount).unwrap();

        let target_stellar_address = b"GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z".to_vec();
        let swap_amount = <T as pallet_balances::Config>::Balance::saturated_from(1000000000 as u128);

        assert_ok!(TFTBridgeModule::<T>::swap_to_stellar(
            RawOrigin::Signed(caller.clone()).into(),
            target_stellar_address.clone(),
            swap_amount
        ));

        let tx_id = 1;
        let tx_amount = 500000000;
        let sequence_number = 1;

        let bob_sig = b"bob_sig".to_vec();
        let bob_pubkey = b"bob_stellar_pubkey".to_vec();
        assert_ok!(TFTBridgeModule::<T>::propose_burn_transaction_or_add_sig(
            RawOrigin::Signed(account("Bob", 0, 1)).into(),
            tx_id,
            target_stellar_address.clone(),
            tx_amount,
            bob_sig,
            bob_pubkey,
            sequence_number
        ));

        let ferdie_sig = b"ferdie_sig".to_vec();
        let ferdie_pubkey = b"ferdie_stellar_pubkey".to_vec();
        assert_ok!(TFTBridgeModule::<T>::propose_burn_transaction_or_add_sig(
            RawOrigin::Signed(account("Ferdie", 0, 2)).into(),
            tx_id,
            target_stellar_address.clone(),
            tx_amount,
            ferdie_sig,
            ferdie_pubkey,
            sequence_number
        ));

        let validator: T::AccountId = account("Alice", 0, 0);
        let alice_sig = b"alice_sig".to_vec();
        let alice_pubkey = b"alice_stellar_pubkey".to_vec();
    }: _(
        RawOrigin::Signed(validator),
        tx_id,
        target_stellar_address.clone(),
        tx_amount,
        alice_sig,
        alice_pubkey,
        sequence_number
    )
    verify {
        assert_last_event::<T>(Event::BurnTransactionReady(tx_id).into());
    }

    // set_burn_transaction_executed
    set_burn_transaction_executed {
        _prepare_validators::<T>();

        let caller: T::AccountId = whitelisted_caller();
        let caller_lookup = T::Lookup::unlookup(caller.clone());
        let balance_init_amount = <T as pallet_balances::Config>::Balance::saturated_from(1500000000 as u128);
        Balances::<T>::force_set_balance(RawOrigin::Root.into(), caller_lookup, balance_init_amount).unwrap();

        let target_stellar_address = b"GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z".to_vec();
        let swap_amount = <T as pallet_balances::Config>::Balance::saturated_from(1000000000 as u128);

        assert_ok!(TFTBridgeModule::<T>::swap_to_stellar(
            RawOrigin::Signed(caller.clone()).into(),
            target_stellar_address.clone(),
            swap_amount
        ));

        let tx_id = 1;
        let tx = TFTBridgeModule::<T>::burn_transactions(tx_id);

        let validator: T::AccountId = account("Alice", 0, 0);
    }: _(RawOrigin::Signed(validator), tx_id)
    verify {
        assert_last_event::<T>(Event::BurnTransactionProcessed(tx).into());
    }

    // create_refund_transaction_or_add_sig
    create_refund_transaction_or_add_sig {
        _prepare_validators::<T>();

        let validator: T::AccountId = account("Alice", 0, 0);
        let tx_hash = b"some_tx_hash".to_vec();
        let target_stellar_address = b"GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z".to_vec();
        let amount = 10000000;
        let alice_sig = b"alice_sig".to_vec();
        let alice_pubkey = b"alice_stellar_pubkey".to_vec();
        let sequence_number = 1;
    }: _(
        RawOrigin::Signed(validator),
        tx_hash.clone(),
        target_stellar_address.clone(),
        amount,
        alice_sig,
        alice_pubkey,
        sequence_number
    )
    verify {
        assert_last_event::<T>(Event::RefundTransactionCreated(
            tx_hash,
            target_stellar_address,
            amount,
        ).into());
    }

    // set_refund_transaction_executed
    set_refund_transaction_executed {
        _prepare_validators::<T>();

        let validator: T::AccountId = account("Alice", 0, 0);
        let tx_hash = b"some_tx_hash".to_vec();
        let target_stellar_address = b"GBIYYEQO73AYJEADTHMTF5M42WICTHU55IIT2CPEZBBLLDSJ322OGW7Z".to_vec();
        let amount = 10000000;
        let alice_sig = b"alice_sig".to_vec();
        let alice_pubkey = b"alice_stellar_pubkey".to_vec();
        let sequence_number = 1;
        assert_ok!(TFTBridgeModule::<T>::create_refund_transaction_or_add_sig(
            RawOrigin::Signed(validator.clone()).into(),
            tx_hash.clone(),
            target_stellar_address,
            amount,
            alice_sig,
            alice_pubkey,
            sequence_number
        ));

        let tx = TFTBridgeModule::<T>::refund_transactions(tx_hash.clone());
    }: _(RawOrigin::Signed(validator), tx_hash)
    verify {
        assert_last_event::<T>(Event::RefundTransactionProcessed(tx).into());
    }

    // Calling the `impl_benchmark_test_suite` macro inside the `benchmarks`
    // block will generate one #[test] function per benchmark
    impl_benchmark_test_suite!(TFTBridgeModule, crate::mock::new_test_ext(), crate::mock::TestRuntime)
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    let events = System::<T>::events();
    let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}

fn _prepare_validators<T: Config>() {
    assert_ok!(TFTBridgeModule::<T>::add_bridge_validator(
        RawOrigin::Root.into(),
        account("Alice", 0, 0)
    ));
    assert_ok!(TFTBridgeModule::<T>::add_bridge_validator(
        RawOrigin::Root.into(),
        account("Bob", 0, 1)
    ));
    assert_ok!(TFTBridgeModule::<T>::add_bridge_validator(
        RawOrigin::Root.into(),
        account("Ferdie", 0, 2)
    ));
    assert_ok!(TFTBridgeModule::<T>::add_bridge_validator(
        RawOrigin::Root.into(),
        account("Eve", 0, 3)
    ));

    assert_ok!(TFTBridgeModule::<T>::set_fee_account(
        RawOrigin::Root.into(),
        account("Ferdie", 0, 2)
    ));
    assert_ok!(TFTBridgeModule::<T>::set_deposit_fee(
        RawOrigin::Root.into(),
        500000000
    ));
    assert_ok!(TFTBridgeModule::<T>::set_withdraw_fee(
        RawOrigin::Root.into(),
        500000000
    ));
}
