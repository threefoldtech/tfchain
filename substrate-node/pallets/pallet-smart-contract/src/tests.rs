use super::Event as SmartContractEvent;
use crate::{mock::Event as MockEvent, mock::*, Error};
use frame_support::{
    assert_noop, assert_ok, bounded_vec,
    traits::{LockableCurrency, OnFinalize, OnInitialize, WithdrawReasons},
    BoundedVec,
};
use frame_system::{EventRecord, Phase, RawOrigin};
use sp_core::H256;
use sp_runtime::{assert_eq_error_rate, traits::SaturatedConversion, Perbill, Percent};
use substrate_fixed::types::U64F64;

use super::types;
use crate::cost;
use pallet_tfgrid::types as pallet_tfgrid_types;
use sp_std::convert::{TryFrom, TryInto};
use tfchain_support::types::{FarmCertification, Location, NodeCertification, PublicIP, Resources};

const GIGABYTE: u64 = 1024 * 1024 * 1024;

//  NODE CONTRACT TESTS //
// -------------------- //

#[test]
fn test_create_node_contract_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
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
            generate_deployment_hash(),
            get_deployment_data(),
            2,
            None
        ));

        let node_contract = SmartContractModule::contracts(1).unwrap();

        match node_contract.contract_type.clone() {
            types::ContractData::NodeContract(c) => {
                let farm = TfgridModule::farms(1).unwrap();
                assert_eq!(farm.public_ips[0].contract_id, 1);

                assert_eq!(c.public_ips, 2);

                let pub_ip = PublicIP {
                    ip: "185.206.122.33/24".as_bytes().to_vec().try_into().unwrap(),
                    gateway: "185.206.122.1".as_bytes().to_vec().try_into().unwrap(),
                    contract_id: 1,
                };

                let pub_ip_2 = PublicIP {
                    ip: "185.206.122.34/24".as_bytes().to_vec().try_into().unwrap(),
                    gateway: "185.206.122.1".as_bytes().to_vec().try_into().unwrap(),
                    contract_id: 1,
                };
                assert_eq!(c.public_ips_list[0], pub_ip);
                assert_eq!(c.public_ips_list[1], pub_ip_2);
            }
            _ => (),
        }
    });
}

#[test]
fn test_create_node_contract_with_undefined_node_fails() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_noop!(
            SmartContractModule::create_node_contract(
                Origin::signed(alice()),
                2,
                generate_deployment_hash(),
                get_deployment_data(),
                0,
                None
            ),
            Error::<TestRuntime>::NodeNotExists
        );
    });
}

#[test]
fn test_create_node_contract_with_same_hash_and_node_fails() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        let h = generate_deployment_hash();
        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            h,
            get_deployment_data(),
            0,
            None
        ));

        assert_noop!(
            SmartContractModule::create_node_contract(
                Origin::signed(alice()),
                1,
                h,
                get_deployment_data(),
                0,
                None
            ),
            Error::<TestRuntime>::ContractIsNotUnique
        );
    });
}

#[test]
fn test_create_node_contract_which_was_canceled_before_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        let h = generate_deployment_hash();
        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            h,
            get_deployment_data(),
            0,
            None
        ));
        let contract_id = SmartContractModule::node_contract_by_hash(1, h);
        assert_eq!(contract_id, 1);

        assert_ok!(SmartContractModule::cancel_contract(
            Origin::signed(alice()),
            1
        ));

        let h = generate_deployment_hash();
        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            h,
            get_deployment_data(),
            0,
            None
        ));
        let contract_id = SmartContractModule::node_contract_by_hash(1, h);
        assert_eq!(contract_id, 2);
    });
}

#[test]
fn test_update_node_contract_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));

        let new_hash = generate_deployment_hash();
        let deployment_data = get_deployment_data();
        assert_ok!(SmartContractModule::update_node_contract(
            Origin::signed(alice()),
            1,
            new_hash,
            get_deployment_data()
        ));

        let node_contract = types::NodeContract {
            node_id: 1,
            deployment_hash: new_hash,
            deployment_data,
            public_ips: 0,
            public_ips_list: Vec::new().try_into().unwrap(),
        };
        let contract_type = types::ContractData::NodeContract(node_contract);

        let expected_contract_value = types::Contract {
            contract_id: 1,
            state: types::ContractState::Created,
            twin_id: 1,
            version: crate::CONTRACT_VERSION,
            contract_type,
            solution_provider_id: None,
        };

        let node_contract = SmartContractModule::contracts(1).unwrap();
        assert_eq!(node_contract, expected_contract_value);

        let contracts = SmartContractModule::active_node_contracts(1);
        assert_eq!(contracts.len(), 1);

        assert_eq!(contracts[0], 1);

        let node_contract_id_by_hash = SmartContractModule::node_contract_by_hash(1, new_hash);
        assert_eq!(node_contract_id_by_hash, 1);
    });
}

#[test]
fn test_update_node_contract_not_exists_fails() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_noop!(
            SmartContractModule::update_node_contract(
                Origin::signed(alice()),
                1,
                generate_deployment_hash(),
                get_deployment_data()
            ),
            Error::<TestRuntime>::ContractNotExists
        );
    });
}

#[test]
fn test_update_node_contract_wrong_twins_fails() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));

        assert_noop!(
            SmartContractModule::update_node_contract(
                Origin::signed(bob()),
                1,
                generate_deployment_hash(),
                get_deployment_data()
            ),
            Error::<TestRuntime>::TwinNotAuthorizedToUpdateContract
        );
    });
}

#[test]
fn test_cancel_node_contract_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));

        assert_ok!(SmartContractModule::cancel_contract(
            Origin::signed(alice()),
            1
        ));

        let node_contract = SmartContractModule::contracts(1);
        assert_eq!(node_contract, None);

        let contracts = SmartContractModule::active_node_contracts(1);
        assert_eq!(contracts.len(), 0);
    });
}

