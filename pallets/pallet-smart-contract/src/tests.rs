use crate::{mock::*, Error, RawEvent};
use frame_support::{
    assert_noop, assert_ok,
    traits::{OnFinalize, OnInitialize},
};
use frame_system::RawOrigin;
use sp_runtime::{Perbill, Percent};
use substrate_fixed::types::{U16F16, U64F64};

use super::types;
use pallet_tfgrid::types as pallet_tfgrid_types;

const GIGABYTE: u64 = 1024 * 1024 * 1024;

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
            version: 3,
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

        let node_contract = SmartContractModule::contracts(1);
        assert_eq!(node_contract, types::Contract::default());

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

        let name_contract = SmartContractModule::contracts(1);
        assert_eq!(name_contract, types::Contract::default());

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
fn test_multiple_contracts_billing_loop() {
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
        assert_ok!(SmartContractModule::create_name_contract(
            Origin::signed(bob()),
            "some_name".as_bytes().to_vec(),
        ));

        let contract_to_bill_at_block = SmartContractModule::contract_to_bill_at_block(11);
        assert_eq!(contract_to_bill_at_block.len(), 2);

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

        for event in our_events {
            println!("\nevent: {:?}", event);
        }
    })
}

#[test]
fn test_create_rent_contract() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), U16F16::from_num(0.05), 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(bob()),
            node_id
        ));

        let contract = SmartContractModule::contracts(1);
        let rent_contract = types::RentContract { node_id };
        assert_eq!(
            contract.contract_type,
            types::ContractData::RentContract(rent_contract)
        );
    });
}

#[test]
fn test_create_rent_contract_cancel_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), U16F16::from_num(0.05), 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(bob()),
            node_id
        ));

        let contract = SmartContractModule::contracts(1);
        let rent_contract = types::RentContract { node_id };
        assert_eq!(
            contract.contract_type,
            types::ContractData::RentContract(rent_contract)
        );

        assert_ok!(SmartContractModule::cancel_contract(
            Origin::signed(bob()),
            1
        ));

        let contract = SmartContractModule::contracts(1);
        assert_eq!(
            contract,
            types::Contract::default()
        );
    });
}

#[test]
fn test_create_node_contract_other_owner_when_rent_contract_active_fails() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), U16F16::from_num(0.05), 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(bob()),
            node_id
        ));

        let contract = SmartContractModule::contracts(1);
        let rent_contract = types::RentContract { node_id };
        assert_eq!(
            contract.contract_type,
            types::ContractData::RentContract(rent_contract)
        );

        assert_noop!(
            SmartContractModule::create_node_contract(
                Origin::signed(alice()),
                1,
                "some_data".as_bytes().to_vec(),
                "hash".as_bytes().to_vec(),
                1
            ),
            Error::<TestRuntime>::NodeHasRentContract
        );
    });
}

#[test]
fn test_create_node_contract_same_owner_when_rent_contract_active_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), U16F16::from_num(0.05), 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(bob()),
            node_id
        ));

        let contract = SmartContractModule::contracts(1);
        let rent_contract = types::RentContract { node_id };
        assert_eq!(
            contract.contract_type,
            types::ContractData::RentContract(rent_contract)
        );

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(bob()),
            1,
            "some_data".as_bytes().to_vec(),
            "hash".as_bytes().to_vec(),
            1
        ));
    });
}

#[test]
fn test_create_rent_contract_billing() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), U16F16::from_num(0.05), 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(bob()),
            node_id
        ));

        let contract = SmartContractModule::contracts(1);
        let rent_contract = types::RentContract { node_id };
        assert_eq!(
            contract.contract_type,
            types::ContractData::RentContract(rent_contract)
        );

        run_to_block(12);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(1, 2, 11);
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(1, 1, amount_due_as_u128, 12, discount_received);
    });
}

#[test]
fn test_create_rent_contract_billing_cancel_should_bill_reserved_balance() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), U16F16::from_num(0.05), 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(bob()),
            node_id
        ));

        let contract = SmartContractModule::contracts(1);
        let rent_contract = types::RentContract { node_id };
        assert_eq!(
            contract.contract_type,
            types::ContractData::RentContract(rent_contract)
        );

        run_to_block(12);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(1, 2, 11);
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(1, 1, amount_due_as_u128, 12, discount_received.clone());

        let twin = TfgridModule::twins(2);
        let usable_balance = Balances::usable_balance(&twin.account_id);
        let free_balance = Balances::free_balance(&twin.account_id);
        assert_ne!(usable_balance, free_balance);

        run_to_block(14);
        // cancel contract
        let (amount_due_as_u128, discount_received) = calculate_tft_cost(1, 2, 2);
        assert_ok!(SmartContractModule::cancel_contract(
            Origin::signed(bob()),
            1
        ));

        let twin = TfgridModule::twins(2);
        let usable_balance = Balances::usable_balance(&twin.account_id);
        assert_ne!(usable_balance, 0);
        Balances::transfer(Origin::signed(bob()), alice(), usable_balance).unwrap();

        run_to_block(22);

        // Last amount due is the same as the first one
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(1, 3, amount_due_as_u128, 14, discount_received);

        let usable_balance = Balances::usable_balance(&twin.account_id);
        let free_balance = Balances::free_balance(&twin.account_id);
        assert_eq!(usable_balance, free_balance);
    });
}

