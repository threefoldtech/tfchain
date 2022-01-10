use crate::{mock::*, Error, RawEvent};
use substrate_fixed::types::{U16F16};
use frame_support::{
    assert_noop, assert_ok,
    traits::{OnFinalize, OnInitialize},
};
use frame_system::RawOrigin;
use sp_runtime::traits::SaturatedConversion;

use super::types;
use pallet_tfgrid::types as pallet_tfgrid_types;

#[test]
fn test_create_contract_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            "some_data".as_bytes().to_vec(),
            "hash".as_bytes().to_vec(),
            0
        ));
    });
}

#[test]
fn test_create_node_contract_with_public_ips_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            "some_data".as_bytes().to_vec(),
            "hash".as_bytes().to_vec(),
            1
        ));

        let node_contract = SmartContractModule::contracts(1);

        match node_contract.contract_type.clone() {
            types::ContractData::NodeContract(c) => {
                let farm = TfgridModule::farms(1);
                assert_eq!(farm.public_ips[0].contract_id, 1);

                assert_eq!(c.public_ips, 1);
                assert_eq!(c.public_ips_list[0].ip, "1.1.1.0".as_bytes().to_vec());
            }
            _ => (),
        }
    });
}

#[test]
fn test_create_contract_with_undefined_node_fails() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_noop!(
            SmartContractModule::create_node_contract(
                Origin::signed(alice()),
                2,
                "some_data".as_bytes().to_vec(),
                "hash".as_bytes().to_vec(),
                0
            ),
            Error::<TestRuntime>::NodeNotExists
        );
    });
}

#[test]
fn test_create_contract_with_same_hash_and_node_fails() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            "some_data".as_bytes().to_vec(),
            "hash".as_bytes().to_vec(),
            0
        ));

        assert_noop!(
            SmartContractModule::create_node_contract(
                Origin::signed(alice()),
                1,
                "some_data".as_bytes().to_vec(),
                "hash".as_bytes().to_vec(),
                0
            ),
            Error::<TestRuntime>::ContractIsNotUnique
        );
    });
}

#[test]
fn test_create_contract_which_was_canceled_before_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            "some_data".as_bytes().to_vec(),
            "hash".as_bytes().to_vec(),
            0
        ));
        let contract_id = SmartContractModule::node_contract_by_hash(1, "hash".as_bytes().to_vec());
        assert_eq!(contract_id, 1);

        assert_ok!(SmartContractModule::cancel_contract(
            Origin::signed(alice()),
            1
        ));

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            "some_data".as_bytes().to_vec(),
            "hash".as_bytes().to_vec(),
            0
        ));
        let contract_id = SmartContractModule::node_contract_by_hash(1, "hash".as_bytes().to_vec());
        assert_eq!(contract_id, 2);
    });
}

#[test]
fn test_update_contract_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            "some_data".as_bytes().to_vec(),
            "hash".as_bytes().to_vec(),
            0
        ));

        assert_ok!(SmartContractModule::update_node_contract(
            Origin::signed(alice()),
            1,
            "no_data".as_bytes().to_vec(),
            "some_other_hash".as_bytes().to_vec()
        ));

        let node_contract = types::NodeContract {
            node_id: 1,
            deployment_data: "no_data".as_bytes().to_vec(),
            deployment_hash: "some_other_hash".as_bytes().to_vec(),
            public_ips: 0,
            public_ips_list: Vec::new(),
        };
        let contract_type = types::ContractData::NodeContract(node_contract);

        let expected_contract_value = types::Contract {
            contract_id: 1,
            state: types::ContractState::Created,
            twin_id: 1,
            version: 1,
            contract_type,
        };

        let node_contract = SmartContractModule::contracts(1);
        assert_eq!(node_contract, expected_contract_value);

        let contracts = SmartContractModule::active_node_contracts(1);
        assert_eq!(contracts.len(), 1);

        assert_eq!(contracts[0], 1);

        let node_contract_id_by_hash =
            SmartContractModule::node_contract_by_hash(1, "some_other_hash".as_bytes().to_vec());
        assert_eq!(node_contract_id_by_hash, 1);
    });
}