#[test]
fn test_create_multiple_node_contracts_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
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
fn test_cancel_node_contract_frees_public_ips_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            2,
            None
        ));

        let farm = TfgridModule::farms(1).unwrap();
        assert_eq!(farm.public_ips[0].contract_id, 1);
        assert_eq!(farm.public_ips[1].contract_id, 1);

        assert_ok!(SmartContractModule::cancel_contract(
            Origin::signed(alice()),
            1
        ));

        let farm = TfgridModule::farms(1).unwrap();
        assert_eq!(farm.public_ips[0].contract_id, 0);
        assert_eq!(farm.public_ips[1].contract_id, 0);
    });
}

#[test]
fn test_cancel_node_contract_not_exists_fails() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_noop!(
            SmartContractModule::cancel_contract(Origin::signed(alice()), 1),
            Error::<TestRuntime>::ContractNotExists
        );
    });
}

#[test]
fn test_cancel_node_contract_wrong_twins_fails() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));

        assert_noop!(
            SmartContractModule::cancel_contract(Origin::signed(bob()), 1),
            Error::<TestRuntime>::TwinNotAuthorizedToCancelContract
        );
    });
}

//  NAME CONTRACT TESTS //
// -------------------- //

#[test]
fn test_create_name_contract_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_name_contract(
            Origin::signed(bob()),
            "foobar".as_bytes().to_vec()
        ));
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
        assert_eq!(name_contract, None);

        let contract_id = SmartContractModule::contract_id_by_name_registration(
            get_name_contract_name(&"some_name".as_bytes().to_vec()),
        );
        assert_eq!(contract_id, 0);
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
fn test_create_name_contract_with_invalid_dns_name_fails() {
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

//  RENT CONTRACT TESTS //
// -------------------- //

#[test]
fn test_create_rent_contract_works() {
    new_test_ext().execute_with(|| {
        prepare_dedicated_farm_and_node();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(bob()),
            node_id,
            None
        ));

        let contract = SmartContractModule::contracts(1).unwrap();
        let rent_contract = types::RentContract { node_id };
        assert_eq!(
            contract.contract_type,
            types::ContractData::RentContract(rent_contract)
        );
    });
}

#[test]
fn test_cancel_rent_contract_works() {
    new_test_ext().execute_with(|| {
        prepare_dedicated_farm_and_node();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(bob()),
            node_id,
            None
        ));

        let contract = SmartContractModule::contracts(1).unwrap();
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
        assert_eq!(contract, None);
    });
}

#[test]
fn test_create_rent_contract_on_node_in_use_fails() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            1,
            None
        ));

        assert_noop!(
            SmartContractModule::create_rent_contract(Origin::signed(bob()), 1, None),
            Error::<TestRuntime>::NodeNotAvailableToDeploy
        );
    })
}

#[test]
fn test_create_rent_contract_non_dedicated_empty_node_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(bob()),
            node_id,
            None
        ));
    })
}

#[test]
fn test_create_node_contract_on_dedicated_node_without_rent_contract_fails() {
    new_test_ext().execute_with(|| {
        prepare_dedicated_farm_and_node();

        assert_noop!(
            SmartContractModule::create_node_contract(
                Origin::signed(bob()),
                1,
                generate_deployment_hash(),
                get_deployment_data(),
                1,
                None
            ),
            Error::<TestRuntime>::NodeNotAvailableToDeploy
        );
    })
}

#[test]
fn test_create_node_contract_when_having_a_rentcontract_works() {
    new_test_ext().execute_with(|| {
        prepare_dedicated_farm_and_node();

        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(bob()),
            1,
            None
        ));

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            1,
            None
        ));
    })
}

#[test]
fn test_create_node_contract_when_someone_else_has_rent_contract_fails() {
    new_test_ext().execute_with(|| {
        prepare_dedicated_farm_and_node();

        // create rent contract with bob
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(bob()),
            1,
            None
        ));

        // try to create node contract with Alice
        // Alice not the owner of the rent contract so she is unauthorized to deploy a node contract
        assert_noop!(
            SmartContractModule::create_node_contract(
                Origin::signed(alice()),
                1,
                generate_deployment_hash(),
                get_deployment_data(),
                1,
                None
            ),
            Error::<TestRuntime>::NodeHasRentContract
        );
    })
}

#[test]
fn test_cancel_rent_contract_with_active_node_contracts_fails() {
    new_test_ext().execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(bob()),
            node_id,
            None
        ));

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            1,
            None
        ));

        assert_noop!(
            SmartContractModule::cancel_contract(Origin::signed(bob()), 1,),
            Error::<TestRuntime>::NodeHasActiveContracts
        );
    });
}

//  CONTRACT BILLING TESTS //
// ----------------------- //

#[test]
fn test_node_contract_billing_details() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(0);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        let twin = TfgridModule::twins(2).unwrap();
        let initial_twin_balance = Balances::free_balance(&twin.account_id);

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            1,
            None
        ));

        push_contract_resources_used(1);

        push_nru_report_for_contract(1, 10);

        let contract_to_bill = SmartContractModule::contract_to_bill_at_block(10);
        assert_eq!(contract_to_bill, [1]);

        let initial_total_issuance = Balances::total_issuance();
        // advance 25 cycles
        let mut i = 0;
        while i != 24 {
            i += 1;
            run_to_block(i * 10 + 1);
        }

        let free_balance = Balances::free_balance(&twin.account_id);
        let total_amount_billed = initial_twin_balance - free_balance;
        println!("locked balance {:?}", total_amount_billed);

        println!("total locked balance {:?}", total_amount_billed);

        let staking_pool_account_balance = Balances::free_balance(&get_staking_pool_account());
        println!(
            "staking pool account balance, {:?}",
            staking_pool_account_balance
        );

        // 5% is sent to the staking pool account
        assert_eq!(
            staking_pool_account_balance,
            Perbill::from_percent(5) * total_amount_billed
        );

        // 10% is sent to the foundation account
        let pricing_policy = TfgridModule::pricing_policies(1).unwrap();
        let foundation_account_balance = Balances::free_balance(&pricing_policy.foundation_account);
        assert_eq!(
            foundation_account_balance,
            Perbill::from_percent(10) * total_amount_billed
        );

        // 50% is sent to the sales account
        let sales_account_balance = Balances::free_balance(&pricing_policy.certified_sales_account);
        assert_eq!(
            sales_account_balance,
            Perbill::from_percent(50) * total_amount_billed
        );

        let total_issuance = Balances::total_issuance();
        // total issueance is now previous total - amount burned from contract billed (35%)
        let burned_amount = Perbill::from_percent(35) * total_amount_billed;
        assert_eq!(
            total_issuance,
            initial_total_issuance - burned_amount as u64
        );

        // amount unbilled should have been reset after a transfer between contract owner and farmer
        let contract_billing_info = SmartContractModule::contract_billing_information_by_id(1);
        assert_eq!(contract_billing_info.amount_unbilled, 0);
    });
}