#[test]
fn test_rent_contract_canceled_mid_cycle_should_bill_for_remainder() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), U16F16::from_num(0.05), 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(bob()),
            node_id
        ));

        let contract = SmartContractModule::contracts(1);
        let rent_contract = types::RentContract { node_id };
        assert_eq!(
            contract.contract_type,
            types::ContractData::RentContract(rent_contract)
        );

        let twin = TfgridModule::twins(2);
        let usable_balance = Balances::usable_balance(&twin.account_id);
        let free_balance = Balances::free_balance(&twin.account_id);

        let locked_balance = free_balance - usable_balance;
        println!("locked balance: {:?}", locked_balance);

        run_to_block(8);
        // Calculate the cost for 7 blocks of runtime (created a block 1, canceled at block 8)
        let (amount_due_as_u128, discount_received) = calculate_tft_cost(1, 2, 7);
        // cancel rent contract at block 8
        assert_ok!(SmartContractModule::cancel_contract(
            Origin::signed(bob()),
            1
        ));
        
        assert_ne!(amount_due_as_u128, 0);
        
        check_report_cost(1, 2, amount_due_as_u128, 8, discount_received.clone());

        // Twin should have no more locked balance
        let twin = TfgridModule::twins(2);
        let usable_balance = Balances::usable_balance(&twin.account_id);
        let free_balance = Balances::free_balance(&twin.account_id);
        assert_eq!(usable_balance, free_balance);
    });
}

#[test]
fn test_create_rent_contract_and_free_node_contract() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), U16F16::from_num(0.05), 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(bob()),
            node_id
        ));

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(bob()),
            1,
            "some_data".as_bytes().to_vec(),
            "hash".as_bytes().to_vec(),
            0
        ));

        run_to_block(12);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(1, 2, 11);
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(1, 2, amount_due_as_u128, 12, discount_received);

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

        for e in our_events.clone().into_iter() {
            println!("EVEEEENT: {:?}", e);
        }
        // Event 1: Rent contract created
        // Event 2: Node Contract created
        // Event 4: Rent contract billed
        // => no Node Contract billed event
        assert_eq!(our_events.len(), 3);
    });
}

#[test]
fn test_create_rent_contract_and_node_contract_with_ip() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), U16F16::from_num(0.05), 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(bob()),
            node_id
        ));

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(bob()),
            1,
            "some_data".as_bytes().to_vec(),
            "hash".as_bytes().to_vec(),
            1
        ));

        run_to_block(12);

        // check contract 1 costs (Rent Contract)
        let (amount_due_as_u128, discount_received) = calculate_tft_cost(1, 2, 11);
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(1, 2, amount_due_as_u128, 12, discount_received);
        
        // check contract 2 costs (Node Contract)
        let (amount_due_as_u128, discount_received) = calculate_tft_cost(2, 2, 11);
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(2, 3, amount_due_as_u128, 12, discount_received);

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

        // Event 1: Rent contract created
        // Event 2: Node Contract created
        // Event 3: Rent contract billed
        // Event 6: Node Contract billed
        assert_eq!(our_events.len(), 4);
    });
}