#[test]
fn test_update_contract_not_exists_fails() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_noop!(
            SmartContractModule::update_node_contract(
                Origin::signed(alice()),
                1,
                "some_data".as_bytes().to_vec(),
                "hash".as_bytes().to_vec()
            ),
            Error::<TestRuntime>::ContractNotExists
        );
    });
}

#[test]
fn test_update_contract_wrong_twins_fails() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            "some_data".as_bytes().to_vec(),
            "hash".as_bytes().to_vec(),
            0
        ));

        assert_noop!(
            SmartContractModule::update_node_contract(
                Origin::signed(bob()),
                1,
                "some_data".as_bytes().to_vec(),
                "hash".as_bytes().to_vec()
            ),
            Error::<TestRuntime>::TwinNotAuthorizedToUpdateContract
        );
    });
}

#[test]
fn test_cancel_contract_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            "some_data".as_bytes().to_vec(),
            "hash".as_bytes().to_vec(),
            0
        ));

        assert_ok!(SmartContractModule::cancel_contract(
            Origin::signed(alice()),
            1
        ));

        let node_contract = types::NodeContract {
            node_id: 1,
            deployment_data: "some_data".as_bytes().to_vec(),
            deployment_hash: "hash".as_bytes().to_vec(),
            public_ips: 0,
            public_ips_list: Vec::new(),
        };
        let contract_type = types::ContractData::NodeContract(node_contract);

        let expected_contract_value = types::Contract {
            contract_id: 1,
            state: types::ContractState::Deleted(types::Cause::CanceledByUser),
            twin_id: 1,
            version: 1,
            contract_type,
        };

        let node_contract = SmartContractModule::contracts(1);
        assert_eq!(node_contract, expected_contract_value);

        let contracts = SmartContractModule::active_node_contracts(1);
        assert_eq!(contracts.len(), 0);
    });
}

#[test]
fn test_cancel_name_contract_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_name_contract(
            Origin::signed(alice()),
            "some_name".as_bytes().to_vec()
        ));

        assert_ok!(SmartContractModule::cancel_contract(
            Origin::signed(alice()),
            1
        ));

        let name_contract = types::NameContract {
            name: "some_name".as_bytes().to_vec(),
        };
        let contract_type = types::ContractData::NameContract(name_contract);

        let expected_contract_value = types::Contract {
            contract_id: 1,
            state: types::ContractState::Deleted(types::Cause::CanceledByUser),
            twin_id: 1,
            version: 1,
            contract_type,
        };

        let name_contract = SmartContractModule::contracts(1);
        assert_eq!(name_contract, expected_contract_value);

        let contract_id =
            SmartContractModule::contract_id_by_name_registration("some_name".as_bytes().to_vec());
        assert_eq!(contract_id, 0);
    });
}

#[test]
fn test_create_multiple_contracts_work() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            "some_data".as_bytes().to_vec(),
            "hash1".as_bytes().to_vec(),
            0
        ));

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            "some_data2".as_bytes().to_vec(),
            "hash2".as_bytes().to_vec(),
            0
        ));

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            "some_data3".as_bytes().to_vec(),
            "hash3".as_bytes().to_vec(),
            0
        ));

        let node_contracts = SmartContractModule::active_node_contracts(1);
        assert_eq!(node_contracts.len(), 3);

        // now cancel 1 and check if the storage maps are updated correctly
        assert_ok!(SmartContractModule::cancel_contract(
            Origin::signed(alice()),
            1
        ));

        let node_contracts = SmartContractModule::active_node_contracts(1);
        assert_eq!(node_contracts.len(), 2);
    });
}

#[test]
fn test_cancel_contract_works_public_ips_frees_ip() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            "some_data".as_bytes().to_vec(),
            "hash".as_bytes().to_vec(),
            1
        ));

        let farm = TfgridModule::farms(1);
        assert_eq!(farm.public_ips[0].contract_id, 1);

        assert_ok!(SmartContractModule::cancel_contract(
            Origin::signed(alice()),
            1
        ));

        let farm = TfgridModule::farms(1);
        assert_eq!(farm.public_ips[0].contract_id, 0);
    });
}

#[test]
fn test_cancel_contract_not_exists_fails() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_noop!(
            SmartContractModule::cancel_contract(Origin::signed(alice()), 1),
            Error::<TestRuntime>::ContractNotExists
        );
    });
}