#[test]
fn test_node_contract_billing_details_with_solution_provider() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        prepare_solution_provider();

        run_to_block(0);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        let twin = TfgridModule::twins(2).unwrap();
        let initial_twin_balance = Balances::free_balance(&twin.account_id);
        let initial_total_issuance = Balances::total_issuance();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            1,
            Some(1)
        ));

        push_contract_resources_used(1);

        push_nru_report_for_contract(1, 10);

        let contract_to_bill = SmartContractModule::contract_to_bill_at_block(10);
        assert_eq!(contract_to_bill, [1]);

        // advance 25 cycles
        let mut i = 0;
        while i != 24 {
            i += 1;
            run_to_block(i * 10 + 1);
        }

        let usable_balance = Balances::usable_balance(&twin.account_id);
        let total_amount_billed = initial_twin_balance - usable_balance;

        validate_distribution_rewards(initial_total_issuance, total_amount_billed, true);

        // amount unbilled should have been reset after a transfer between contract owner and farmer
        let contract_billing_info = SmartContractModule::contract_billing_information_by_id(1);
        assert_eq!(contract_billing_info.amount_unbilled, 0);
    });
}

#[test]
fn test_multiple_contracts_billing_loop_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            1,
            None
        ));
        assert_ok!(SmartContractModule::create_name_contract(
            Origin::signed(bob()),
            "some_name".as_bytes().to_vec(),
        ));

        let contract_to_bill_at_block = SmartContractModule::contract_to_bill_at_block(11);
        assert_eq!(contract_to_bill_at_block.len(), 2);

        run_to_block(12);

        // Test that the expected events were emitted
        let our_events = System::events();

        // 1: Contract Created (node contract)
        // 2: Contract created (name contract)
        // 3: Contract Billed (node contract)
        // 4: Contract Billed (name contract)
        assert_eq!(our_events.len(), 6);
    })
}

#[test]
fn test_node_contract_billing_cycles() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let contract_id = 1;
        let twin_id = 2;

        push_contract_resources_used(1);

        let (amount_due_1, discount_received) = calculate_tft_cost(contract_id, twin_id, 11);
        run_to_block(12);
        check_report_cost(1, amount_due_1, 12, discount_received);

        let twin = TfgridModule::twins(twin_id).unwrap();
        let usable_balance = Balances::usable_balance(&twin.account_id);
        let free_balance = Balances::free_balance(&twin.account_id);

        let locked_balance = free_balance - usable_balance;
        assert_eq!(
            locked_balance.saturated_into::<u128>(),
            amount_due_1 as u128
        );

        let (amount_due_2, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(22);
        check_report_cost(1, amount_due_2, 22, discount_received);

        let (amount_due_3, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(32);
        check_report_cost(1, amount_due_3, 32, discount_received);

        let twin = TfgridModule::twins(twin_id).unwrap();
        let usable_balance = Balances::usable_balance(&twin.account_id);
        let free_balance = Balances::free_balance(&twin.account_id);

        let locked_balance = free_balance - usable_balance;
        assert_eq!(
            locked_balance.saturated_into::<u128>(),
            amount_due_1 as u128 + amount_due_2 as u128 + amount_due_3 as u128
        );
    });
}

#[test]
fn test_node_multiple_contract_billing_cycles() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let twin_id = 2;

        push_contract_resources_used(1);
        push_contract_resources_used(2);

        let (amount_due_contract_1, discount_received) = calculate_tft_cost(1, twin_id, 11);
        run_to_block(12);
        check_report_cost(1, amount_due_contract_1, 12, discount_received);

        let (amount_due_contract_2, discount_received) = calculate_tft_cost(2, twin_id, 11);
        run_to_block(12);
        check_report_cost(2, amount_due_contract_2, 12, discount_received);

        let twin = TfgridModule::twins(twin_id).unwrap();
        let usable_balance = Balances::usable_balance(&twin.account_id);
        let free_balance = Balances::free_balance(&twin.account_id);

        let locked_balance = free_balance - usable_balance;
        assert_eq!(
            locked_balance.saturated_into::<u128>(),
            amount_due_contract_1 as u128 + amount_due_contract_2 as u128
        );
    });
}

#[test]
fn test_node_contract_billing_cycles_delete_node_cancels_contract() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            1,
            None
        ));
        let contract_id = 1;
        let twin_id = 2;

        push_contract_resources_used(1);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 11);
        run_to_block(12);
        check_report_cost(1, amount_due_as_u128, 12, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(22);
        check_report_cost(1, amount_due_as_u128, 22, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(32);
        check_report_cost(1, amount_due_as_u128, 32, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(42);
        check_report_cost(1, amount_due_as_u128, 42, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(52);
        check_report_cost(1, amount_due_as_u128, 52, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 4);
        run_to_block(56);

        // Delete node
        TfgridModule::delete_node_farm(Origin::signed(alice()), 1).unwrap();

        // After deleting a node, the contract gets billed before it's canceled
        check_report_cost(1, amount_due_as_u128, 56, discount_received);

        let our_events = System::events();

        for e in our_events.clone().iter() {
            println!("{:?}", e);
        }

        let public_ip = PublicIP {
            ip: "185.206.122.33/24".as_bytes().to_vec().try_into().unwrap(),
            gateway: "185.206.122.1".as_bytes().to_vec().try_into().unwrap(),
            contract_id: 0,
        };

        let mut ips: BoundedVec<
            PublicIP<TestPublicIP, TestGatewayIP>,
            crate::MaxNodeContractPublicIPs<TestRuntime>,
        > = vec![].try_into().unwrap();
        ips.try_push(public_ip).unwrap();

        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::IPsFreed {
                    contract_id: 1,
                    public_ips: ips
                }
            ))),
            true
        );
        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::NodeContractCanceled {
                    contract_id: 1,
                    node_id: 1,
                    twin_id: 2
                }
            ))),
            true
        );
    });
}