#[test]
fn test_node_contract_billing() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(0);
        TFTPriceModule::set_prices(Origin::signed(bob()), U16F16::from_num(0.05), 101).unwrap();

        let twin = TfgridModule::twins(2);
        let initial_twin_balance = Balances::free_balance(&twin.account_id);

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(bob()),
            1,
            "some_data".as_bytes().to_vec(),
            "hash".as_bytes().to_vec(),
            1
        ));

        push_contract_resources_used(1);

        push_nru_report_for_contract(1, 10);

        let contract_to_bill = SmartContractModule::contract_to_bill_at_block(10);
        assert_eq!(contract_to_bill, [1]);

        let initial_total_issuance = Balances::total_issuance();
        
        // advance 25 cycles
        let mut i = 0;
        while i != 24 {
            i +=1;
            run_to_block(i*10+1);
        }

        let free_balance = Balances::free_balance(&twin.account_id);
        let total_amount_billed = initial_twin_balance - free_balance;
        println!("locked balance {:?}", total_amount_billed);
   

        println!("total locked balance {:?}", total_amount_billed);

        let staking_pool_account_balance = Balances::free_balance(&get_staking_pool_account());
        println!("staking pool account balance, {:?}", staking_pool_account_balance);

        // 5% is sent to the staking pool account
        assert_eq!(staking_pool_account_balance, Perbill::from_percent(5) * total_amount_billed);

        // 10% is sent to the foundation account
        let pricing_policy = TfgridModule::pricing_policies(1);
        let foundation_account_balance = Balances::free_balance(&pricing_policy.foundation_account);
        assert_eq!(foundation_account_balance, Perbill::from_percent(10) * total_amount_billed);

        // 50% is sent to the sales account
        let sales_account_balance = Balances::free_balance(&pricing_policy.certified_sales_account);
        assert_eq!(sales_account_balance, Perbill::from_percent(50) * total_amount_billed);

        let total_issuance = Balances::total_issuance();
        // total issueance is now previous total - amount burned from contract billed (35%)
        let burned_amount = Perbill::from_percent(35) * total_amount_billed;
        assert_eq!(
            total_issuance,
            initial_total_issuance - burned_amount as u64 - 1
        );

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
        let contract_id = 1;
        let twin_id = 2;

        push_contract_resources_used(1);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 11);
        run_to_block(12);
        check_report_cost(1, 2, amount_due_as_u128, 12, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(22);
        check_report_cost(1, 3, amount_due_as_u128, 22, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(32);
        check_report_cost(1, 4, amount_due_as_u128, 32, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(42);
        check_report_cost(1, 5, amount_due_as_u128, 42, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(52);
        check_report_cost(1, 6, amount_due_as_u128, 52, discount_received);
    });
}

#[test]
fn test_node_contract_billing_cycles_cancel_contract() {
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

        let contract_id = 1;
        let twin_id = 2;

        push_contract_resources_used(1);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 11);
        run_to_block(12);
        check_report_cost(1, 2, amount_due_as_u128, 12, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(22);
        check_report_cost(1, 3, amount_due_as_u128, 22, discount_received);

        run_to_block(28);
        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 6);
        assert_ok!(SmartContractModule::cancel_contract(
            Origin::signed(bob()),
            1
        ));

        run_to_block(32);
        check_report_cost(1, 5, amount_due_as_u128, 28, discount_received);

        let contract = SmartContractModule::contracts(1);
        assert_eq!(contract.contract_id, 0);

        let billing_info = SmartContractModule::contract_billing_information_by_id(1);
        assert_eq!(billing_info.amount_unbilled, 0);
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

        push_contract_resources_used(1);

        // cycle 1
        run_to_block(12);

        // cycle 2
        // user does not have enough funds to pay for 2 cycles
        run_to_block(22);

        let c1 = SmartContractModule::contracts(1);
        assert_eq!(
            c1,
            types::Contract::default()
        );

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

        let mut expected_events: std::vec::Vec<RawEvent<AccountId, BalanceOf<TestRuntime>>> =
            Vec::new();
        expected_events.push(RawEvent::NodeContractCanceled(1, 1, 3));

        assert_eq!(our_events[5], expected_events[0]);
    });
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
            discount_level: types::DiscountLevel::Gold,
            amount_billed: 2032,
        };
        let expected_events: std::vec::Vec<RawEvent<AccountId, BalanceOf<TestRuntime>>> =
            vec![RawEvent::ContractBilled(contract_bill_event)];
        assert_eq!(our_events[1], expected_events[0]);
    });
}

#[test]
fn test_cu_calculation() {
    new_test_ext().execute_with(|| {
        let cu = U64F64::from_num(4);
        let mru = U64F64::from_num(1024);
        let cu = SmartContractModule::calculate_cu(cu, mru);
        assert_eq!(cu, 128);

        let cu = U64F64::from_num(32);
        let mru = U64F64::from_num(128);
        let cu = SmartContractModule::calculate_cu(cu, mru);
        assert_eq!(cu, 32);

        let cu = U64F64::from_num(4);
        let mru = U64F64::from_num(2);
        let cu = SmartContractModule::calculate_cu(cu, mru);
        assert_eq!(cu, 1);

        let cu = U64F64::from_num(4);
        let mru = U64F64::from_num(1);
        let cu = SmartContractModule::calculate_cu(cu, mru);
        assert_eq!(cu, 1);

        let cu = U64F64::from_num(16);
        let mru = U64F64::from_num(16);
        let cu = SmartContractModule::calculate_cu(cu, mru);
        assert_eq!(cu, 8);
    })
}