#[test]
fn test_cancel_contract_wrong_twins_fails() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            "some_data".as_bytes().to_vec(),
            "hash".as_bytes().to_vec(),
            0
        ));

        assert_noop!(
            SmartContractModule::cancel_contract(Origin::signed(bob()), 1),
            Error::<TestRuntime>::TwinNotAuthorizedToCancelContract
        );
    });
}

#[test]
fn test_create_name_contract() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_name_contract(
            Origin::signed(bob()),
            "foobar".as_bytes().to_vec()
        ));
    });
}

#[test]
fn test_create_name_contract_double_with_same_name_fails() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_name_contract(
            Origin::signed(bob()),
            "foobar".as_bytes().to_vec()
        ));
        assert_noop!(
            SmartContractModule::create_name_contract(
                Origin::signed(alice()),
                "foobar".as_bytes().to_vec()
            ),
            Error::<TestRuntime>::NameExists
        );
    });
}

#[test]
fn test_recreate_name_contract_after_cancel_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_name_contract(
            Origin::signed(bob()),
            "foobar".as_bytes().to_vec()
        ));

        assert_ok!(SmartContractModule::cancel_contract(
            Origin::signed(bob()),
            1
        ));

        assert_ok!(SmartContractModule::create_name_contract(
            Origin::signed(bob()),
            "foobar".as_bytes().to_vec()
        ));
    });
}

#[test]
fn test_name_registration_fails_with_invalid_dns_name() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_noop!(
            SmartContractModule::create_name_contract(
                Origin::signed(alice()),
                "foo.bar".as_bytes().to_vec()
            ),
            Error::<TestRuntime>::NameNotValid
        );

        assert_noop!(
            SmartContractModule::create_name_contract(
                Origin::signed(alice()),
                "foo!".as_bytes().to_vec()
            ),
            Error::<TestRuntime>::NameNotValid
        );

        assert_noop!(
            SmartContractModule::create_name_contract(
                Origin::signed(alice()),
                "foo;'".as_bytes().to_vec()
            ),
            Error::<TestRuntime>::NameNotValid
        );

        assert_noop!(
            SmartContractModule::create_name_contract(
                Origin::signed(alice()),
                "foo123.%".as_bytes().to_vec()
            ),
            Error::<TestRuntime>::NameNotValid
        );
    });
}

#[test]
fn test_contract_billing_loop() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        TFTPriceModule::set_prices(Origin::signed(bob()), U16F16::from_num(0.05), 1).unwrap();
        run_to_block(1);

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(bob()),
            1,
            "some_data".as_bytes().to_vec(),
            "hash".as_bytes().to_vec(),
            1
        ));

        let contract_to_bill_at_block = SmartContractModule::contract_to_bill_at_block(11);
        assert_eq!(contract_to_bill_at_block.len(), 1);

        run_to_block(12);
        let contract_to_bill_at_block = SmartContractModule::contract_to_bill_at_block(21);
        assert_eq!(contract_to_bill_at_block.len(), 1);

        run_to_block(22);
        let contract_to_bill_at_block = SmartContractModule::contract_to_bill_at_block(31);
        assert_eq!(contract_to_bill_at_block.len(), 1);

        run_to_block(31);
        assert_ok!(SmartContractModule::create_name_contract(
            Origin::signed(bob()),
            "foobar".as_bytes().to_vec()
        ));

        run_to_block(32);
        let contract_to_bill_at_block = SmartContractModule::contract_to_bill_at_block(41);
        assert_eq!(contract_to_bill_at_block.len(), 2);

        run_to_block(42);
        let contract_to_bill_at_block = SmartContractModule::contract_to_bill_at_block(51);
        assert_eq!(contract_to_bill_at_block.len(), 2);

        run_to_block(52);
        assert_ok!(SmartContractModule::cancel_contract(
            Origin::signed(bob()),
            2
        ));

        // after a canceling the second contract it should still be in the contract to be billed map
        // but it should be removed from the next billing cycle since it's canceled and it does not have unbilled amounts
        let contract_to_bill_at_block = SmartContractModule::contract_to_bill_at_block(61);
        assert_eq!(contract_to_bill_at_block.len(), 2);

        run_to_block(62);
        let contract_to_bill_at_block = SmartContractModule::contract_to_bill_at_block(71);
        assert_eq!(contract_to_bill_at_block.len(), 1);
    })
}