#[test]
fn test_node_contract_only_public_ip_billing_cycles() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            1,
            None
        ));
        let contract_id = 1;
        let twin_id = 2;

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 11);
        assert_ne!(amount_due_as_u128, 0);
        run_to_block(12);
        check_report_cost(1, amount_due_as_u128, 12, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(22);
        check_report_cost(1, amount_due_as_u128, 22, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(32);
        check_report_cost(1, amount_due_as_u128, 32, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(42);
        check_report_cost(1, amount_due_as_u128, 42, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(52);
        check_report_cost(1, amount_due_as_u128, 52, discount_received);
    });
}

#[test]
fn test_node_contract_billing_cycles_cancel_contract_during_cycle_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));

        let contract_id = 1;
        let twin_id = 2;

        push_contract_resources_used(1);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 11);
        run_to_block(12);
        check_report_cost(1, amount_due_as_u128, 12, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(22);
        check_report_cost(1, amount_due_as_u128, 22, discount_received);

        run_to_block(28);
        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 6);
        assert_ok!(SmartContractModule::cancel_contract(
            Origin::signed(bob()),
            1
        ));

        run_to_block(29);
        check_report_cost(1, amount_due_as_u128, 28, discount_received);

        let contract = SmartContractModule::contracts(1);
        assert_eq!(contract, None);

        let billing_info = SmartContractModule::contract_billing_information_by_id(1);
        assert_eq!(billing_info.amount_unbilled, 0);
    });
}

#[test]
fn test_node_contract_billing_cycles_cancel_contract_during_cycle_without_balance_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        let twin = TfgridModule::twins(2).unwrap();
        let initial_twin_balance = Balances::free_balance(&twin.account_id);
        println!("initial twin balance: {:?}", initial_twin_balance);
        let initial_total_issuance = Balances::total_issuance();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));

        let contract_id = 1;
        let twin_id = 2;

        push_contract_resources_used(1);

        let (amount_due_1, discount_received) = calculate_tft_cost(contract_id, twin_id, 11);
        run_to_block(12);
        check_report_cost(1, amount_due_1, 12, discount_received);

        let (amount_due_2, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(22);
        check_report_cost(1, amount_due_2, 22, discount_received);

        // Run halfway ish next cycle and cancel
        run_to_block(25);

        let usable_balance = Balances::usable_balance(&twin.account_id);
        let total_amount_billed = initial_twin_balance - usable_balance;

        let extrinsic_fee = 10000;
        Balances::transfer(
            Origin::signed(bob()),
            alice(),
            initial_twin_balance - total_amount_billed - extrinsic_fee,
        )
        .unwrap();

        let usable_balance_before_canceling = Balances::usable_balance(&twin.account_id);
        assert_ne!(usable_balance_before_canceling, 0);

        assert_ok!(SmartContractModule::cancel_contract(
            Origin::signed(bob()),
            1
        ));

        run_to_block(29);

        // After canceling contract, and not being able to pay for the remainder of the cycle
        // where the cancel was excecuted, the remaining balance should still be the same
        let usable_balance_after_canceling = Balances::usable_balance(&twin.account_id);
        assert_eq!(
            usable_balance_after_canceling,
            usable_balance_before_canceling
        );

        validate_distribution_rewards(initial_total_issuance, total_amount_billed, false);
    });
}

#[test]
fn test_node_contract_out_of_funds_should_move_state_to_graceperiod_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(charlie()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));

        push_contract_resources_used(1);

        // cycle 1
        run_to_block(12);

        // cycle 2
        // user does not have enough funds to pay for 2 cycles
        run_to_block(22);

        let c1 = SmartContractModule::contracts(1).unwrap();
        assert_eq!(c1.state, types::ContractState::GracePeriod(21));

        let our_events = System::events();
        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::ContractGracePeriodStarted {
                    contract_id: 1,
                    node_id: 1,
                    twin_id: 3,
                    block_number: 21
                }
            ))),
            true
        );
    });
}

#[test]
fn test_restore_node_contract_in_grace_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(charlie()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));

        push_contract_resources_used(1);

        // cycle 1
        run_to_block(12);

        // cycle 2
        // user does not have enough funds to pay for 2 cycles
        run_to_block(22);

        let c1 = SmartContractModule::contracts(1).unwrap();
        assert_eq!(c1.state, types::ContractState::GracePeriod(21));

        let our_events = System::events();
        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::ContractGracePeriodStarted {
                    contract_id: 1,
                    node_id: 1,
                    twin_id: 3,
                    block_number: 21
                }
            ))),
            true
        );

        let contract_to_bill = SmartContractModule::contract_to_bill_at_block(31);
        assert_eq!(contract_to_bill.len(), 1);
        run_to_block(32);

        let contract_to_bill = SmartContractModule::contract_to_bill_at_block(41);
        assert_eq!(contract_to_bill.len(), 1);
        run_to_block(42);

        // Transfer some balance to the owner of the contract to trigger the grace period to stop
        Balances::transfer(Origin::signed(bob()), charlie(), 100000000).unwrap();

        let contract_to_bill = SmartContractModule::contract_to_bill_at_block(51);
        assert_eq!(contract_to_bill.len(), 1);
        run_to_block(52);

        let contract_to_bill = SmartContractModule::contract_to_bill_at_block(61);
        assert_eq!(contract_to_bill.len(), 1);
        run_to_block(62);

        let c1 = SmartContractModule::contracts(1).unwrap();
        assert_eq!(c1.state, types::ContractState::Created);
    });
}