#[test]
fn test_percent() {
    let cost: u64 = 1000;
    let new_cost = Percent::from_percent(25) * cost;
    assert_eq!(new_cost, 250);

    let cost: u64 = 1000;
    let new_cost = Percent::from_percent(50) * cost;
    assert_eq!(new_cost, 500);

    let cost: u64 = 992;
    let new_cost = Percent::from_percent(25) * cost;
    assert_eq!(new_cost, 248);
}

// ***** HELPER FUNCTIONS ***** //
// ---------------------------- //
// ---------------------------- //

fn push_nru_report_for_contract(contract_id: u64, block_number: u64) {
    let gigabyte = 1000 * 1000 * 1000;
    let mut consumption_reports = Vec::new();
    consumption_reports.push(super::types::NruConsumption {
        contract_id,
        nru: 3 * gigabyte,
        timestamp: 1628082000 + (6 * block_number),
        window: 6 * block_number,
    });

    assert_ok!(SmartContractModule::add_nru_reports(
        Origin::signed(alice()),
        consumption_reports
    ));
}

fn push_contract_resources_used(contract_id: u64) {
    let mut resources = Vec::new();
    resources.push(types::ContractResources {
        contract_id,
        used: pallet_tfgrid_types::Resources {
            cru: 2,
            hru: 0,
            mru: 2 * GIGABYTE,
            sru: 60 * GIGABYTE,
        },
    });

    assert_ok!(SmartContractModule::report_contract_resources(
        Origin::signed(alice()),
        resources
    ));
}

fn check_report_cost(
    contract_id: u64,
    index: usize,
    amount_billed: u64,
    block_number: u64,
    discount_level: types::DiscountLevel,
) {
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
        contract_id,
        timestamp: 1628082000 + (6 * block_number),
        discount_level,
        amount_billed: amount_billed as u128,
    };
    let mut expected_events: std::vec::Vec<RawEvent<AccountId, BalanceOf<TestRuntime>>> =
        Vec::new();
    expected_events.push(RawEvent::ContractBilled(contract_bill_event));

    for event in our_events.clone().iter() {
        println!("event: {:?}", event);
    };

    assert_eq!(our_events[index], expected_events[0]);
}

fn calculate_tft_cost(
    contract_id: u64,
    twin_id: u32,
    blocks: u64,
) -> (u64, types::DiscountLevel) {
    let twin = TfgridModule::twins(twin_id);
    let b = Balances::free_balance(&twin.account_id);
    let contract = SmartContractModule::contracts(contract_id);
    let (amount_due, discount_received) = SmartContractModule::calculate_contract_cost_tft(&contract, b, blocks*6).unwrap();

    (amount_due, discount_received)
}

pub fn prepare_twins() {
    create_twin(alice());
    create_twin(bob());
    create_twin(charlie());
}

pub fn prepare_farm(source: AccountId) {
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
        50,
    )
    .unwrap();

    TfgridModule::create_farm(
        Origin::signed(source),
        farm_name.as_bytes().to_vec(),
        pub_ips.clone(),
    )
    .unwrap();

    TfgridModule::set_farm_dedicated(
        RawOrigin::Root.into(),
        1,
        true
    )
    .unwrap();
}

pub fn prepare_farm_and_node() {
    prepare_twins();

    prepare_farm(alice());

    // random location
    let location = pallet_tfgrid_types::Location {
        longitude: "12.233213231".as_bytes().to_vec(),
        latitude: "32.323112123".as_bytes().to_vec(),
    };

    let resources = pallet_tfgrid_types::Resources {
        hru: 1024 * GIGABYTE,
        sru: 512 * GIGABYTE,
        cru: 8,
        mru: 16 * GIGABYTE,
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
        false,
        false,
        "some_serial".as_bytes().to_vec(),
    )
    .unwrap();
}

pub fn create_twin(origin: AccountId) {
    let document = "some_link".as_bytes().to_vec();
    let hash = "some_hash".as_bytes().to_vec();

    assert_ok!(TfgridModule::user_accept_tc(
        Origin::signed(origin.clone()),
        document.clone(),
        hash.clone(),
    ));
    let ip = "10.2.3.3";
    TfgridModule::create_twin(Origin::signed(origin), ip.as_bytes().to_vec()).unwrap();
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