#[test]
fn test_node_contract_billing() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), U16F16::from_num(0.05), 101).unwrap();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(bob()),
            1,
            "some_data".as_bytes().to_vec(),
            "hash".as_bytes().to_vec(),
            1
        ));

        let contract_to_bill = SmartContractModule::contract_to_bill_at_block(11);
        assert_eq!(contract_to_bill, [1]);

        let initial_total_issuance = Balances::total_issuance();

        push_report(11);
        run_to_block(12);
        check_report_cost(3, 36070, 12, types::DiscountLevel::Gold);

        // check the contract owners address to see if it got balance credited
        let twin = TfgridModule::twins(2);
        let b = Balances::free_balance(&twin.account_id);
        let balances_as_u128: u128 = b.saturated_into::<u128>();

        let twin2_balance_should_be = 2500000000 - 36070 as u128;
        assert_eq!(balances_as_u128, twin2_balance_should_be);
        
        let staking_pool_account_balance = Balances::free_balance(&get_staking_pool_account());
        let staking_pool_account_balance_as_u128: u128 = staking_pool_account_balance.saturated_into::<u128>();
        // equal to 5%
        assert_eq!(staking_pool_account_balance_as_u128, 1803);

        let pricing_policy = TfgridModule::pricing_policies(1);
        let foundation_account_balance = Balances::free_balance(&pricing_policy.foundation_account);
        let foundation_account_balance_as_u128: u128 = foundation_account_balance.saturated_into::<u128>();
        // equal to 10%
        assert_eq!(foundation_account_balance_as_u128, 3607);

        let sales_account_balance = Balances::free_balance(&pricing_policy.certified_sales_account);
        let sales_account_balance_as_u128: u128 = sales_account_balance.saturated_into::<u128>();
        // equal to 50%
        assert_eq!(sales_account_balance_as_u128, 18035);

        let total_issuance = Balances::total_issuance();
        // total issueance is now previous total - amount burned from contract billed (35%)
        assert_eq!(total_issuance, initial_total_issuance - 12625);

        // amount unbilled should have been reset after a transfer between contract owner and farmer
        let contract_billing_info = SmartContractModule::contract_billing_information_by_id(1);
        assert_eq!(contract_billing_info.amount_unbilled, 0);
    });
}

#[test]
fn test_node_contract_billing_cycles() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), U16F16::from_num(0.05), 101).unwrap();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(bob()),
            1,
            "some_data".as_bytes().to_vec(),
            "hash".as_bytes().to_vec(),
            0
        ));

        push_report(11);
        run_to_block(12);
        check_report_cost(3, 25895, 12, types::DiscountLevel::Gold);

        push_report(21);
        run_to_block(22);
        check_report_cost(6, 25559, 22, types::DiscountLevel::Gold);

        push_report(31);
        run_to_block(32);
        check_report_cost(9, 25559, 32, types::DiscountLevel::Gold);

        push_report(41);
        run_to_block(42);
        check_report_cost(12, 25559, 42, types::DiscountLevel::Gold);

        push_report(51);
        run_to_block(52);
        check_report_cost(15, 25559, 52, types::DiscountLevel::Gold);
    });
}

#[test]
fn test_node_contract_billing_should_cancel_contract_when_out_of_funds() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), U16F16::from_num(0.05), 101).unwrap();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(charlie()),
            1,
            "some_data".as_bytes().to_vec(),
            "hash".as_bytes().to_vec(),
            0
        ));

        push_report(11);
        run_to_block(12);
        check_report_cost(3, 64736, 12, types::DiscountLevel::None);

        let twin = TfgridModule::twins(3);
        let b = Balances::free_balance(&twin.account_id);
        let balances_as_u128: u128 = b.saturated_into::<u128>();

        let twin2_balance_should_be = 100000 - 64736 as u128;
        assert_eq!(balances_as_u128, twin2_balance_should_be);

        push_report(21);
        run_to_block(22);
        check_report_cost(6, 35264, 22, types::DiscountLevel::None);

        let twin = TfgridModule::twins(3);
        let b = Balances::free_balance(&twin.account_id);
        assert_eq!(b, 1);

        let c1 = SmartContractModule::contracts(1);
        assert_eq!(c1.state, types::ContractState::Deleted(types::Cause::OutOfFunds));

        let contract_billing_info = SmartContractModule::contract_billing_information_by_id(1);
        assert_eq!(contract_billing_info.amount_unbilled, 0); //this amount in unit USD = 1/1e7

        let our_events = System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(|e| {
            if let Event::pallet_smart_contract(inner) = e {
                Some(inner)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
        
        let mut expected_events: std::vec::Vec<RawEvent<AccountId, BalanceOf<TestRuntime>>> = Vec::new();
        expected_events.push(RawEvent::NodeContractCanceled(1, 1, 3));

        assert_eq!(our_events[7], expected_events[0]);
    });
}