#[test]
fn test_node_contract_grace_period_cancels_contract_when_grace_period_ends_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();
        let initial_total_issuance = Balances::total_issuance();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(charlie()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));

        push_contract_resources_used(1);

        // cycle 1
        run_to_block(12);

        // cycle 2
        // user does not have enough funds to pay for 2 cycles
        run_to_block(22);

        let c1 = SmartContractModule::contracts(1).unwrap();
        assert_eq!(c1.state, types::ContractState::GracePeriod(21));

        let our_events = System::events();
        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::ContractGracePeriodStarted {
                    contract_id: 1,
                    node_id: 1,
                    twin_id: 3,
                    block_number: 21
                }
            ))),
            true
        );

        let twin = TfgridModule::twins(3).unwrap();
        let free_balance = Balances::free_balance(&twin.account_id);

        run_to_block(32);
        run_to_block(42);
        run_to_block(52);
        run_to_block(62);
        run_to_block(72);
        run_to_block(82);
        run_to_block(92);
        run_to_block(102);
        run_to_block(112);
        run_to_block(122);
        run_to_block(132);

        // The user's total free balance should be distributed
        validate_distribution_rewards(initial_total_issuance, free_balance, false);

        let c1 = SmartContractModule::contracts(1);
        assert_eq!(c1, None);
    });
}

#[test]
fn test_name_contract_billing() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_name_contract(
            Origin::signed(bob()),
            "foobar".as_bytes().to_vec()
        ));

        let contract_to_bill = SmartContractModule::contract_to_bill_at_block(11);
        assert_eq!(contract_to_bill, [1]);

        // let mature 11 blocks
        // because we bill every 10 blocks
        run_to_block(12);

        let contract_bill_event = types::ContractBill {
            contract_id: 1,
            timestamp: 1628082072,
            discount_level: types::DiscountLevel::Gold,
            amount_billed: 2032,
        };

        let our_events = System::events();
        println!("events: {:?}", our_events.clone());
        assert_eq!(
            our_events[3],
            record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::ContractBilled(
                contract_bill_event
            )))
        );
    });
}

#[test]
fn test_rent_contract_billing() {
    new_test_ext().execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(bob()),
            node_id,
            None
        ));

        let contract = SmartContractModule::contracts(1).unwrap();
        let rent_contract = types::RentContract { node_id };
        assert_eq!(
            contract.contract_type,
            types::ContractData::RentContract(rent_contract)
        );

        run_to_block(12);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(1, 2, 11);
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(1, amount_due_as_u128, 12, discount_received);
    });
}

#[test]
fn test_rent_contract_billing_cancel_should_bill_reserved_balance() {
    new_test_ext().execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(bob()),
            node_id,
            None
        ));

        let contract = SmartContractModule::contracts(1).unwrap();
        let rent_contract = types::RentContract { node_id };
        assert_eq!(
            contract.contract_type,
            types::ContractData::RentContract(rent_contract)
        );

        run_to_block(12);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(1, 2, 11);
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(1, amount_due_as_u128, 12, discount_received.clone());

        let twin = TfgridModule::twins(2).unwrap();
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

        let twin = TfgridModule::twins(2).unwrap();
        let usable_balance = Balances::usable_balance(&twin.account_id);
        assert_ne!(usable_balance, 0);
        Balances::transfer(Origin::signed(bob()), alice(), usable_balance).unwrap();

        run_to_block(22);

        // Last amount due is the same as the first one
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(1, amount_due_as_u128, 14, discount_received);

        let usable_balance = Balances::usable_balance(&twin.account_id);
        let free_balance = Balances::free_balance(&twin.account_id);
        assert_eq!(usable_balance, free_balance);
    });
}

#[test]
fn test_rent_contract_canceled_mid_cycle_should_bill_for_remainder() {
    new_test_ext().execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(bob()),
            node_id,
            None
        ));

        let contract = SmartContractModule::contracts(1).unwrap();
        let rent_contract = types::RentContract { node_id };
        assert_eq!(
            contract.contract_type,
            types::ContractData::RentContract(rent_contract)
        );

        let twin = TfgridModule::twins(2).unwrap();
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
        check_report_cost(1, amount_due_as_u128, 8, discount_received.clone());

        // Twin should have no more locked balance
        let twin = TfgridModule::twins(2).unwrap();
        let usable_balance = Balances::usable_balance(&twin.account_id);
        let free_balance = Balances::free_balance(&twin.account_id);
        assert_eq!(usable_balance, free_balance);
    });
}

#[test]
fn test_create_rent_contract_and_node_contract_excludes_node_contract_from_billing_works() {
    new_test_ext().execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(bob()),
            node_id,
            None
        ));

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        push_contract_resources_used(2);

        run_to_block(12);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(1, 2, 11);
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(1, amount_due_as_u128, 12, discount_received);

        let our_events = System::events();
        // Event 1: Rent contract created
        // Event 2: Node Contract created
        // Event 4: Rent contract billed
        // => no Node Contract billed event
        assert_eq!(our_events.len(), 6);
    });
}

#[test]
fn test_rent_contract_canceled_due_to_out_of_funds_should_cancel_node_contracts_works() {
    new_test_ext().execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(charlie()),
            node_id,
            None
        ));

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(charlie()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        push_contract_resources_used(2);

        run_to_block(12);
        run_to_block(22);
        run_to_block(32);
        run_to_block(42);
        run_to_block(52);
        run_to_block(62);
        run_to_block(72);
        run_to_block(82);
        run_to_block(92);
        run_to_block(102);
        run_to_block(112);
        run_to_block(122);

        // let (amount_due_as_u128, discount_received) = calculate_tft_cost(1, 2, 11);
        // assert_ne!(amount_due_as_u128, 0);
        // check_report_cost(1, 3, amount_due_as_u128, 12, discount_received);

        let our_events = System::events();
        assert_eq!(our_events.len(), 31);

        assert_eq!(
            our_events[5],
            record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::ContractGracePeriodStarted {
                contract_id: 1,
                node_id: 1,
                twin_id: 3,
                block_number: 11
            }))
        );
        assert_eq!(
            our_events[6],
            record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::ContractGracePeriodStarted {
                contract_id: 2,
                node_id: 1,
                twin_id: 3,
                block_number: 11
            }))
        );

        assert_eq!(
            our_events[29],
            record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::NodeContractCanceled {
                contract_id: 2,
                node_id: 1,
                twin_id: 3
            }))
        );
        assert_eq!(
            our_events[30],
            record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::RentContractCanceled {
                contract_id: 1
            }))
        );
    });
}

#[test]
fn test_create_rent_contract_and_node_contract_with_ip_billing_works() {
    new_test_ext().execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(bob()),
            node_id,
            None
        ));

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            1,
            None
        ));

        run_to_block(12);

        // check contract 1 costs (Rent Contract)
        let (amount_due_as_u128, discount_received) = calculate_tft_cost(1, 2, 11);
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(1, amount_due_as_u128, 12, discount_received);
        // check contract 2 costs (Node Contract)
        let (amount_due_as_u128, discount_received) = calculate_tft_cost(2, 2, 11);
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(2, amount_due_as_u128, 12, discount_received);

        let our_events = System::events();
        // Event 1: Price Stored
        // Event 2: Avg price stored
        // Event 2: Rent contract created
        // Event 3: Node Contract created
        // Event 4: Rent contract billed
        // Event 5: Node Contract billed
        assert_eq!(our_events.len(), 6);
    });
}

#[test]
fn test_rent_contract_out_of_funds_should_move_state_to_graceperiod_works() {
    new_test_ext().execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(charlie()),
            node_id,
            None
        ));

        // cycle 1
        // user does not have enough funds to pay for 1 cycle
        run_to_block(12);

        let c1 = SmartContractModule::contracts(1).unwrap();
        assert_eq!(c1.state, types::ContractState::GracePeriod(11));

        let our_events = System::events();
        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::ContractGracePeriodStarted {
                    contract_id: 1,
                    node_id: 1,
                    twin_id: 3,
                    block_number: 11
                }
            ))),
            true
        );
    });
}

#[test]
fn test_restore_rent_contract_in_grace_works() {
    new_test_ext().execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(charlie()),
            node_id,
            None
        ));

        // cycle 1
        run_to_block(12);

        let c1 = SmartContractModule::contracts(1).unwrap();
        assert_eq!(c1.state, types::ContractState::GracePeriod(11));

        let our_events = System::events();
        assert_eq!(
            our_events[3],
            record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::ContractGracePeriodStarted {
                contract_id: 1,
                node_id: 1,
                twin_id: 3,
                block_number: 11
            }))
        );

        let contract_to_bill = SmartContractModule::contract_to_bill_at_block(21);
        assert_eq!(contract_to_bill.len(), 1);
        run_to_block(22);

        let contract_to_bill = SmartContractModule::contract_to_bill_at_block(31);
        assert_eq!(contract_to_bill.len(), 1);
        run_to_block(32);

        // Transfer some balance to the owner of the contract to trigger the grace period to stop
        Balances::transfer(Origin::signed(bob()), charlie(), 100000000).unwrap();

        let contract_to_bill = SmartContractModule::contract_to_bill_at_block(41);
        assert_eq!(contract_to_bill.len(), 1);
        run_to_block(42);

        let contract_to_bill = SmartContractModule::contract_to_bill_at_block(51);
        assert_eq!(contract_to_bill.len(), 1);
        run_to_block(52);

        let c1 = SmartContractModule::contracts(1).unwrap();
        assert_eq!(c1.state, types::ContractState::Created);
    });
}

#[test]
fn test_restore_rent_contract_and_node_contracts_in_grace_works() {
    new_test_ext().execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(charlie()),
            node_id,
            None
        ));
        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(charlie()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        push_contract_resources_used(2);

        // cycle 1
        run_to_block(12);

        let c1 = SmartContractModule::contracts(1).unwrap();
        assert_eq!(c1.state, types::ContractState::GracePeriod(11));

        let our_events = System::events();
        assert_eq!(
            our_events[5],
            record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::ContractGracePeriodStarted {
                contract_id: 1,
                node_id: 1,
                twin_id: 3,
                block_number: 11
            }))
        );
        assert_eq!(
            our_events[6],
            record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::ContractGracePeriodStarted {
                contract_id: 2,
                node_id: 1,
                twin_id: 3,
                block_number: 11
            }))
        );

        let contract_to_bill = SmartContractModule::contract_to_bill_at_block(21);
        assert_eq!(contract_to_bill.len(), 2);
        run_to_block(22);

        let contract_to_bill = SmartContractModule::contract_to_bill_at_block(31);
        assert_eq!(contract_to_bill.len(), 2);
        run_to_block(32);

        // Transfer some balance to the owner of the contract to trigger the grace period to stop
        Balances::transfer(Origin::signed(bob()), charlie(), 100000000).unwrap();

        let contract_to_bill = SmartContractModule::contract_to_bill_at_block(41);
        assert_eq!(contract_to_bill.len(), 2);
        run_to_block(42);

        let contract_to_bill = SmartContractModule::contract_to_bill_at_block(51);
        assert_eq!(contract_to_bill.len(), 2);
        run_to_block(52);

        let c1 = SmartContractModule::contracts(1).unwrap();
        assert_eq!(c1.state, types::ContractState::Created);

        let our_events = System::events();
        assert_eq!(
            our_events[11],
            record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::ContractGracePeriodEnded {
                contract_id: 1,
                node_id: 1,
                twin_id: 3,
            }))
        );
        assert_eq!(
            our_events[12],
            record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::ContractGracePeriodEnded {
                contract_id: 2,
                node_id: 1,
                twin_id: 3,
            }))
        );
    });
}

#[test]
fn test_rent_contract_grace_period_cancels_contract_when_grace_period_ends_works() {
    new_test_ext().execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(charlie()),
            node_id,
            None
        ));

        // cycle 1
        run_to_block(12);

        let c1 = SmartContractModule::contracts(1).unwrap();
        assert_eq!(c1.state, types::ContractState::GracePeriod(11));

        let our_events = System::events();
        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::ContractGracePeriodStarted {
                    contract_id: 1,
                    node_id: 1,
                    twin_id: 3,
                    block_number: 11
                }
            ))),
            true
        );

        run_to_block(22);
        run_to_block(32);
        run_to_block(42);
        run_to_block(52);
        run_to_block(62);
        run_to_block(72);
        run_to_block(82);
        run_to_block(92);
        run_to_block(102);
        run_to_block(112);
        run_to_block(122);
        run_to_block(132);

        let c1 = SmartContractModule::contracts(1);
        assert_eq!(c1, None);
    });
}