#[test]
fn test_new_contract_bill() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), U16F16::from_num(0.062), 101).unwrap();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            "some_data".as_bytes().to_vec(),
            "hash".as_bytes().to_vec(),
            1
        ));

        let gigabyte = 1000 * 1000 * 1000;
        let mut consumption_reports = Vec::new();
        consumption_reports.push(super::types::Consumption {
            contract_id: 1,
            cru: 2,
            hru: 0,
            mru: 8 * gigabyte,
            sru: 25 * gigabyte,
            nru: 0,
            timestamp: 1628082000 + (11*6),
        });
    
        assert_ok!(SmartContractModule::add_reports(
            Origin::signed(alice()),
            consumption_reports
        ));

        run_to_block(12);

        // Test that the expected events were emitted
        let our_events = System::events()
        .into_iter()
        .map(|r| r.event)
        .filter_map(|e| {
            if let Event::pallet_smart_contract(inner) = e {
                Some(inner)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

        let contract_bill_event = types::ContractBill {
            contract_id: 1,
            timestamp: 1628082072,
            discount_level: types::DiscountLevel::Gold,
            amount_billed: 42816
        };
        let mut expected_events: std::vec::Vec<RawEvent<AccountId, BalanceOf<TestRuntime>>> = Vec::new();
        expected_events.push(RawEvent::ContractBilled(contract_bill_event));

        assert_eq!(our_events[3], expected_events[0]);
    })
}

fn push_report(block_number: u64) {
    let gigabyte = 1000 * 1000 * 1000;
    let mut consumption_reports = Vec::new();
    consumption_reports.push(super::types::Consumption {
        contract_id: 1,
        cru: 2,
        hru: 0,
        mru: 2 * gigabyte,
        sru: 60 * gigabyte,
        nru: 3 * gigabyte,
        timestamp: 1628082000 + (6*block_number),
    });

    assert_ok!(SmartContractModule::add_reports(
        Origin::signed(alice()),
        consumption_reports
    ));
}

fn check_report_cost(index: usize, amount_billed: u128, block_number: u64, discount_level: types::DiscountLevel) {
    // Test that the expected events were emitted
    let our_events = System::events()
    .into_iter()
    .map(|r| r.event)
    .filter_map(|e| {
        if let Event::pallet_smart_contract(inner) = e {
            Some(inner)
        } else {
            None
        }
    })
    .collect::<Vec<_>>();

    let contract_bill_event = types::ContractBill {
        contract_id: 1,
        timestamp: 1628082000 + (6*block_number),
        discount_level,
        amount_billed
    };
    let mut expected_events: std::vec::Vec<RawEvent<AccountId, BalanceOf<TestRuntime>>> = Vec::new();
    expected_events.push(RawEvent::ContractBilled(contract_bill_event));

    assert_eq!(our_events[index], expected_events[0]);
}

#[test]
fn test_name_contract_billing() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), U16F16::from_num(0.05), 101).unwrap();

        assert_ok!(SmartContractModule::create_name_contract(
            Origin::signed(bob()),
            "foobar".as_bytes().to_vec()
        ));

        let contract_to_bill = SmartContractModule::contract_to_bill_at_block(11);
        assert_eq!(contract_to_bill, [1]);

        // let mature 11 blocks
        // because we bill every 10 blocks
        run_to_block(12);

        // Test that the expected events were emitted
        let our_events = System::events()
            .into_iter()
            .map(|r| r.event)
            .filter_map(|e| {
                if let Event::pallet_smart_contract(inner) = e {
                    Some(inner)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let contract_bill_event = types::ContractBill {
            contract_id: 1,
            timestamp: 1628082072,
            discount_level: types::DiscountLevel::None,
            amount_billed: 277983,
        };
        let expected_events: std::vec::Vec<RawEvent<AccountId, BalanceOf<TestRuntime>>> =
            vec![RawEvent::ContractBilled(contract_bill_event)];
        assert_eq!(our_events[1], expected_events[0]);
    });
}