#[test]
fn test_rent_contract_and_node_contract_canceled_when_node_is_deleted_works() {
    new_test_ext().execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1);
        TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            Origin::signed(bob()),
            node_id,
            None
        ));

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        push_contract_resources_used(2);

        run_to_block(12);

        run_to_block(16);

        // Delete node
        TfgridModule::delete_node_farm(Origin::signed(alice()), 1).unwrap();

        let our_events = System::events();

        let ip = "1.1.1.0".as_bytes().to_vec();
        let mut ips = Vec::new();
        ips.push(ip);

        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::NodeContractCanceled {
                    contract_id: 2,
                    node_id: 1,
                    twin_id: 2
                }
            ))),
            true
        );
        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::RentContractCanceled { contract_id: 1 }
            ))),
            true
        );
    });
}

//  SOLUTION PROVIDER TESTS //
// ------------------------ //
#[test]
fn test_create_solution_provider_works() {
    new_test_ext().execute_with(|| {
        let provider1 = super::types::Provider {
            take: 10,
            who: alice(),
        };
        let provider2 = super::types::Provider {
            take: 10,
            who: bob(),
        };
        let providers = vec![provider1, provider2];

        assert_ok!(SmartContractModule::create_solution_provider(
            Origin::signed(alice()),
            "some_description".as_bytes().to_vec(),
            "some_link".as_bytes().to_vec(),
            providers
        ));

        assert_ok!(SmartContractModule::approve_solution_provider(
            RawOrigin::Root.into(),
            1,
            true
        ));
    })
}

#[test]
fn test_create_solution_provider_fails_if_take_to_high() {
    new_test_ext().execute_with(|| {
        let provider = super::types::Provider {
            take: 51,
            who: alice(),
        };
        let providers = vec![provider];

        assert_noop!(
            SmartContractModule::create_solution_provider(
                Origin::signed(alice()),
                "some_description".as_bytes().to_vec(),
                "some_link".as_bytes().to_vec(),
                providers
            ),
            Error::<TestRuntime>::InvalidProviderConfiguration
        );
    })
}

#[test]
fn test_create_node_contract_with_solution_provider_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        prepare_solution_provider();

        assert_ok!(SmartContractModule::create_node_contract(
            Origin::signed(alice()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            Some(1)
        ));
    });
}

#[test]
fn test_create_node_contract_with_solution_provider_fails_if_not_approved() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        let provider = super::types::Provider {
            take: 10,
            who: alice(),
        };
        let providers = vec![provider];

        assert_ok!(SmartContractModule::create_solution_provider(
            Origin::signed(alice()),
            "some_description".as_bytes().to_vec(),
            "some_link".as_bytes().to_vec(),
            providers
        ));

        assert_noop!(
            SmartContractModule::create_node_contract(
                Origin::signed(alice()),
                1,
                generate_deployment_hash(),
                get_deployment_data(),
                0,
                Some(1)
            ),
            Error::<TestRuntime>::SolutionProviderNotApproved
        );
    });
}

//  MODULE FUNCTION TESTS //
// ---------------------- //

#[test]
fn test_cu_calculation() {
    new_test_ext().execute_with(|| {
        let cu = U64F64::from_num(4);
        let mru = U64F64::from_num(1024);
        let cu = cost::calculate_cu(cu, mru);
        assert_eq!(cu, 128);

        let cu = U64F64::from_num(32);
        let mru = U64F64::from_num(128);
        let cu = cost::calculate_cu(cu, mru);
        assert_eq!(cu, 32);

        let cu = U64F64::from_num(4);
        let mru = U64F64::from_num(2);
        let cu = cost::calculate_cu(cu, mru);
        assert_eq!(cu, 1);

        let cu = U64F64::from_num(4);
        let mru = U64F64::from_num(1);
        let cu = cost::calculate_cu(cu, mru);
        assert_eq!(cu, 1);

        let cu = U64F64::from_num(16);
        let mru = U64F64::from_num(16);
        let cu = cost::calculate_cu(cu, mru);
        assert_eq!(cu, 8);
    })
}

#[test]
fn test_lock() {
    new_test_ext().execute_with(|| {
        let id: u64 = 1;
        Balances::set_lock(id.to_be_bytes(), &bob(), 100, WithdrawReasons::all());

        let usable_balance = Balances::usable_balance(&bob());
        let free_balance = Balances::free_balance(&bob());

        let locked_balance = free_balance - usable_balance;
        assert_eq!(locked_balance, 100);

        Balances::extend_lock(id.to_be_bytes(), &bob(), 200, WithdrawReasons::all());
        let usable_balance = Balances::usable_balance(&bob());
        let free_balance = Balances::free_balance(&bob());

        let locked_balance = free_balance - usable_balance;
        assert_eq!(locked_balance, 200);
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

fn validate_distribution_rewards(
    initial_total_issuance: u64,
    total_amount_billed: u64,
    had_solution_provider: bool,
) {
    println!("total locked balance {:?}", total_amount_billed);

    let staking_pool_account_balance = Balances::free_balance(&get_staking_pool_account());
    println!(
        "staking pool account balance, {:?}",
        staking_pool_account_balance
    );

    // 5% is sent to the staking pool account
    assert_eq!(
        staking_pool_account_balance,
        Perbill::from_percent(5) * total_amount_billed
    );

    // 10% is sent to the foundation account
    let pricing_policy = TfgridModule::pricing_policies(1).unwrap();
    let foundation_account_balance = Balances::free_balance(&pricing_policy.foundation_account);
    assert_eq!(
        foundation_account_balance,
        Perbill::from_percent(10) * total_amount_billed
    );

    if had_solution_provider {
        // 40% is sent to the sales account
        let sales_account_balance = Balances::free_balance(&pricing_policy.certified_sales_account);
        assert_eq!(
            sales_account_balance,
            Perbill::from_percent(40) * total_amount_billed
        );

        // 10% is sent to the solution provider
        let solution_provider = SmartContractModule::solution_providers(1).unwrap();
        let solution_provider_1_balance =
            Balances::free_balance(solution_provider.providers[0].who.clone());
        println!("solution provider b: {:?}", solution_provider_1_balance);
        assert_eq!(
            solution_provider_1_balance,
            Perbill::from_percent(10) * total_amount_billed
        );
    } else {
        // 50% is sent to the sales account
        let sales_account_balance = Balances::free_balance(&pricing_policy.certified_sales_account);
        assert_eq!(
            sales_account_balance,
            Perbill::from_percent(50) * total_amount_billed
        );
    }

    let total_issuance = Balances::total_issuance();
    // total issueance is now previous total - amount burned from contract billed (35%)
    let burned_amount = Perbill::from_percent(35) * total_amount_billed;
    assert_eq_error_rate!(
        total_issuance,
        initial_total_issuance - burned_amount as u64,
        1
    );
}

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
        used: Resources {
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
    amount_billed: u64,
    block_number: u64,
    discount_level: types::DiscountLevel,
) {
    let our_events = System::events();

    let contract_bill_event = types::ContractBill {
        contract_id,
        timestamp: 1628082000 + (6 * block_number),
        discount_level,
        amount_billed: amount_billed as u128,
    };

    assert_eq!(
        our_events.contains(&record(MockEvent::SmartContractModule(
            SmartContractEvent::<TestRuntime>::ContractBilled(contract_bill_event)
        ))),
        true
    );
    // assert_eq!(
    //     our_events[index],
    //     record(MockEvent::SmartContractModule(SmartContractEvent::<
    //         TestRuntime,
    //     >::ContractBilled(
    //         contract_bill_event
    //     )))
    // );
}

fn calculate_tft_cost(contract_id: u64, twin_id: u32, blocks: u64) -> (u64, types::DiscountLevel) {
    let twin = TfgridModule::twins(twin_id).unwrap();
    let b = Balances::free_balance(&twin.account_id);
    let contract = SmartContractModule::contracts(contract_id).unwrap();
    let (amount_due, discount_received) =
        contract.calculate_contract_cost_tft(b, blocks * 6).unwrap();

    (amount_due, discount_received)
}

pub fn prepare_twins() {
    create_twin(alice());
    create_twin(bob());
    create_twin(charlie());
}

pub fn prepare_farm(source: AccountId, dedicated: bool) {
    let farm_name = "test_farm";
    let mut pub_ips = Vec::new();
    pub_ips.push(pallet_tfgrid_types::PublicIpInput {
        ip: "185.206.122.33/24".as_bytes().to_vec().try_into().unwrap(),
        gw: "185.206.122.1".as_bytes().to_vec().try_into().unwrap(),
    });
    pub_ips.push(pallet_tfgrid_types::PublicIpInput {
        ip: "185.206.122.34/24".as_bytes().to_vec().try_into().unwrap(),
        gw: "185.206.122.1".as_bytes().to_vec().try_into().unwrap(),
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
        farm_name.as_bytes().to_vec().try_into().unwrap(),
        pub_ips.clone().try_into().unwrap(),
    )
    .unwrap();

    if !dedicated {
        return;
    }

    TfgridModule::set_farm_dedicated(RawOrigin::Root.into(), 1, true).unwrap();
}

pub fn prepare_farm_and_node() {
    TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();

    create_farming_policies();

    prepare_twins();

    prepare_farm(alice(), false);

    // random location
    let location = Location {
        longitude: "12.233213231".as_bytes().to_vec(),
        latitude: "32.323112123".as_bytes().to_vec(),
    };

    let resources = Resources {
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
        bounded_vec![],
        false,
        false,
        "some_serial".as_bytes().to_vec(),
    )
    .unwrap();
}

pub fn prepare_dedicated_farm_and_node() {
    TFTPriceModule::set_prices(Origin::signed(bob()), 50, 101).unwrap();
    create_farming_policies();

    prepare_twins();

    prepare_farm(alice(), true);

    // random location
    let location = Location {
        longitude: "12.233213231".as_bytes().to_vec(),
        latitude: "32.323112123".as_bytes().to_vec(),
    };

    let resources = Resources {
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
        bounded_vec![],
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
    let ip = get_twin_ip(b"::1");
    assert_ok!(TfgridModule::create_twin(
        Origin::signed(origin),
        ip.clone().0
    ));
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

fn create_farming_policies() {
    let name = "f1".as_bytes().to_vec();
    assert_ok!(TfgridModule::create_farming_policy(
        RawOrigin::Root.into(),
        name,
        12,
        15,
        10,
        8,
        9999,
        System::block_number() + 100,
        true,
        true,
        NodeCertification::Diy,
        FarmCertification::Gold,
    ));

    let name = "f2".as_bytes().to_vec();
    assert_ok!(TfgridModule::create_farming_policy(
        RawOrigin::Root.into(),
        name,
        12,
        15,
        10,
        8,
        9999,
        System::block_number() + 100,
        true,
        true,
        NodeCertification::Diy,
        FarmCertification::NotCertified,
    ));

    let name = "f3".as_bytes().to_vec();
    assert_ok!(TfgridModule::create_farming_policy(
        RawOrigin::Root.into(),
        name,
        12,
        15,
        10,
        8,
        9999,
        System::block_number() + 100,
        true,
        true,
        NodeCertification::Certified,
        FarmCertification::Gold,
    ));

    let name = "f1".as_bytes().to_vec();
    assert_ok!(TfgridModule::create_farming_policy(
        RawOrigin::Root.into(),
        name,
        12,
        15,
        10,
        8,
        9999,
        System::block_number() + 100,
        true,
        true,
        NodeCertification::Certified,
        FarmCertification::NotCertified,
    ));
}

fn prepare_solution_provider() {
    let provider = super::types::Provider {
        take: 10,
        who: dave(),
    };
    let providers = vec![provider];

    assert_ok!(SmartContractModule::create_solution_provider(
        Origin::signed(dave()),
        "some_description".as_bytes().to_vec(),
        "some_link".as_bytes().to_vec(),
        providers
    ));

    assert_ok!(SmartContractModule::approve_solution_provider(
        RawOrigin::Root.into(),
        1,
        true
    ));
}

fn record(event: Event) -> EventRecord<Event, H256> {
    EventRecord {
        phase: Phase::Initialization,
        event,
        topics: vec![],
    }
}

fn generate_deployment_hash() -> H256 {
    H256::random()
}

fn get_deployment_data() -> crate::DeploymentDataInput<TestRuntime> {
    BoundedVec::<u8, crate::MaxDeploymentDataLength<TestRuntime>>::try_from(
        "some_data".as_bytes().to_vec(),
    )
    .unwrap()
}