fn prepare_farm_and_node() {
    let document = "some_link".as_bytes().to_vec();
    let hash = "some_hash".as_bytes().to_vec();

    assert_ok!(TfgridModule::user_accept_tc(
        Origin::signed(alice()),
        document.clone(),
        hash.clone(),
    ));
    let ip = "10.2.3.3";
    TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()).unwrap();

    let ip = "10.2.3.3";
    assert_ok!(TfgridModule::user_accept_tc(
        Origin::signed(bob()),
        document.clone(),
        hash.clone(),
    ));
    TfgridModule::create_twin(Origin::signed(bob()), ip.as_bytes().to_vec()).unwrap();

    let ip = "10.2.3.3";
    assert_ok!(TfgridModule::user_accept_tc(
        Origin::signed(charlie()),
        document,
        hash,
    ));
    TfgridModule::create_twin(Origin::signed(charlie()), ip.as_bytes().to_vec()).unwrap();

    let farm_name = "test_farm";
    let mut pub_ips = Vec::new();
    pub_ips.push(pallet_tfgrid_types::PublicIP {
        ip: "1.1.1.0".as_bytes().to_vec(),
        gateway: "1.1.1.1".as_bytes().to_vec(),
        contract_id: 0,
    });

    let su_policy = pallet_tfgrid_types::Policy {
        value: 194400,
        unit: pallet_tfgrid_types::Unit::Gigabytes,
    };
    let nu_policy = pallet_tfgrid_types::Policy {
        value: 50000,
        unit: pallet_tfgrid_types::Unit::Gigabytes,
    };
    let cu_policy = pallet_tfgrid_types::Policy {
        value: 305600,
        unit: pallet_tfgrid_types::Unit::Gigabytes,
    };
    let ipu_policy = pallet_tfgrid_types::Policy {
        value: 69400,
        unit: pallet_tfgrid_types::Unit::Gigabytes,
    };
    let unique_name_policy = pallet_tfgrid_types::Policy {
        value: 13900,
        unit: pallet_tfgrid_types::Unit::Gigabytes,
    };
    let domain_name_policy = pallet_tfgrid_types::Policy {
        value: 27800,
        unit: pallet_tfgrid_types::Unit::Gigabytes,
    };

    TfgridModule::create_pricing_policy(
        RawOrigin::Root.into(),
        "policy_1".as_bytes().to_vec(),
        su_policy,
        cu_policy,
        nu_policy,
        ipu_policy,
        unique_name_policy,
        domain_name_policy,
        ferdie(),
        eve(),
    )
    .unwrap();

    let document = "some_link".as_bytes().to_vec();
    let hash = "some_hash".as_bytes().to_vec();

    assert_ok!(TfgridModule::farmer_accept_tc(
        Origin::signed(alice()),
        document,
        hash,
    ));
    TfgridModule::create_farm(
        Origin::signed(alice()),
        farm_name.as_bytes().to_vec(),
        pub_ips.clone(),
    )
    .unwrap();

    // random location
    let location = pallet_tfgrid_types::Location {
        longitude: "12.233213231".as_bytes().to_vec(),
        latitude: "32.323112123".as_bytes().to_vec(),
    };

    let resources = pallet_tfgrid_types::Resources {
        hru: 1,
        sru: 1,
        cru: 1,
        mru: 1,
    };

    let country = "Belgium".as_bytes().to_vec();
    let city = "Ghent".as_bytes().to_vec();
    TfgridModule::create_node(
        Origin::signed(alice()),
        1,
        resources,
        location,
        country,
        city,
        Vec::new(),
    )
    .unwrap();
}

fn run_to_block(n: u64) {
    Timestamp::set_timestamp((1628082000 * 1000) + (6000 * n));
    while System::block_number() < n {
        SmartContractModule::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        SmartContractModule::on_initialize(System::block_number());
    }
}
