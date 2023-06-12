use super::{types, Event as SmartContractEvent};
use crate::cost;
use crate::types::HexHash;
use crate::{mock::RuntimeEvent as MockEvent, mock::*, test_utils::*, Error};
use frame_support::{
    assert_noop, assert_ok, bounded_vec,
    dispatch::Pays,
    traits::{LockableCurrency, WithdrawReasons},
    BoundedVec,
};
use frame_system::{EventRecord, Phase, RawOrigin};
use log::info;
use pallet_tfgrid::{
    types::{self as pallet_tfgrid_types, LocationInput},
    ResourcesInput,
};
use sp_core::H256;
use sp_runtime::{assert_eq_error_rate, traits::SaturatedConversion, Perbill, Percent};
use sp_std::convert::{TryFrom, TryInto};
use substrate_fixed::types::U64F64;
use tfchain_support::constants::time::{SECS_PER_BLOCK, SECS_PER_HOUR};
use tfchain_support::{
    resources::Resources,
    types::{FarmCertification, NodeCertification, PublicIP, IP4},
};

const GIGABYTE: u64 = 1024 * 1024 * 1024;
const BASE_FEE: u64 = 1000;
const VARIABLE_FEE: u64 = 1000;
const VARIABLE_AMOUNT: u64 = 100;

//  NODE CONTRACT TESTS //
// -------------------- //

#[test]
fn test_create_node_contract_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(alice()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let contract_id = 1;

        // Ensure contract_id is stored at right billing loop index
        let index = SmartContractModule::get_contract_billing_loop_index(contract_id);
        assert_eq!(
            SmartContractModule::contract_to_bill_at_block(index),
            vec![contract_id]
        );
    });
}

#[test]
fn test_create_node_contract_on_offline_node_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        assert_ok!(TfgridModule::change_power_state(
            RuntimeOrigin::signed(alice()),
            tfchain_support::types::Power::Down
        ));

        assert_noop!(
            SmartContractModule::create_node_contract(
                RuntimeOrigin::signed(alice()),
                node_id,
                generate_deployment_hash(),
                get_deployment_data(),
                0,
                None
            ),
            Error::<TestRuntime>::NodeNotAvailableToDeploy
        );
    });
}

#[test]
fn test_create_node_contract_with_public_ips_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(alice()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            2,
            None
        ));
        let contract_id = 1;

        let node_contract = SmartContractModule::contracts(contract_id).unwrap();

        match node_contract.contract_type.clone() {
            types::ContractData::NodeContract(c) => {
                let farm = TfgridModule::farms(1).unwrap();
                assert_eq!(farm.public_ips[0].contract_id, 1);

                assert_eq!(c.public_ips, 2);

                let pub_ip = PublicIP {
                    ip: get_public_ip_ip_input(b"185.206.122.33/24"),
                    gateway: get_public_ip_gw_input(b"185.206.122.1"),
                    contract_id,
                };

                let pub_ip_2 = PublicIP {
                    ip: get_public_ip_ip_input(b"185.206.122.34/24"),
                    gateway: get_public_ip_gw_input(b"185.206.122.1"),
                    contract_id,
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
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        assert_noop!(
            SmartContractModule::create_node_contract(
                RuntimeOrigin::signed(alice()),
                node_id + 1,
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
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        let h = generate_deployment_hash();
        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(alice()),
            node_id,
            h,
            get_deployment_data(),
            0,
            None
        ));

        assert_noop!(
            SmartContractModule::create_node_contract(
                RuntimeOrigin::signed(alice()),
                node_id,
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
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        let h = generate_deployment_hash();
        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(alice()),
            node_id,
            h,
            get_deployment_data(),
            0,
            None
        ));
        let contract_id = SmartContractModule::node_contract_by_hash(node_id, h);
        assert_eq!(contract_id, 1);

        assert_ok!(SmartContractModule::cancel_contract(
            RuntimeOrigin::signed(alice()),
            contract_id
        ));

        let h = generate_deployment_hash();
        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(alice()),
            node_id,
            h,
            get_deployment_data(),
            0,
            None
        ));
        let contract_id = SmartContractModule::node_contract_by_hash(node_id, h);
        assert_eq!(contract_id, 2);
    });
}

#[test]
fn test_update_node_contract_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(alice()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let contract_id = 1;

        let new_hash = generate_deployment_hash();
        let deployment_data = get_deployment_data();
        assert_ok!(SmartContractModule::update_node_contract(
            RuntimeOrigin::signed(alice()),
            contract_id,
            new_hash,
            get_deployment_data()
        ));

        let node_contract = types::NodeContract {
            node_id,
            deployment_hash: new_hash,
            deployment_data,
            public_ips: 0,
            public_ips_list: bounded_vec![],
        };
        let contract_type = types::ContractData::NodeContract(node_contract);

        let expected_contract_value = types::Contract {
            contract_id,
            state: types::ContractState::Created,
            twin_id: 1,
            version: crate::CONTRACT_VERSION,
            contract_type,
            solution_provider_id: None,
        };

        let node_contract = SmartContractModule::contracts(contract_id).unwrap();
        assert_eq!(node_contract, expected_contract_value);

        let contracts = SmartContractModule::active_node_contracts(1);
        assert_eq!(contracts.len(), 1);

        assert_eq!(contracts[0], contract_id);

        let node_contract_id_by_hash =
            SmartContractModule::node_contract_by_hash(node_id, new_hash);
        assert_eq!(node_contract_id_by_hash, contract_id);
    });
}

#[test]
fn test_update_node_contract_not_exists_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        assert_noop!(
            SmartContractModule::update_node_contract(
                RuntimeOrigin::signed(alice()),
                node_id,
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
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(alice()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let contract_id = 1;

        assert_noop!(
            SmartContractModule::update_node_contract(
                RuntimeOrigin::signed(bob()),
                contract_id,
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
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(alice()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let contract_id = 1;

        assert_ok!(SmartContractModule::cancel_contract(
            RuntimeOrigin::signed(alice()),
            contract_id
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
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(alice()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let contract_id = 1;

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(alice()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(alice()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));

        let node_contracts = SmartContractModule::active_node_contracts(node_id);
        assert_eq!(node_contracts.len(), 3);

        // now cancel 1 and check if the storage maps are updated correctly
        assert_ok!(SmartContractModule::cancel_contract(
            RuntimeOrigin::signed(alice()),
            contract_id
        ));

        let node_contracts = SmartContractModule::active_node_contracts(node_id);
        assert_eq!(node_contracts.len(), 2);
    });
}

#[test]
fn test_cancel_node_contract_frees_public_ips_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let farm_id = 1;
        let node_id = 1;

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(alice()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            2,
            None
        ));
        let contract_id = 1;

        let farm = TfgridModule::farms(farm_id).unwrap();
        assert_eq!(farm.public_ips[0].contract_id, contract_id);
        assert_eq!(farm.public_ips[1].contract_id, contract_id);

        assert_ok!(SmartContractModule::cancel_contract(
            RuntimeOrigin::signed(alice()),
            contract_id
        ));

        let farm = TfgridModule::farms(farm_id).unwrap();
        assert_eq!(farm.public_ips[0].contract_id, 0);
        assert_eq!(farm.public_ips[1].contract_id, 0);
    });
}

#[test]
fn test_cancel_node_contract_not_exists_fails() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        let contract_id = 1;
        assert_noop!(
            SmartContractModule::cancel_contract(RuntimeOrigin::signed(alice()), contract_id),
            Error::<TestRuntime>::ContractNotExists
        );
    });
}

#[test]
fn test_cancel_node_contract_wrong_twins_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(alice()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let contract_id = 1;

        assert_noop!(
            SmartContractModule::cancel_contract(RuntimeOrigin::signed(bob()), contract_id),
            Error::<TestRuntime>::TwinNotAuthorizedToCancelContract
        );
    });
}

#[test]
fn test_cancel_node_contract_and_remove_from_billing_loop_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(alice()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let contract_id = 1;

        run_to_block(6, None);

        // Ensure contract_id is stored at right billing loop index
        let index = SmartContractModule::get_contract_billing_loop_index(contract_id);
        assert_eq!(
            SmartContractModule::contract_to_bill_at_block(index),
            vec![contract_id]
        );

        // Canceling contract will remove it from contract storage, try
        // to bill remaining amount due and also remove it from billing
        assert_ok!(SmartContractModule::cancel_contract(
            RuntimeOrigin::signed(alice()),
            contract_id
        ));

        // Ensure contract has been removed from billing loop
        assert_eq!(
            SmartContractModule::contract_to_bill_at_block(index),
            Vec::<u64>::new()
        );
    });
}

#[test]
fn test_remove_from_billing_loop_wrong_index_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(alice()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let contract_id = 1;

        // Ensure contract_id is stored at right billing loop index
        let index = SmartContractModule::get_contract_billing_loop_index(contract_id);
        assert_eq!(
            SmartContractModule::contract_to_bill_at_block(index),
            vec![contract_id]
        );

        assert_noop!(
            SmartContractModule::remove_contract_from_billing_loop(2),
            Error::<TestRuntime>::ContractWrongBillingLoopIndex
        );

        assert_noop!(
            SmartContractModule::remove_contract_from_billing_loop(11),
            Error::<TestRuntime>::ContractWrongBillingLoopIndex
        );
    });
}

//  NAME CONTRACT TESTS //
// -------------------- //

#[test]
fn test_create_name_contract_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_name_contract(
            RuntimeOrigin::signed(bob()),
            b"foobar".to_vec()
        ));
    });
}

#[test]
fn test_cancel_name_contract_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_name_contract(
            RuntimeOrigin::signed(alice()),
            b"some_name".to_vec()
        ));
        let contract_id = 1;

        assert_ok!(SmartContractModule::cancel_contract(
            RuntimeOrigin::signed(alice()),
            contract_id
        ));

        let name_contract = SmartContractModule::contracts(contract_id);
        assert_eq!(name_contract, None);

        let contract_id = SmartContractModule::contract_id_by_name_registration(
            get_name_contract_name(&b"some_name".to_vec()),
        );
        assert_eq!(contract_id, 0);
    });
}

#[test]
fn test_create_name_contract_double_with_same_name_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_name_contract(
            RuntimeOrigin::signed(bob()),
            b"foobar".to_vec()
        ));
        assert_noop!(
            SmartContractModule::create_name_contract(
                RuntimeOrigin::signed(alice()),
                b"foobar".to_vec()
            ),
            Error::<TestRuntime>::NameExists
        );
    });
}

#[test]
fn test_recreate_name_contract_after_cancel_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::create_name_contract(
            RuntimeOrigin::signed(bob()),
            b"foobar".to_vec()
        ));
        let contract_id = 1;

        assert_ok!(SmartContractModule::cancel_contract(
            RuntimeOrigin::signed(bob()),
            contract_id
        ));

        assert_ok!(SmartContractModule::create_name_contract(
            RuntimeOrigin::signed(bob()),
            b"foobar".to_vec()
        ));
    });
}

#[test]
fn test_create_name_contract_with_invalid_dns_name_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_noop!(
            SmartContractModule::create_name_contract(
                RuntimeOrigin::signed(alice()),
                b"foo.bar".to_vec()
            ),
            Error::<TestRuntime>::NameNotValid
        );

        assert_noop!(
            SmartContractModule::create_name_contract(
                RuntimeOrigin::signed(alice()),
                b"foo!".to_vec()
            ),
            Error::<TestRuntime>::NameNotValid
        );

        assert_noop!(
            SmartContractModule::create_name_contract(
                RuntimeOrigin::signed(alice()),
                b"foo;'".to_vec()
            ),
            Error::<TestRuntime>::NameNotValid
        );

        assert_noop!(
            SmartContractModule::create_name_contract(
                RuntimeOrigin::signed(alice()),
                b"foo123.%".to_vec()
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
        run_to_block(1, None);
        prepare_dedicated_farm_and_node();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            RuntimeOrigin::signed(bob()),
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
fn test_create_rent_contract_on_offline_node_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_dedicated_farm_and_node();
        let node_id = 1;

        assert_ok!(TfgridModule::change_power_state(
            RuntimeOrigin::signed(alice()),
            tfchain_support::types::Power::Down
        ));

        assert_noop!(
            SmartContractModule::create_rent_contract(RuntimeOrigin::signed(bob()), node_id, None),
            Error::<TestRuntime>::NodeNotAvailableToDeploy
        );
    });
}

#[test]
fn test_cancel_rent_contract_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_dedicated_farm_and_node();
        let node_id = 1;

        assert_ok!(SmartContractModule::create_rent_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            None
        ));
        let contract_id = 1;

        let contract = SmartContractModule::contracts(1).unwrap();
        let rent_contract = types::RentContract { node_id };
        assert_eq!(
            contract.contract_type,
            types::ContractData::RentContract(rent_contract)
        );

        assert_ok!(SmartContractModule::cancel_contract(
            RuntimeOrigin::signed(bob()),
            contract_id
        ));

        let contract = SmartContractModule::contracts(contract_id);
        assert_eq!(contract, None);
    });
}

#[test]
fn test_create_rent_contract_on_node_in_use_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(alice()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            1,
            None
        ));

        assert_noop!(
            SmartContractModule::create_rent_contract(RuntimeOrigin::signed(bob()), node_id, None),
            Error::<TestRuntime>::NodeNotAvailableToDeploy
        );
    })
}

#[test]
fn test_create_rent_contract_non_dedicated_empty_node_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        assert_ok!(SmartContractModule::create_rent_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            None
        ));
    })
}

#[test]
fn test_create_node_contract_on_dedicated_node_without_rent_contract_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_dedicated_farm_and_node();
        let node_id = 1;

        assert_noop!(
            SmartContractModule::create_node_contract(
                RuntimeOrigin::signed(bob()),
                node_id,
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
        run_to_block(1, None);
        prepare_dedicated_farm_and_node();
        let node_id = 1;

        assert_ok!(SmartContractModule::create_rent_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            None
        ));

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
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
        run_to_block(1, None);
        prepare_dedicated_farm_and_node();
        let node_id = 1;

        // create rent contract with bob
        assert_ok!(SmartContractModule::create_rent_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            None
        ));

        // try to create node contract with Alice
        // Alice not the owner of the rent contract so she is unauthorized to deploy a node contract
        assert_noop!(
            SmartContractModule::create_node_contract(
                RuntimeOrigin::signed(alice()),
                node_id,
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
fn test_cancel_rent_contract_with_active_node_contracts_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_dedicated_farm_and_node();
        let node_id = 1;

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_rent_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            None
        ));
        let contract_id = 1;

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            1,
            None
        ));

        assert_noop!(
            SmartContractModule::cancel_contract(RuntimeOrigin::signed(bob()), contract_id),
            Error::<TestRuntime>::NodeHasActiveContracts
        );
    });
}

//  CONTRACT BILLING TESTS //
// ----------------------- //

#[test]
fn test_node_contract_billing_details() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        let twin = TfgridModule::twins(2).unwrap();
        let initial_twin_balance = Balances::free_balance(&twin.account_id);

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            1,
            None
        ));
        let contract_id = 1;

        push_contract_resources_used(contract_id);
        push_nru_report_for_contract(contract_id, 10);

        // Ensure contract_id is stored at right billing loop index
        let index = SmartContractModule::get_contract_billing_loop_index(contract_id);
        assert_eq!(
            SmartContractModule::contract_to_bill_at_block(index),
            vec![contract_id]
        );

        let initial_total_issuance = Balances::total_issuance();
        // advance 25 cycles
        for i in 0..25 {
            let block_number = 11 + i * 10;
            pool_state.write().should_call_bill_contract(
                contract_id,
                Ok(Pays::Yes.into()),
                block_number,
            );
            run_to_block(block_number, Some(&mut pool_state));
        }

        let free_balance = Balances::free_balance(&twin.account_id);
        let total_amount_billed = initial_twin_balance - free_balance;
        info!("locked balance {:?}", total_amount_billed);

        info!("total locked balance {:?}", total_amount_billed);

        let staking_pool_account_balance = Balances::free_balance(&get_staking_pool_account());
        info!(
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
        assert_eq_error_rate!(
            total_issuance,
            initial_total_issuance - burned_amount as u64,
            1
        );

        // amount unbilled should have been reset after a transfer between contract owner and farmer
        let contract_billing_info =
            SmartContractModule::contract_billing_information_by_id(contract_id);
        assert_eq!(contract_billing_info.amount_unbilled, 0);
    });
}

#[test]
fn test_node_contract_billing_details_with_solution_provider() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        prepare_solution_provider(dave());

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        let twin = TfgridModule::twins(2).unwrap();
        let initial_twin_balance = Balances::free_balance(&twin.account_id);
        let initial_total_issuance = Balances::total_issuance();

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            1,
            Some(1)
        ));
        let contract_id = 1;

        push_contract_resources_used(contract_id);
        push_nru_report_for_contract(contract_id, 10);

        // Ensure contract_id is stored at right billing loop index
        let index = SmartContractModule::get_contract_billing_loop_index(contract_id);
        assert_eq!(
            SmartContractModule::contract_to_bill_at_block(index),
            vec![contract_id]
        );

        // advance 25 cycles
        for i in 0..25 {
            let block_number = 11 + i * 10;
            pool_state.write().should_call_bill_contract(
                contract_id,
                Ok(Pays::Yes.into()),
                block_number,
            );
            run_to_block(block_number, Some(&mut pool_state));
        }

        let free_balance = Balances::free_balance(&twin.account_id);
        let total_amount_billed = initial_twin_balance - free_balance;

        validate_distribution_rewards(initial_total_issuance, total_amount_billed, true);

        // amount unbilled should have been reset after a transfer between contract owner and farmer
        let contract_billing_info =
            SmartContractModule::contract_billing_information_by_id(contract_id);
        assert_eq!(contract_billing_info.amount_unbilled, 0);
    });
}

#[test]
fn test_multiple_contracts_billing_loop_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_farm_and_node();
        let node_id = 1;
        run_to_block(1, None);

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            1,
            None
        ));
        let node_contract_id = 1;

        run_to_block(2, None);

        assert_ok!(SmartContractModule::create_name_contract(
            RuntimeOrigin::signed(bob()),
            "some_name".as_bytes().to_vec(),
        ));
        let name_contract_id = 2;

        // Ensure node_contract_id is stored at right billing loop index
        let index = SmartContractModule::get_contract_billing_loop_index(node_contract_id);
        assert_eq!(
            SmartContractModule::contract_to_bill_at_block(index),
            vec![node_contract_id]
        );

        // Ensure name_contract_id is stored at right billing loop index
        let index = SmartContractModule::get_contract_billing_loop_index(name_contract_id);
        assert_eq!(
            SmartContractModule::contract_to_bill_at_block(index),
            vec![name_contract_id]
        );

        // 2 contracts => 2 billings
        pool_state
            .write()
            .should_call_bill_contract(node_contract_id, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));
        pool_state
            .write()
            .should_call_bill_contract(name_contract_id, Ok(Pays::Yes.into()), 12);
        run_to_block(12, Some(&mut pool_state));

        // Test that the expected events were emitted
        let our_events = System::events();

        // 1: Contract Created (node contract)
        // 2: Contract created (name contract)
        // 3: Contract Billed (node contract)
        // 4: Contract Billed (name contract)
        assert_eq!(our_events.len(), 8);
    })
}

#[test]
fn test_node_contract_billing_cycles() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let contract_id = 1;
        let twin_id = 2;

        push_contract_resources_used(contract_id);

        let (amount_due_1, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        pool_state
            .write()
            .should_call_bill_contract(contract_id, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));
        check_report_cost(1, amount_due_1, 11, discount_received);

        let twin = TfgridModule::twins(twin_id).unwrap();
        let usable_balance = Balances::usable_balance(&twin.account_id);
        let free_balance = Balances::free_balance(&twin.account_id);

        let locked_balance = free_balance - usable_balance;
        assert_eq!(
            locked_balance.saturated_into::<u128>(),
            amount_due_1 as u128
        );

        let (amount_due_2, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        pool_state
            .write()
            .should_call_bill_contract(contract_id, Ok(Pays::Yes.into()), 21);
        run_to_block(21, Some(&mut pool_state));
        check_report_cost(1, amount_due_2, 21, discount_received);

        let (amount_due_3, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        pool_state
            .write()
            .should_call_bill_contract(contract_id, Ok(Pays::Yes.into()), 31);
        run_to_block(31, Some(&mut pool_state));
        check_report_cost(1, amount_due_3, 31, discount_received);

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
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let first_node_contract_id = 1;
        push_contract_resources_used(first_node_contract_id);

        run_to_block(2, None);

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let second_node_contract_id = 2;
        let twin_id = 2;
        push_contract_resources_used(second_node_contract_id);

        pool_state.write().should_call_bill_contract(
            first_node_contract_id,
            Ok(Pays::Yes.into()),
            11,
        );

        let (amount_due_contract_1, discount_received) =
            calculate_tft_cost(first_node_contract_id, twin_id, 10);
        run_to_block(11, Some(&mut pool_state));
        check_report_cost(
            first_node_contract_id,
            amount_due_contract_1,
            11,
            discount_received,
        );

        pool_state.write().should_call_bill_contract(
            second_node_contract_id,
            Ok(Pays::Yes.into()),
            12,
        );

        let (amount_due_contract_2, discount_received) =
            calculate_tft_cost(second_node_contract_id, twin_id, 10);
        run_to_block(12, Some(&mut pool_state));
        check_report_cost(
            second_node_contract_id,
            amount_due_contract_2,
            12,
            discount_received,
        );

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
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            1,
            None
        ));
        let contract_id = 1;
        let twin_id = 2;

        for i in 0..5 {
            let block_number = 11 + i * 10;
            pool_state.write().should_call_bill_contract(
                contract_id,
                Ok(Pays::Yes.into()),
                block_number,
            );
        }
        push_contract_resources_used(contract_id);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(11, Some(&mut pool_state));
        check_report_cost(1, amount_due_as_u128, 11, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(21, Some(&mut pool_state));
        check_report_cost(1, amount_due_as_u128, 21, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(31, Some(&mut pool_state));
        check_report_cost(1, amount_due_as_u128, 31, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(41, Some(&mut pool_state));
        check_report_cost(1, amount_due_as_u128, 41, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(51, Some(&mut pool_state));
        check_report_cost(1, amount_due_as_u128, 51, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 4);
        run_to_block(55, None);

        // Delete node
        TfgridModule::delete_node_farm(RuntimeOrigin::signed(alice()), 1).unwrap();

        // After deleting a node, the contract gets billed before it's canceled
        check_report_cost(1, amount_due_as_u128, 55, discount_received);

        let our_events = System::events();

        for e in our_events.clone().iter() {
            info!("{:?}", e);
        }

        let public_ip = PublicIP {
            ip: get_public_ip_ip_input(b"185.206.122.33/24"),
            gateway: get_public_ip_gw_input(b"185.206.122.1"),
            contract_id: 0,
        };

        let mut ips: BoundedVec<PublicIP, crate::MaxNodeContractPublicIPs<TestRuntime>> =
            vec![].try_into().unwrap();
        ips.try_push(public_ip).unwrap();

        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::IPsFreed {
                    contract_id,
                    public_ips: ips
                }
            ))),
            true
        );
        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::NodeContractCanceled {
                    contract_id,
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
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            1,
            None
        ));
        let contract_id = 1;
        let twin_id = 2;

        for i in 0..5 {
            let block_number = 11 + i * 10;
            pool_state.write().should_call_bill_contract(
                contract_id,
                Ok(Pays::Yes.into()),
                block_number,
            );
        }

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        assert_ne!(amount_due_as_u128, 0);
        run_to_block(11, Some(&mut pool_state));
        check_report_cost(1, amount_due_as_u128, 11, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(21, Some(&mut pool_state));
        check_report_cost(1, amount_due_as_u128, 21, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(31, Some(&mut pool_state));
        check_report_cost(1, amount_due_as_u128, 31, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(41, Some(&mut pool_state));
        check_report_cost(1, amount_due_as_u128, 41, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(51, Some(&mut pool_state));
        check_report_cost(1, amount_due_as_u128, 51, discount_received);
    });
}

#[test]
fn test_node_contract_billing_cycles_cancel_contract_during_cycle_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let contract_id = 1;
        let twin_id = 2;

        // 2 cycles for billing
        for i in 0..2 {
            let block_number = 11 + i * 10;
            pool_state.write().should_call_bill_contract(
                contract_id,
                Ok(Pays::Yes.into()),
                block_number,
            );
        }
        push_contract_resources_used(contract_id);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(11, Some(&mut pool_state));
        check_report_cost(1, amount_due_as_u128, 11, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(21, Some(&mut pool_state));
        check_report_cost(1, amount_due_as_u128, 21, discount_received);

        run_to_block(28, Some(&mut pool_state));
        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 7);
        assert_ok!(SmartContractModule::cancel_contract(
            RuntimeOrigin::signed(bob()),
            1
        ));

        run_to_block(29, Some(&mut pool_state));
        check_report_cost(1, amount_due_as_u128, 28, discount_received);

        let contract = SmartContractModule::contracts(1);
        assert_eq!(contract, None);

        let billing_info = SmartContractModule::contract_billing_information_by_id(contract_id);
        assert_eq!(billing_info.amount_unbilled, 0);
    });
}

#[test]
fn test_node_contract_billing_cycles_cancel_contract_during_cycle_without_balance_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        let twin = TfgridModule::twins(2).unwrap();
        let initial_twin_balance = Balances::free_balance(&twin.account_id);
        info!("initial twin balance: {:?}", initial_twin_balance);
        let initial_total_issuance = Balances::total_issuance();

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let contract_id = 1;
        let twin_id = 2;
        push_contract_resources_used(contract_id);

        let (amount_due_1, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        pool_state
            .write()
            .should_call_bill_contract(contract_id, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));
        check_report_cost(1, amount_due_1, 11, discount_received);

        let (amount_due_2, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        pool_state
            .write()
            .should_call_bill_contract(contract_id, Ok(Pays::Yes.into()), 21);
        run_to_block(21, Some(&mut pool_state));
        check_report_cost(1, amount_due_2, 21, discount_received);

        // Run halfway ish next cycle and cancel
        run_to_block(25, Some(&mut pool_state));

        let usable_balance = Balances::usable_balance(&twin.account_id);
        let total_amount_billed = initial_twin_balance - usable_balance;

        let extrinsic_fee = 10000;
        Balances::transfer(
            RuntimeOrigin::signed(bob()),
            alice(),
            initial_twin_balance - total_amount_billed - extrinsic_fee,
        )
        .unwrap();

        let usable_balance_before_canceling = Balances::usable_balance(&twin.account_id);
        assert_ne!(usable_balance_before_canceling, 0);

        assert_ok!(SmartContractModule::cancel_contract(
            RuntimeOrigin::signed(bob()),
            contract_id
        ));

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
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(charlie()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let contract_id = 1;

        push_contract_resources_used(contract_id);

        // cycle 1
        pool_state
            .write()
            .should_call_bill_contract(contract_id, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

        // cycle 2
        // user does not have enough funds to pay for 2 cycles
        pool_state
            .write()
            .should_call_bill_contract(contract_id, Ok(Pays::Yes.into()), 21);
        run_to_block(21, Some(&mut pool_state));

        let c1 = SmartContractModule::contracts(1).unwrap();
        assert_eq!(c1.state, types::ContractState::GracePeriod(21));

        let our_events = System::events();
        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::ContractGracePeriodStarted {
                    contract_id,
                    node_id,
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
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(charlie()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let contract_id = 1;

        for i in 0..6 {
            let block_number = 11 + i * 10;
            pool_state.write().should_call_bill_contract(
                contract_id,
                Ok(Pays::Yes.into()),
                block_number,
            );
        }
        push_contract_resources_used(contract_id);

        // cycle 1
        run_to_block(11, Some(&mut pool_state));

        // cycle 2
        // user does not have enough funds to pay for 2 cycles
        run_to_block(21, Some(&mut pool_state));

        let c1 = SmartContractModule::contracts(1).unwrap();
        assert_eq!(c1.state, types::ContractState::GracePeriod(21));

        let our_events = System::events();
        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::ContractGracePeriodStarted {
                    contract_id,
                    node_id,
                    twin_id: 3,
                    block_number: 21
                }
            ))),
            true
        );

        run_to_block(31, Some(&mut pool_state));
        run_to_block(41, Some(&mut pool_state));
        // Transfer some balance to the owner of the contract to trigger the grace period to stop
        Balances::transfer(RuntimeOrigin::signed(bob()), charlie(), 100000000).unwrap();
        run_to_block(52, Some(&mut pool_state));
        run_to_block(62, Some(&mut pool_state));

        let c1 = SmartContractModule::contracts(contract_id).unwrap();
        assert_eq!(c1.state, types::ContractState::Created);
    });
}

#[test]
fn test_node_contract_grace_period_cancels_contract_when_grace_period_ends_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        let twin = TfgridModule::twins(3).unwrap();
        let initial_total_issuance = Balances::total_issuance();
        let initial_twin_balance = Balances::free_balance(&twin.account_id);

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(charlie()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let contract_id = 1;

        push_contract_resources_used(contract_id);

        // cycle 1
        pool_state
            .write()
            .should_call_bill_contract(contract_id, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

        // cycle 2
        // user does not have enough funds to pay for 2 cycles
        pool_state
            .write()
            .should_call_bill_contract(contract_id, Ok(Pays::Yes.into()), 21);
        run_to_block(21, Some(&mut pool_state));

        let c1 = SmartContractModule::contracts(1).unwrap();
        assert_eq!(c1.state, types::ContractState::GracePeriod(21));

        let our_events = System::events();
        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::ContractGracePeriodStarted {
                    contract_id,
                    node_id,
                    twin_id: 3,
                    block_number: 21
                }
            ))),
            true
        );

        // grace period stops after 100 blocknumbers, so after 121
        for i in 1..11 {
            let block_number = 21 + i * 10;
            pool_state.write().should_call_bill_contract(
                contract_id,
                Ok(Pays::Yes.into()),
                block_number,
            );
        }

        for i in 1..11 {
            run_to_block(21 + i * 10, Some(&mut pool_state));
        }

        // The user's total free balance should be distributed
        let free_balance = Balances::free_balance(&twin.account_id);
        let total_amount_billed = initial_twin_balance - free_balance;

        validate_distribution_rewards(initial_total_issuance, total_amount_billed, false);

        let c1 = SmartContractModule::contracts(contract_id);
        assert_eq!(c1, None);
    });
}

#[test]
fn test_name_contract_billing() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1, None);

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_name_contract(
            RuntimeOrigin::signed(bob()),
            b"foobar".to_vec()
        ));
        let contract_id = 1;

        // Ensure contract_id is stored at right billing loop index
        let index = SmartContractModule::get_contract_billing_loop_index(contract_id);
        assert_eq!(
            SmartContractModule::contract_to_bill_at_block(index),
            vec![contract_id]
        );

        // let mature 11 blocks
        // because we bill every 10 blocks
        pool_state
            .write()
            .should_call_bill_contract(contract_id, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

        // get the contract cost for 1 billing cycle
        let contract = SmartContractModule::contracts(contract_id).unwrap();
        let twin_id = 2;
        let twin = TfgridModule::twins(twin_id).unwrap();
        let balance = Balances::free_balance(&twin.account_id);
        let second_elapsed = BillingFrequency::get() * SECS_PER_BLOCK;
        let (contract_cost, _) = contract
            .calculate_contract_cost_tft(balance, second_elapsed)
            .unwrap();

        // the contractbill event should look like:
        let contract_bill_event = types::ContractBill {
            contract_id,
            timestamp: 1628082066,
            discount_level: types::DiscountLevel::Gold,
            amount_billed: contract_cost as u128,
        };

        let our_events = System::events();
        info!("events: {:?}", our_events.clone());
        assert_eq!(
            our_events[4],
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
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, None);
        prepare_dedicated_farm_and_node();
        let node_id = 1;

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_rent_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            None
        ));
        let contract_id = 1;

        let contract = SmartContractModule::contracts(contract_id).unwrap();
        let rent_contract = types::RentContract { node_id };
        assert_eq!(
            contract.contract_type,
            types::ContractData::RentContract(rent_contract)
        );

        pool_state
            .write()
            .should_call_bill_contract(contract_id, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(1, 2, 10);
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(1, amount_due_as_u128, 11, discount_received);
    });
}

#[test]
fn test_rent_contract_billing_cancel_should_bill_reserved_balance() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, None);
        prepare_dedicated_farm_and_node();
        let node_id = 1;

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_rent_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            None
        ));
        let contract_id = 1;

        let contract = SmartContractModule::contracts(contract_id).unwrap();
        let rent_contract = types::RentContract { node_id };
        assert_eq!(
            contract.contract_type,
            types::ContractData::RentContract(rent_contract)
        );

        pool_state
            .write()
            .should_call_bill_contract(contract_id, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, 2, 10);
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(
            contract_id,
            amount_due_as_u128,
            11,
            discount_received.clone(),
        );

        let twin = TfgridModule::twins(2).unwrap();
        let usable_balance = Balances::usable_balance(&twin.account_id);
        let free_balance = Balances::free_balance(&twin.account_id);
        assert_ne!(usable_balance, free_balance);

        run_to_block(13, Some(&mut pool_state));
        // cancel contract
        // it will bill before removing the contract and it should bill all
        // reserved balance
        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, 2, 2);
        assert_ok!(SmartContractModule::cancel_contract(
            RuntimeOrigin::signed(bob()),
            contract_id
        ));

        let twin = TfgridModule::twins(2).unwrap();
        let usable_balance = Balances::usable_balance(&twin.account_id);
        assert_ne!(usable_balance, 0);
        Balances::transfer(RuntimeOrigin::signed(bob()), alice(), usable_balance).unwrap();

        // Last amount due is the same as the first one
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(contract_id, amount_due_as_u128, 13, discount_received);

        let usable_balance = Balances::usable_balance(&twin.account_id);
        let free_balance = Balances::free_balance(&twin.account_id);
        assert_eq!(usable_balance, free_balance);
    });
}

#[test]
fn test_rent_contract_canceled_mid_cycle_should_bill_for_remainder() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, None);
        prepare_dedicated_farm_and_node();
        let node_id = 1;

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_rent_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            None
        ));
        let contract_id = 1;

        let contract = SmartContractModule::contracts(contract_id).unwrap();
        let rent_contract = types::RentContract { node_id };
        assert_eq!(
            contract.contract_type,
            types::ContractData::RentContract(rent_contract)
        );

        let twin = TfgridModule::twins(2).unwrap();
        let usable_balance = Balances::usable_balance(&twin.account_id);
        let free_balance = Balances::free_balance(&twin.account_id);

        let locked_balance = free_balance - usable_balance;
        info!("locked balance: {:?}", locked_balance);

        run_to_block(8, Some(&mut pool_state));
        // Calculate the cost for 7 blocks of runtime (created a block 1, canceled at block 8)
        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, 2, 7);
        // cancel rent contract at block 8
        assert_ok!(SmartContractModule::cancel_contract(
            RuntimeOrigin::signed(bob()),
            contract_id
        ));
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(
            contract_id,
            amount_due_as_u128,
            8,
            discount_received.clone(),
        );

        // Twin should have no more locked balance
        let twin = TfgridModule::twins(2).unwrap();
        let usable_balance = Balances::usable_balance(&twin.account_id);
        let free_balance = Balances::free_balance(&twin.account_id);
        assert_eq!(usable_balance, free_balance);
    });
}

#[test]
fn test_create_rent_contract_and_node_contract_excludes_node_contract_from_billing_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_dedicated_farm_and_node();
        let node_id = 1;
        run_to_block(1, None);

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_rent_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            None
        ));
        let rent_contract_id = 1;

        run_to_block(2, None);

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let node_contract_id = 2;
        push_contract_resources_used(node_contract_id);

        pool_state
            .write()
            .should_call_bill_contract(rent_contract_id, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));
        pool_state
            .write()
            .should_call_bill_contract(node_contract_id, Ok(Pays::Yes.into()), 12);
        run_to_block(12, Some(&mut pool_state));

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(rent_contract_id, 2, 10);
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(rent_contract_id, amount_due_as_u128, 11, discount_received);

        let our_events = System::events();
        // Event 1: Price Stored
        // Event 2: Avg price stored
        // Event 3: Rent contract created
        // Event 4: Node Contract created
        // Event 5: Updated resources contract
        // Event 6: Balances locked
        // Event 7: Contract Billed
        // => no Node Contract billed event
        assert_eq!(our_events.len(), 7);
    });
}

#[test]
fn test_rent_contract_canceled_due_to_out_of_funds_should_cancel_node_contracts_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_dedicated_farm_and_node();
        let node_id = 1;
        run_to_block(1, None);

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_rent_contract(
            RuntimeOrigin::signed(charlie()),
            node_id,
            None
        ));
        let rent_contract_id = 1;

        run_to_block(2, None);

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(charlie()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let node_contract_id = 2;
        push_contract_resources_used(node_contract_id);

        // run 10 cycles
        for i in 0..10 {
            let block_number = 11 + i * 10;
            pool_state.write().should_call_bill_contract(
                rent_contract_id,
                Ok(Pays::Yes.into()),
                block_number,
            );
            run_to_block(block_number, Some(&mut pool_state));
            pool_state.write().should_call_bill_contract(
                node_contract_id,
                Ok(Pays::Yes.into()),
                block_number + 1,
            );
            run_to_block(block_number + 1, Some(&mut pool_state));
        }

        // contracts should cancel at 11th due to lack of funds
        let end_grace_block_number = 11 + 10 * 10;
        pool_state.write().should_call_bill_contract(
            rent_contract_id,
            Ok(Pays::Yes.into()),
            end_grace_block_number,
        );
        run_to_block(end_grace_block_number, Some(&mut pool_state));

        let our_events = System::events();
        assert_eq!(our_events.len(), 21);

        for e in our_events.clone() {
            log::info!("event: {:?}", e);
        }

        assert_eq!(
            our_events[5],
            record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::ContractGracePeriodStarted {
                contract_id: rent_contract_id,
                node_id,
                twin_id: 3,
                block_number: 11
            }))
        );
        assert_eq!(
            our_events[6],
            record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::ContractGracePeriodStarted {
                contract_id: node_contract_id,
                node_id,
                twin_id: 3,
                block_number: 11
            }))
        );

        assert_eq!(
            our_events[19],
            record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::NodeContractCanceled {
                contract_id: node_contract_id,
                node_id,
                twin_id: 3
            }))
        );
        assert_eq!(
            our_events[20],
            record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::RentContractCanceled {
                contract_id: rent_contract_id
            }))
        );
    });
}

#[test]
fn test_create_rent_contract_and_node_contract_with_ip_billing_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::create_rent_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            None
        ));
        let rent_contract_id = 1;

        run_to_block(2, None);

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            1,
            None
        ));
        let node_contract_id = 2;

        // 2 contracts => we expect 2 calls to bill_contract
        pool_state
            .write()
            .should_call_bill_contract(rent_contract_id, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

        // check contract 1 costs (Rent Contract)
        let (amount_due_as_u128, discount_received) = calculate_tft_cost(rent_contract_id, 2, 10);
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(rent_contract_id, amount_due_as_u128, 11, discount_received);

        pool_state
            .write()
            .should_call_bill_contract(node_contract_id, Ok(Pays::Yes.into()), 12);
        run_to_block(12, Some(&mut pool_state));

        // check contract 2 costs (Node Contract)
        let (amount_due_as_u128, discount_received) = calculate_tft_cost(node_contract_id, 2, 10);
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(node_contract_id, amount_due_as_u128, 12, discount_received);

        let our_events = System::events();
        assert_eq!(our_events.len(), 8);
    });
}

#[test]
fn test_rent_contract_out_of_funds_should_move_state_to_graceperiod_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, None);
        prepare_dedicated_farm_and_node();
        let node_id = 1;

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_rent_contract(
            RuntimeOrigin::signed(charlie()),
            node_id,
            None
        ));
        let contract_id = 1;

        // cycle 1
        // user does not have enough funds to pay for 1 cycle
        pool_state
            .write()
            .should_call_bill_contract(contract_id, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

        let c1 = SmartContractModule::contracts(1).unwrap();
        assert_eq!(c1.state, types::ContractState::GracePeriod(11));

        let our_events = System::events();
        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::ContractGracePeriodStarted {
                    contract_id,
                    node_id,
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
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_dedicated_farm_and_node();
        let node_id = 1;
        run_to_block(1, None);

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_rent_contract(
            RuntimeOrigin::signed(charlie()),
            node_id,
            None
        ));
        let contract_id = 1;

        // cycle 1
        pool_state
            .write()
            .should_call_bill_contract(contract_id, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

        let c1 = SmartContractModule::contracts(contract_id).unwrap();
        assert_eq!(c1.state, types::ContractState::GracePeriod(11));

        let our_events = System::events();
        info!("events: {:?}", our_events.clone());
        assert_eq!(
            our_events[3],
            record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::ContractGracePeriodStarted {
                contract_id,
                node_id,
                twin_id: 3,
                block_number: 11
            }))
        );

        pool_state
            .write()
            .should_call_bill_contract(contract_id, Ok(Pays::Yes.into()), 21);
        run_to_block(21, Some(&mut pool_state));

        pool_state
            .write()
            .should_call_bill_contract(contract_id, Ok(Pays::Yes.into()), 31);
        run_to_block(31, Some(&mut pool_state));

        // Transfer some balance to the owner of the contract to trigger the grace period to stop
        Balances::transfer(RuntimeOrigin::signed(bob()), charlie(), 100000000).unwrap();

        pool_state
            .write()
            .should_call_bill_contract(contract_id, Ok(Pays::Yes.into()), 41);
        run_to_block(41, Some(&mut pool_state));

        pool_state
            .write()
            .should_call_bill_contract(contract_id, Ok(Pays::Yes.into()), 51);
        run_to_block(51, Some(&mut pool_state));

        let c1 = SmartContractModule::contracts(contract_id).unwrap();
        assert_eq!(c1.state, types::ContractState::Created);
    });
}

#[test]
fn test_restore_rent_contract_and_node_contracts_in_grace_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_dedicated_farm_and_node();
        let node_id = 1;
        run_to_block(1, None);

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_rent_contract(
            RuntimeOrigin::signed(charlie()),
            node_id,
            None
        ));
        let rent_contract_id = 1;

        run_to_block(2, None);

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(charlie()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let node_contract_id = 2;
        push_contract_resources_used(node_contract_id);

        // cycle 1
        pool_state
            .write()
            .should_call_bill_contract(rent_contract_id, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));
        pool_state
            .write()
            .should_call_bill_contract(node_contract_id, Ok(Pays::Yes.into()), 12);
        run_to_block(12, Some(&mut pool_state));

        let c1 = SmartContractModule::contracts(rent_contract_id).unwrap();
        assert_eq!(c1.state, types::ContractState::GracePeriod(11));

        let our_events = System::events();
        assert_eq!(
            our_events[5],
            record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::ContractGracePeriodStarted {
                contract_id: rent_contract_id,
                node_id,
                twin_id: 3,
                block_number: 11
            }))
        );
        assert_eq!(
            our_events[6],
            record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::ContractGracePeriodStarted {
                contract_id: node_contract_id,
                node_id,
                twin_id: 3,
                block_number: 11
            }))
        );

        pool_state
            .write()
            .should_call_bill_contract(rent_contract_id, Ok(Pays::Yes.into()), 21);
        run_to_block(21, Some(&mut pool_state));
        pool_state
            .write()
            .should_call_bill_contract(node_contract_id, Ok(Pays::Yes.into()), 22);
        run_to_block(22, Some(&mut pool_state));

        pool_state
            .write()
            .should_call_bill_contract(rent_contract_id, Ok(Pays::Yes.into()), 31);
        run_to_block(31, Some(&mut pool_state));
        pool_state
            .write()
            .should_call_bill_contract(node_contract_id, Ok(Pays::Yes.into()), 32);
        run_to_block(32, Some(&mut pool_state));

        // Transfer some balance to the owner of the contract to trigger the grace period to stop
        Balances::transfer(RuntimeOrigin::signed(bob()), charlie(), 100000000).unwrap();

        pool_state
            .write()
            .should_call_bill_contract(rent_contract_id, Ok(Pays::Yes.into()), 41);
        run_to_block(41, Some(&mut pool_state));
        pool_state
            .write()
            .should_call_bill_contract(node_contract_id, Ok(Pays::Yes.into()), 42);
        run_to_block(42, Some(&mut pool_state));

        pool_state
            .write()
            .should_call_bill_contract(rent_contract_id, Ok(Pays::Yes.into()), 51);
        run_to_block(51, Some(&mut pool_state));
        pool_state
            .write()
            .should_call_bill_contract(node_contract_id, Ok(Pays::Yes.into()), 52);
        run_to_block(52, Some(&mut pool_state));

        let c1 = SmartContractModule::contracts(rent_contract_id).unwrap();
        assert_eq!(c1.state, types::ContractState::Created);

        let our_events = System::events();

        assert_eq!(
            our_events[8],
            record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::ContractGracePeriodEnded {
                contract_id: rent_contract_id,
                node_id,
                twin_id: 3,
            }))
        );
        assert_eq!(
            our_events[9],
            record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::ContractGracePeriodEnded {
                contract_id: node_contract_id,
                node_id,
                twin_id: 3,
            }))
        );
    });
}

#[test]
fn test_rent_contract_grace_period_cancels_contract_when_grace_period_ends_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, None);
        prepare_dedicated_farm_and_node();
        let node_id = 1;

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_rent_contract(
            RuntimeOrigin::signed(charlie()),
            node_id,
            None
        ));
        let contract_id = 1;

        // cycle 1
        pool_state
            .write()
            .should_call_bill_contract(contract_id, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

        let c1 = SmartContractModule::contracts(contract_id).unwrap();
        assert_eq!(c1.state, types::ContractState::GracePeriod(11));

        let our_events = System::events();
        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::ContractGracePeriodStarted {
                    contract_id,
                    node_id,
                    twin_id: 3,
                    block_number: 11
                }
            ))),
            true
        );

        // run 9 more cycles
        for i in 0..10 {
            let block_number = 21 + i * 10;
            pool_state.write().should_call_bill_contract(
                contract_id,
                Ok(Pays::Yes.into()),
                block_number,
            );
            run_to_block(block_number, Some(&mut pool_state));
        }

        // after 10 cycles grace period has finished
        // so contract is removed and no more billing!
        let c1 = SmartContractModule::contracts(contract_id);
        assert_eq!(c1, None);

        // Ensure contract_id is not in billing loop anymore
        let index = SmartContractModule::get_contract_billing_loop_index(contract_id);
        assert_eq!(
            SmartContractModule::contract_to_bill_at_block(index),
            Vec::<u64>::new()
        );
    });
}

#[test]
fn test_rent_contract_and_node_contract_canceled_when_node_is_deleted_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, None);
        prepare_dedicated_farm_and_node();
        let node_id = 1;

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::create_rent_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            None
        ));
        let rent_contract_id = 1;

        run_to_block(2, None);

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let node_contract_id = 2;
        push_contract_resources_used(node_contract_id);

        // 2 contracts => 2 calls to bill_contract
        pool_state
            .write()
            .should_call_bill_contract(rent_contract_id, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));
        pool_state
            .write()
            .should_call_bill_contract(node_contract_id, Ok(Pays::Yes.into()), 12);
        run_to_block(12, Some(&mut pool_state));

        run_to_block(16, Some(&mut pool_state));

        // Delete node
        TfgridModule::delete_node_farm(RuntimeOrigin::signed(alice()), node_id).unwrap();

        let our_events = System::events();

        let ip = b"1.1.1.0".to_vec();
        let mut ips = Vec::new();
        ips.push(ip);

        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::NodeContractCanceled {
                    contract_id: node_contract_id,
                    node_id,
                    twin_id: 2
                }
            ))),
            true
        );
        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::RentContractCanceled {
                    contract_id: rent_contract_id
                }
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
            RuntimeOrigin::signed(alice()),
            b"some_description".to_vec(),
            b"some_link".to_vec(),
            providers
        ));
        let provider_id = 1;

        assert_ok!(SmartContractModule::approve_solution_provider(
            RawOrigin::Root.into(),
            provider_id,
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
                RuntimeOrigin::signed(alice()),
                b"some_description".to_vec(),
                b"some_link".to_vec(),
                providers
            ),
            Error::<TestRuntime>::InvalidProviderConfiguration
        );
    })
}

#[test]
fn test_create_node_contract_with_solution_provider_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        prepare_solution_provider(alice());
        let provider_id = 1;

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(alice()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            Some(provider_id)
        ));
    });
}

#[test]
fn test_create_node_contract_with_solution_provider_fails_if_not_approved() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        let node_id = 1;

        let provider = super::types::Provider {
            take: 10,
            who: alice(),
        };
        let providers = vec![provider];

        assert_ok!(SmartContractModule::create_solution_provider(
            RuntimeOrigin::signed(alice()),
            b"some_description".to_vec(),
            b"some_link".to_vec(),
            providers
        ));
        let provider_id = 1;

        assert_noop!(
            SmartContractModule::create_node_contract(
                RuntimeOrigin::signed(alice()),
                node_id,
                generate_deployment_hash(),
                get_deployment_data(),
                0,
                Some(provider_id)
            ),
            Error::<TestRuntime>::SolutionProviderNotApproved
        );
    });
}

// SERVICE CONTRACT TESTS //
// ---------------------- //

#[test]
fn test_service_contract_create_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        create_service_consumer_contract();
        let service_contract_id = 1;

        let service_contract = get_service_contract();
        assert_eq!(
            service_contract,
            SmartContractModule::service_contracts(service_contract_id).unwrap(),
        );

        let our_events = System::events();
        assert_eq!(!our_events.is_empty(), true);
        assert_eq!(
            our_events.last().unwrap(),
            &record(MockEvent::SmartContractModule(
                SmartContractEvent::ServiceContractCreated(service_contract)
            )),
        );
    });
}

#[test]
fn test_service_contract_create_by_anyone_fails() {
    new_test_ext().execute_with(|| {
        create_twin(alice());
        create_twin(bob());
        create_twin(charlie());

        assert_noop!(
            SmartContractModule::service_contract_create(
                RuntimeOrigin::signed(charlie()),
                alice(),
                bob(),
            ),
            Error::<TestRuntime>::TwinNotAuthorized
        );
    });
}

#[test]
fn test_service_contract_create_same_account_fails() {
    new_test_ext().execute_with(|| {
        create_twin(alice());

        assert_noop!(
            SmartContractModule::service_contract_create(
                RuntimeOrigin::signed(alice()),
                alice(),
                alice(),
            ),
            Error::<TestRuntime>::ServiceContractCreationNotAllowed
        );
    });
}

#[test]
fn test_service_contract_set_metadata_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        create_service_consumer_contract();
        let service_contract_id = 1;

        assert_ok!(SmartContractModule::service_contract_set_metadata(
            RuntimeOrigin::signed(alice()),
            service_contract_id,
            b"some_metadata".to_vec(),
        ));

        let mut service_contract = get_service_contract();
        service_contract.metadata = BoundedVec::try_from(b"some_metadata".to_vec()).unwrap();
        assert_eq!(
            service_contract,
            SmartContractModule::service_contracts(1).unwrap(),
        );

        let our_events = System::events();
        assert_eq!(!our_events.is_empty(), true);
        assert_eq!(
            our_events.last().unwrap(),
            &record(MockEvent::SmartContractModule(
                SmartContractEvent::ServiceContractMetadataSet(service_contract)
            )),
        );
    });
}

#[test]
fn test_service_contract_set_metadata_by_unauthorized_fails() {
    new_test_ext().execute_with(|| {
        create_service_consumer_contract();
        let service_contract_id = 1;
        create_twin(charlie());

        assert_noop!(
            SmartContractModule::service_contract_set_metadata(
                RuntimeOrigin::signed(charlie()),
                service_contract_id,
                b"some_metadata".to_vec(),
            ),
            Error::<TestRuntime>::TwinNotAuthorized
        );
    });
}

#[test]
fn test_service_contract_set_metadata_already_approved_fails() {
    new_test_ext().execute_with(|| {
        prepare_service_consumer_contract();
        let service_contract_id = 1;
        approve_service_consumer_contract(service_contract_id);

        assert_noop!(
            SmartContractModule::service_contract_set_metadata(
                RuntimeOrigin::signed(alice()),
                service_contract_id,
                b"some_metadata".to_vec(),
            ),
            Error::<TestRuntime>::ServiceContractModificationNotAllowed
        );
    });
}

#[test]
fn test_service_contract_set_metadata_too_long_fails() {
    new_test_ext().execute_with(|| {
        create_service_consumer_contract();
        let service_contract_id = 1;

        assert_noop!(
            SmartContractModule::service_contract_set_metadata(
                RuntimeOrigin::signed(alice()),
                service_contract_id,
                b"very_loooooooooooooooooooooooooooooooooooooooooooooooooong_metadata".to_vec(),
            ),
            Error::<TestRuntime>::ServiceContractMetadataTooLong
        );
    });
}

#[test]
fn test_service_contract_set_fees_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        create_service_consumer_contract();
        let service_contract_id = 1;

        assert_ok!(SmartContractModule::service_contract_set_fees(
            RuntimeOrigin::signed(alice()),
            1,
            BASE_FEE,
            VARIABLE_FEE,
        ));

        let mut service_contract = get_service_contract();
        service_contract.base_fee = BASE_FEE;
        service_contract.variable_fee = VARIABLE_FEE;
        assert_eq!(
            service_contract,
            SmartContractModule::service_contracts(service_contract_id).unwrap(),
        );

        let our_events = System::events();
        assert_eq!(!our_events.is_empty(), true);
        assert_eq!(
            our_events.last().unwrap(),
            &record(MockEvent::SmartContractModule(
                SmartContractEvent::ServiceContractFeesSet(service_contract)
            )),
        );
    });
}

#[test]
fn test_service_contract_set_fees_by_unauthorized_fails() {
    new_test_ext().execute_with(|| {
        create_service_consumer_contract();
        let service_contract_id = 1;

        assert_noop!(
            SmartContractModule::service_contract_set_fees(
                RuntimeOrigin::signed(bob()),
                service_contract_id,
                BASE_FEE,
                VARIABLE_FEE,
            ),
            Error::<TestRuntime>::TwinNotAuthorized
        );
    });
}

#[test]
fn test_service_contract_set_fees_already_approved_fails() {
    new_test_ext().execute_with(|| {
        prepare_service_consumer_contract();
        let service_contract_id = 1;
        approve_service_consumer_contract(service_contract_id);

        assert_noop!(
            SmartContractModule::service_contract_set_fees(
                RuntimeOrigin::signed(alice()),
                service_contract_id,
                BASE_FEE,
                VARIABLE_FEE,
            ),
            Error::<TestRuntime>::ServiceContractModificationNotAllowed
        );
    });
}

#[test]
fn test_service_contract_approve_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_service_consumer_contract();
        let service_contract_id = 1;

        let mut service_contract = get_service_contract();
        service_contract.base_fee = BASE_FEE;
        service_contract.variable_fee = VARIABLE_FEE;
        service_contract.metadata = BoundedVec::try_from(b"some_metadata".to_vec()).unwrap();
        service_contract.state = types::ServiceContractState::AgreementReady;
        assert_eq!(
            service_contract,
            SmartContractModule::service_contracts(1).unwrap(),
        );

        // Service approves
        assert_ok!(SmartContractModule::service_contract_approve(
            RuntimeOrigin::signed(alice()),
            service_contract_id,
        ));

        service_contract.accepted_by_service = true;
        assert_eq!(
            service_contract,
            SmartContractModule::service_contracts(1).unwrap(),
        );

        let our_events = System::events();
        assert_eq!(!our_events.is_empty(), true);
        assert_eq!(
            our_events.last().unwrap(),
            &record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::ServiceContractApproved(
                service_contract.clone()
            ))),
        );

        // Consumer approves
        assert_ok!(SmartContractModule::service_contract_approve(
            RuntimeOrigin::signed(bob()),
            service_contract_id,
        ));

        service_contract.accepted_by_consumer = true;
        service_contract.last_bill = get_timestamp_in_seconds_for_block(1);
        service_contract.state = types::ServiceContractState::ApprovedByBoth;
        assert_eq!(
            service_contract,
            SmartContractModule::service_contracts(1).unwrap(),
        );

        let our_events = System::events();
        assert_eq!(!our_events.is_empty(), true);
        assert_eq!(
            our_events.last().unwrap(),
            &record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::ServiceContractApproved(
                service_contract
            ))),
        );
    });
}

#[test]
fn test_service_contract_approve_agreement_not_ready_fails() {
    new_test_ext().execute_with(|| {
        create_service_consumer_contract();
        let service_contract_id = 1;

        assert_noop!(
            SmartContractModule::service_contract_approve(
                RuntimeOrigin::signed(alice()),
                service_contract_id
            ),
            Error::<TestRuntime>::ServiceContractApprovalNotAllowed
        );
    });
}

#[test]
fn test_service_contract_approve_already_approved_fails() {
    new_test_ext().execute_with(|| {
        prepare_service_consumer_contract();
        let service_contract_id = 1;
        approve_service_consumer_contract(service_contract_id);

        assert_noop!(
            SmartContractModule::service_contract_approve(
                RuntimeOrigin::signed(alice()),
                service_contract_id
            ),
            Error::<TestRuntime>::ServiceContractApprovalNotAllowed
        );
    });
}

#[test]
fn test_service_contract_approve_by_unauthorized_fails() {
    new_test_ext().execute_with(|| {
        prepare_service_consumer_contract();
        let service_contract_id = 1;
        create_twin(charlie());

        assert_noop!(
            SmartContractModule::service_contract_approve(
                RuntimeOrigin::signed(charlie()),
                service_contract_id
            ),
            Error::<TestRuntime>::TwinNotAuthorized
        );
    });
}

#[test]
fn test_service_contract_reject_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_service_consumer_contract();
        let service_contract_id = 1;

        assert_ok!(SmartContractModule::service_contract_reject(
            RuntimeOrigin::signed(alice()),
            service_contract_id,
        ));

        assert_eq!(SmartContractModule::service_contracts(1).is_none(), true);

        let our_events = System::events();
        assert_eq!(!our_events.is_empty(), true);
        assert_eq!(
            our_events.last().unwrap(),
            &record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::ServiceContractCanceled {
                service_contract_id,
                cause: types::Cause::CanceledByUser,
            })),
        );
    });
}

#[test]
fn test_service_contract_reject_agreement_not_ready_fails() {
    new_test_ext().execute_with(|| {
        create_service_consumer_contract();
        let service_contract_id = 1;

        assert_noop!(
            SmartContractModule::service_contract_reject(
                RuntimeOrigin::signed(alice()),
                service_contract_id
            ),
            Error::<TestRuntime>::ServiceContractRejectionNotAllowed
        );
    });
}

#[test]
fn test_service_contract_reject_already_approved_fails() {
    new_test_ext().execute_with(|| {
        prepare_service_consumer_contract();
        let service_contract_id = 1;
        approve_service_consumer_contract(service_contract_id);

        assert_noop!(
            SmartContractModule::service_contract_reject(
                RuntimeOrigin::signed(alice()),
                service_contract_id
            ),
            Error::<TestRuntime>::ServiceContractRejectionNotAllowed
        );
    });
}

#[test]
fn test_service_contract_reject_by_unauthorized_fails() {
    new_test_ext().execute_with(|| {
        prepare_service_consumer_contract();
        let service_contract_id = 1;
        create_twin(charlie());

        assert_noop!(
            SmartContractModule::service_contract_reject(
                RuntimeOrigin::signed(charlie()),
                service_contract_id
            ),
            Error::<TestRuntime>::TwinNotAuthorized
        );
    });
}

#[test]
fn test_service_contract_cancel_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        create_service_consumer_contract();
        let service_contract_id = 1;

        assert_ok!(SmartContractModule::service_contract_cancel(
            RuntimeOrigin::signed(alice()),
            service_contract_id,
        ));

        assert_eq!(SmartContractModule::service_contracts(1).is_none(), true);

        let our_events = System::events();
        assert_eq!(!our_events.is_empty(), true);
        assert_eq!(
            our_events.last().unwrap(),
            &record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::ServiceContractCanceled {
                service_contract_id,
                cause: types::Cause::CanceledByUser,
            })),
        );
    });
}

#[test]
fn test_service_contract_cancel_by_unauthorized_fails() {
    new_test_ext().execute_with(|| {
        create_service_consumer_contract();
        create_twin(charlie());
        let service_contract_id = 1;

        assert_noop!(
            SmartContractModule::service_contract_cancel(
                RuntimeOrigin::signed(charlie()),
                service_contract_id
            ),
            Error::<TestRuntime>::TwinNotAuthorized
        );
    });
}

#[test]
fn test_service_contract_bill_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, None);
        prepare_service_consumer_contract();
        let service_contract_id = 1;

        let service_contract = SmartContractModule::service_contracts(service_contract_id).unwrap();
        assert_eq!(service_contract.last_bill, 0);

        approve_service_consumer_contract(service_contract_id);

        let service_contract = SmartContractModule::service_contracts(service_contract_id).unwrap();
        assert_eq!(
            service_contract.last_bill,
            get_timestamp_in_seconds_for_block(1)
        );

        let consumer_twin = TfgridModule::twins(2).unwrap();
        let consumer_balance = Balances::free_balance(&consumer_twin.account_id);
        assert_eq!(consumer_balance, 2500000000);

        // Bill 20 min after contract approval
        run_to_block(201, Some(&mut pool_state));
        assert_ok!(SmartContractModule::service_contract_bill(
            RuntimeOrigin::signed(alice()),
            1,
            VARIABLE_AMOUNT,
            b"bill_metadata_1".to_vec(),
        ));

        let service_contract = SmartContractModule::service_contracts(service_contract_id).unwrap();
        assert_eq!(
            service_contract.last_bill,
            get_timestamp_in_seconds_for_block(201)
        );

        // Check consumer balance after first billing
        let consumer_balance = Balances::free_balance(&consumer_twin.account_id);
        let window =
            get_timestamp_in_seconds_for_block(201) - get_timestamp_in_seconds_for_block(1);
        let bill_1 = types::ServiceContractBill {
            variable_amount: VARIABLE_AMOUNT,
            window,
            metadata: BoundedVec::try_from(b"bill_metadata_1".to_vec()).unwrap(),
        };
        let billed_amount_1 = service_contract
            .calculate_bill_cost_tft::<TestRuntime>(bill_1.clone())
            .unwrap();
        assert_eq!(2500000000 - consumer_balance, billed_amount_1);

        // Check event triggering
        let our_events = System::events();
        assert_eq!(!our_events.is_empty(), true);
        assert_eq!(
            our_events.last().unwrap(),
            &record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::ServiceContractBilled {
                service_contract,
                bill: bill_1,
                amount: billed_amount_1,
            })),
        );

        // Bill a second time, 1h10min after first billing
        run_to_block(901, Some(&mut pool_state));
        assert_ok!(SmartContractModule::service_contract_bill(
            RuntimeOrigin::signed(alice()),
            service_contract_id,
            VARIABLE_AMOUNT,
            b"bill_metadata_2".to_vec(),
        ));

        let service_contract = SmartContractModule::service_contracts(1).unwrap();
        assert_eq!(
            service_contract.last_bill,
            get_timestamp_in_seconds_for_block(901)
        );

        // Check consumer balance after second billing
        let consumer_balance = Balances::free_balance(&consumer_twin.account_id);
        let bill_2 = types::ServiceContractBill {
            variable_amount: VARIABLE_AMOUNT,
            window: SECS_PER_HOUR, // force a 1h bill here
            metadata: BoundedVec::try_from(b"bill_metadata_2".to_vec()).unwrap(),
        };
        let billed_amount_2 = service_contract
            .calculate_bill_cost_tft::<TestRuntime>(bill_2.clone())
            .unwrap();
        // Second billing was equivalent to a 1h
        // billing even if window is greater than 1h
        assert_eq!(
            2500000000 - consumer_balance - billed_amount_1,
            billed_amount_2
        );

        // Check event triggering
        let our_events = System::events();
        assert_eq!(!our_events.is_empty(), true);
        assert_eq!(
            our_events.last().unwrap(),
            &record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::ServiceContractBilled {
                service_contract,
                bill: bill_2,
                amount: billed_amount_2,
            })),
        );
    });
}

#[test]
fn test_service_contract_bill_by_unauthorized_fails() {
    new_test_ext().execute_with(|| {
        prepare_service_consumer_contract();
        let service_contract_id = 1;
        approve_service_consumer_contract(service_contract_id);

        assert_noop!(
            SmartContractModule::service_contract_bill(
                RuntimeOrigin::signed(bob()),
                service_contract_id,
                VARIABLE_AMOUNT,
                b"bill_metadata".to_vec(),
            ),
            Error::<TestRuntime>::TwinNotAuthorized
        );
    });
}

#[test]
fn test_service_contract_bill_not_approved_fails() {
    new_test_ext().execute_with(|| {
        prepare_service_consumer_contract();
        let service_contract_id = 1;

        assert_noop!(
            SmartContractModule::service_contract_bill(
                RuntimeOrigin::signed(alice()),
                service_contract_id,
                VARIABLE_AMOUNT,
                b"bill_metadata".to_vec(),
            ),
            Error::<TestRuntime>::ServiceContractBillingNotApprovedByBoth
        );
    });
}

#[test]
fn test_service_contract_bill_variable_amount_too_high_fails() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, None);
        prepare_service_consumer_contract();
        let service_contract_id = 1;
        approve_service_consumer_contract(service_contract_id);

        // Bill 1h after contract approval
        run_to_block(601, Some(&mut pool_state));
        // set variable amount a bit higher than variable fee to trigger error
        let variable_amount = VARIABLE_FEE + 1;
        assert_noop!(
            SmartContractModule::service_contract_bill(
                RuntimeOrigin::signed(alice()),
                service_contract_id,
                variable_amount,
                b"bill_metadata".to_vec(),
            ),
            Error::<TestRuntime>::ServiceContractBillingVariableAmountTooHigh
        );
    });
}

#[test]
fn test_service_contract_bill_metadata_too_long_fails() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, None);
        prepare_service_consumer_contract();
        let service_contract_id = 1;
        approve_service_consumer_contract(service_contract_id);

        // Bill 1h after contract approval
        run_to_block(601, Some(&mut pool_state));
        assert_noop!(
            SmartContractModule::service_contract_bill(
                RuntimeOrigin::signed(alice()),
                service_contract_id,
                VARIABLE_AMOUNT,
                b"very_loooooooooooooooooooooooooooooooooooooooooooooooooong_metadata".to_vec(),
            ),
            Error::<TestRuntime>::ServiceContractBillMetadataTooLong
        );
    });
}

#[test]
fn test_service_contract_bill_out_of_funds_fails() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, None);
        prepare_service_consumer_contract();
        let service_contract_id = 1;
        approve_service_consumer_contract(service_contract_id);

        // Drain consumer account
        let consumer_twin = TfgridModule::twins(2).unwrap();
        let consumer_balance = Balances::free_balance(&consumer_twin.account_id);
        Balances::transfer(RuntimeOrigin::signed(bob()), alice(), consumer_balance).unwrap();
        let consumer_balance = Balances::free_balance(&consumer_twin.account_id);
        assert_eq!(consumer_balance, 0);

        // Bill 1h after contract approval
        run_to_block(601, Some(&mut pool_state));
        assert_noop!(
            SmartContractModule::service_contract_bill(
                RuntimeOrigin::signed(alice()),
                service_contract_id,
                VARIABLE_AMOUNT,
                b"bill_metadata".to_vec(),
            ),
            Error::<TestRuntime>::ServiceContractNotEnoughFundsToPayBill,
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
        let usable_balance = Balances::usable_balance(&bob());
        let free_balance = Balances::free_balance(&bob());

        // should be equal since no activity and no locks
        assert_eq!(usable_balance, free_balance);

        let id: u64 = 1;
        // Try to lock less than EXISTENTIAL_DEPOSIT should fail
        Balances::set_lock(id.to_be_bytes(), &bob(), 100, WithdrawReasons::all());

        // usable balance should now return free balance - EXISTENTIAL_DEPOSIT cause there was some activity
        let usable_balance = Balances::usable_balance(&bob());
        let free_balance = Balances::free_balance(&bob());
        assert_eq!(usable_balance, free_balance - EXISTENTIAL_DEPOSIT);

        // ----- INITIAL ------ //
        // Try to lock more than EXISTENTIAL_DEPOSIT should succeed
        let to_lock = 100 + EXISTENTIAL_DEPOSIT;

        Balances::set_lock(id.to_be_bytes(), &bob(), to_lock, WithdrawReasons::all());

        // usable balance should now be free_balance - to_lock cause there was some activity
        let usable_balance = Balances::usable_balance(&bob());
        let free_balance = Balances::free_balance(&bob());
        assert_eq!(usable_balance, free_balance - to_lock);

        // ----- UPDATE ------ //
        // updating a lock should succeed
        let to_lock = 500 + EXISTENTIAL_DEPOSIT;

        Balances::set_lock(id.to_be_bytes(), &bob(), to_lock, WithdrawReasons::all());

        // usable balance should now be free_balance - to_lock cause there was some activity
        let usable_balance = Balances::usable_balance(&bob());
        let free_balance = Balances::free_balance(&bob());
        assert_eq!(usable_balance, free_balance - to_lock);

        // ----- UNLOCK ------ //
        // Unlock should work
        Balances::remove_lock(id.to_be_bytes(), &bob());

        // usable balance should now be free_balance cause there are no locks
        let usable_balance = Balances::usable_balance(&bob());
        let free_balance = Balances::free_balance(&bob());
        assert_eq!(usable_balance, free_balance);
    })
}

#[test]
fn test_change_billing_frequency_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        let new_frequency = 900;

        assert_ok!(SmartContractModule::change_billing_frequency(
            RawOrigin::Root.into(),
            new_frequency
        ));

        assert_eq!(SmartContractModule::billing_frequency(), new_frequency);

        let our_events = System::events();
        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::BillingFrequencyChanged(new_frequency)
            ))),
            true
        );
    })
}

#[test]
fn test_change_billing_frequency_fails_if_frequency_lower() {
    new_test_ext().execute_with(|| {
        let initial_frequency = SmartContractModule::billing_frequency();
        let new_frequency = initial_frequency - 1;

        assert_noop!(
            SmartContractModule::change_billing_frequency(RawOrigin::Root.into(), new_frequency),
            Error::<TestRuntime>::CanOnlyIncreaseFrequency
        );

        assert_eq!(initial_frequency, SmartContractModule::billing_frequency());
    })
}

#[test]
fn test_attach_solution_provider_id() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(alice()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let contract_id = 1;

        let ctr = SmartContractModule::contracts(contract_id).unwrap();
        assert_eq!(ctr.solution_provider_id, None);

        prepare_solution_provider(alice());
        let provider_id = 1;

        assert_ok!(SmartContractModule::attach_solution_provider_id(
            RuntimeOrigin::signed(alice()),
            contract_id,
            provider_id
        ));

        let ctr = SmartContractModule::contracts(contract_id).unwrap();
        assert_eq!(ctr.solution_provider_id, Some(provider_id));
    })
}

#[test]
fn test_attach_solution_provider_id_wrong_origin_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(alice()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let contract_id = 1;

        let ctr = SmartContractModule::contracts(contract_id).unwrap();
        assert_eq!(ctr.solution_provider_id, None);

        prepare_solution_provider(alice());
        let provider_id = 1;

        assert_noop!(
            SmartContractModule::attach_solution_provider_id(
                RuntimeOrigin::signed(bob()),
                contract_id,
                provider_id
            ),
            Error::<TestRuntime>::UnauthorizedToChangeSolutionProviderId
        );
    })
}

#[test]
fn test_attach_solution_provider_id_not_approved_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(alice()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
        let contract_id = 1;

        let ctr = SmartContractModule::contracts(1).unwrap();
        assert_eq!(ctr.solution_provider_id, None);

        let provider = super::types::Provider {
            take: 10,
            who: dave(),
        };
        let providers = vec![provider];

        assert_ok!(SmartContractModule::create_solution_provider(
            RuntimeOrigin::signed(alice()),
            b"some_description".to_vec(),
            b"some_link".to_vec(),
            providers
        ));
        let provider_id = 1;

        assert_noop!(
            SmartContractModule::attach_solution_provider_id(
                RuntimeOrigin::signed(bob()),
                contract_id,
                provider_id
            ),
            Error::<TestRuntime>::SolutionProviderNotApproved
        );
    })
}

#[test]
fn test_set_dedicated_node_extra_fee_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        let node_id = 1;

        let zero_fee = 0;
        assert_ok!(SmartContractModule::set_dedicated_node_extra_fee(
            RuntimeOrigin::signed(alice()),
            node_id,
            zero_fee
        ));

        assert_eq!(
            SmartContractModule::dedicated_nodes_extra_fee(node_id),
            None
        );

        let extra_fee = 100000;
        assert_ok!(SmartContractModule::set_dedicated_node_extra_fee(
            RuntimeOrigin::signed(alice()),
            node_id,
            extra_fee
        ));

        assert_eq!(
            SmartContractModule::dedicated_nodes_extra_fee(node_id),
            Some(extra_fee)
        );

        let our_events = System::events();
        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::NodeExtraFeeSet { node_id, extra_fee }
            ))),
            true
        );
    })
}

#[test]
fn test_set_dedicated_node_extra_fee_undefined_node_fails() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        let node_id = 1;

        let extra_fee = 100000;
        assert_noop!(
            SmartContractModule::set_dedicated_node_extra_fee(
                RuntimeOrigin::signed(alice()),
                node_id + 1,
                extra_fee
            ),
            Error::<TestRuntime>::NodeNotExists
        );
    })
}

#[test]
fn test_set_dedicated_node_extra_fee_unauthorized_fails() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        let node_id = 1;

        let extra_fee = 100000;
        assert_noop!(
            SmartContractModule::set_dedicated_node_extra_fee(
                RuntimeOrigin::signed(bob()),
                node_id,
                extra_fee
            ),
            Error::<TestRuntime>::UnauthorizedToSetExtraFee
        );
    })
}

#[test]
fn test_set_dedicated_node_extra_fee_with_active_node_contract_fails() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        let node_id = 1;

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));

        let extra_fee = 100000;
        assert_noop!(
            SmartContractModule::set_dedicated_node_extra_fee(
                RuntimeOrigin::signed(alice()),
                node_id,
                extra_fee
            ),
            Error::<TestRuntime>::NodeHasActiveContracts
        );
    })
}

#[test]
fn test_set_dedicated_node_extra_fee_with_active_rent_contract_fails() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        let node_id = 1;

        assert_ok!(SmartContractModule::create_rent_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            None
        ));

        let extra_fee = 100000;
        assert_noop!(
            SmartContractModule::set_dedicated_node_extra_fee(
                RuntimeOrigin::signed(alice()),
                node_id,
                extra_fee
            ),
            Error::<TestRuntime>::NodeHasActiveContracts
        );
    })
}

#[test]
fn test_set_dedicated_node_extra_fee_and_create_node_contract_fails() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        let node_id = 1;

        let extra_fee = 100000;
        assert_ok!(SmartContractModule::set_dedicated_node_extra_fee(
            RuntimeOrigin::signed(alice()),
            node_id,
            extra_fee
        ));

        assert_noop!(
            SmartContractModule::create_node_contract(
                RuntimeOrigin::signed(bob()),
                node_id,
                generate_deployment_hash(),
                get_deployment_data(),
                0,
                None
            ),
            Error::<TestRuntime>::NodeNotAvailableToDeploy
        );
    })
}

#[test]
fn test_set_dedicated_node_extra_fee_and_create_rent_contract_works() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();
        let node_id = 1;

        let extra_fee = 100000;
        assert_ok!(SmartContractModule::set_dedicated_node_extra_fee(
            RuntimeOrigin::signed(alice()),
            node_id,
            extra_fee
        ));

        assert_ok!(SmartContractModule::create_rent_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            None
        ));

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(bob()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));
    })
}

#[test]
fn test_set_dedicated_node_extra_fee_and_create_rent_contract_billing_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_farm_and_node();
        let node_id = 1;

        let start_block = 1;
        run_to_block(start_block, None);

        TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

        let initial_total_issuance = Balances::total_issuance();
        // Get daves's twin
        let twin = TfgridModule::twins(4).unwrap();
        let initial_twin_balance = Balances::free_balance(&twin.account_id);
        log::debug!("Twin balance: {}", initial_twin_balance);

        let extra_fee = 100000;
        assert_ok!(SmartContractModule::set_dedicated_node_extra_fee(
            RuntimeOrigin::signed(alice()),
            node_id,
            extra_fee
        ));

        assert_ok!(SmartContractModule::create_rent_contract(
            RuntimeOrigin::signed(dave()),
            node_id,
            None
        ));

        assert_ok!(SmartContractModule::create_node_contract(
            RuntimeOrigin::signed(dave()),
            node_id,
            generate_deployment_hash(),
            get_deployment_data(),
            0,
            None
        ));

        let rent_contract_id = 1;
        let rent_contract = SmartContractModule::contracts(rent_contract_id).unwrap();

        // Ensure contract_id is stored at right billing loop index
        let index = SmartContractModule::get_contract_billing_loop_index(rent_contract_id);
        assert_eq!(
            SmartContractModule::contract_to_bill_at_block(index),
            vec![rent_contract_id]
        );

        let now = Timestamp::get().saturated_into::<u64>() / 1000;
        let mut rent_contract_cost_tft = 0u64;
        let mut extra_fee_cost_tft = 0;

        // advance 24 cycles to reach reward distribution block
        for i in 1..=DistributionFrequency::get() as u64 {
            let block_number = start_block + i * BillingFrequency::get();
            pool_state.write().should_call_bill_contract(
                rent_contract_id,
                Ok(Pays::Yes.into()),
                block_number,
            );
            run_to_block(block_number, Some(&mut pool_state));

            // check why aggregating seconds elapsed is giving different results
            let elapsed_time_in_secs = BillingFrequency::get() * SECS_PER_BLOCK;

            // aggregate rent contract cost
            let free_balance = Balances::free_balance(&twin.account_id);
            let (contract_cost_tft, _) = rent_contract
                .calculate_contract_cost_tft(free_balance, elapsed_time_in_secs)
                .unwrap();
            rent_contract_cost_tft += contract_cost_tft;

            // aggregate extra fee cost
            extra_fee_cost_tft += rent_contract
                .calculate_extra_fee_cost_tft(node_id, elapsed_time_in_secs)
                .unwrap();
        }

        let then = Timestamp::get().saturated_into::<u64>() / 1000;
        let seconds_elapsed = then - now;
        log::debug!("seconds elapsed: {}", seconds_elapsed);

        let events = System::events();
        for event in events.iter() {
            log::debug!("Event: {:?}", event.event);
        }

        let free_balance = Balances::free_balance(&twin.account_id);
        let total_amount_billed_tft = initial_twin_balance - free_balance;
        log::debug!("total amount billed: {}", total_amount_billed_tft);
        log::debug!(
            "total amount billed for rent contract: {}",
            rent_contract_cost_tft
        );
        log::debug!("total amount billed for extra fee: {}", extra_fee_cost_tft);

        // Ensure total amount billed after 24 cycles is equal
        // to aggregated rent contract cost + aggregated extra_fee_cost
        assert_eq!(
            total_amount_billed_tft,
            rent_contract_cost_tft + extra_fee_cost_tft
        );

        validate_distribution_rewards(initial_total_issuance, rent_contract_cost_tft, false);
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

macro_rules! test_calculate_discount {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() {
            let (number_of_months, expected_discount_level) = $value;

            let amount_due = 1000;
            let seconds_elapsed = SECS_PER_HOUR; // amount due is relative to 1h
            // Give just enough balance for targeted number of months at the rate of 1000 per hour
            let balance = U64F64::from_num(amount_due * 24 * 30) * U64F64::from_num(number_of_months);

            let result = cost::calculate_discount::<TestRuntime>(
                amount_due,
                seconds_elapsed,
                balance.round().to_num::<u64>(),
                NodeCertification::Diy,
            );

            assert_eq!(
                result,
                (
                    (U64F64::from_num(amount_due) * expected_discount_level.price_multiplier())
                        .round()
                        .to_num::<u64>(),
                    expected_discount_level
                )
            );
        }
    )*
    }
}

// Confirm expected discount level given a number of month of balance autonomy
test_calculate_discount! {
    test_calculate_discount_none_works: (1, types::DiscountLevel::None),
    test_calculate_discount_default_works: (1.5, types::DiscountLevel::Default),
    test_calculate_discount_bronze_works: (3, types::DiscountLevel::Bronze),
    test_calculate_discount_silver_works: (6, types::DiscountLevel::Silver),
    test_calculate_gold_discount_gold_works: (18, types::DiscountLevel::Gold),
}

// ***** HELPER FUNCTIONS ***** //
// ---------------------------- //
// ---------------------------- //

fn validate_distribution_rewards(
    initial_total_issuance: u64,
    total_amount_billed: u64,
    had_solution_provider: bool,
) {
    info!("total amount billed {:?}", total_amount_billed);

    let staking_pool_account_balance = Balances::free_balance(&get_staking_pool_account());
    info!(
        "staking pool account balance, {:?}",
        staking_pool_account_balance
    );

    // 5% is sent to the staking pool account
    assert_eq_error_rate!(
        staking_pool_account_balance,
        Perbill::from_percent(5) * total_amount_billed,
        6
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
        info!("solution provider b: {:?}", solution_provider_1_balance);
        assert_ne!(solution_provider_1_balance, 0);
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
        timestamp: get_timestamp_in_seconds_for_block(block_number),
        window: 6 * block_number,
    });

    assert_ok!(SmartContractModule::add_nru_reports(
        RuntimeOrigin::signed(alice()),
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
        RuntimeOrigin::signed(alice()),
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

    for e in our_events.clone().iter() {
        info!("{:?}", e);
    }

    let contract_bill = types::ContractBill {
        contract_id,
        timestamp: get_timestamp_in_seconds_for_block(block_number),
        discount_level,
        amount_billed: amount_billed as u128,
    };

    info!(
        "event: {:?}",
        record(MockEvent::SmartContractModule(SmartContractEvent::<
            TestRuntime,
        >::ContractBilled(
            contract_bill.clone()
        )))
    );

    assert_eq!(
        our_events.contains(&record(MockEvent::SmartContractModule(
            SmartContractEvent::<TestRuntime>::ContractBilled(contract_bill)
        ))),
        true
    );
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
    create_twin(dave());
}

pub fn prepare_farm(source: AccountId, dedicated: bool) {
    let farm_name = "test_farm";
    let mut pub_ips = Vec::new();
    pub_ips.push(IP4 {
        ip: b"185.206.122.33/24".to_vec().try_into().unwrap(),
        gw: b"185.206.122.1".to_vec().try_into().unwrap(),
    });
    pub_ips.push(IP4 {
        ip: b"185.206.122.34/24".to_vec().try_into().unwrap(),
        gw: b"185.206.122.1".to_vec().try_into().unwrap(),
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
        b"policy_1".to_vec(),
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
        RuntimeOrigin::signed(source),
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
    TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();

    create_farming_policies();
    prepare_twins();
    prepare_farm(alice(), false);

    let resources = ResourcesInput {
        hru: 1024 * GIGABYTE,
        sru: 512 * GIGABYTE,
        cru: 8,
        mru: 16 * GIGABYTE,
    };

    // random location
    let location = LocationInput {
        city: get_city_name_input(b"Ghent"),
        country: get_country_name_input(b"Belgium"),
        latitude: get_latitude_input(b"12.233213231"),
        longitude: get_longitude_input(b"32.323112123"),
    };

    TfgridModule::create_node(
        RuntimeOrigin::signed(alice()),
        1,
        resources,
        location,
        bounded_vec![],
        false,
        false,
        None,
    )
    .unwrap();
}

pub fn prepare_dedicated_farm_and_node() {
    TFTPriceModule::set_prices(RuntimeOrigin::signed(alice()), 50, 101).unwrap();
    create_farming_policies();
    prepare_twins();
    prepare_farm(alice(), true);

    let resources = ResourcesInput {
        hru: 1024 * GIGABYTE,
        sru: 512 * GIGABYTE,
        cru: 8,
        mru: 16 * GIGABYTE,
    };

    // random location
    let location = LocationInput {
        city: get_city_name_input(b"Ghent"),
        country: get_country_name_input(b"Belgium"),
        latitude: get_latitude_input(b"12.233213231"),
        longitude: get_longitude_input(b"32.323112123"),
    };

    TfgridModule::create_node(
        RuntimeOrigin::signed(alice()),
        1,
        resources,
        location,
        bounded_vec![],
        false,
        false,
        None,
    )
    .unwrap();
}

pub fn create_twin(origin: AccountId) {
    assert_ok!(TfgridModule::user_accept_tc(
        RuntimeOrigin::signed(origin.clone()),
        get_document_link_input(b"some_link"),
        get_document_hash_input(b"some_hash"),
    ));

    let relay = get_relay_input(b"somerelay.io");
    let pk =
        get_public_key_input(b"0x6c8fd181adc178cea218e168e8549f0b0ff30627c879db9eac4318927e87c901");
    assert_ok!(TfgridModule::create_twin(
        RuntimeOrigin::signed(origin),
        relay,
        pk
    ));
}

fn create_farming_policies() {
    let name = b"f1".to_vec();
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

    let name = b"f2".to_vec();
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

    let name = b"f3".to_vec();
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

    let name = b"f1".to_vec();
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

fn prepare_solution_provider(origin: AccountId) {
    let provider = super::types::Provider {
        take: 10,
        who: dave(),
    };
    let providers = vec![provider];

    assert_ok!(SmartContractModule::create_solution_provider(
        RuntimeOrigin::signed(origin),
        b"some_description".to_vec(),
        b"some_link".to_vec(),
        providers
    ));
    let provider_id = 1;

    assert_ok!(SmartContractModule::approve_solution_provider(
        RawOrigin::Root.into(),
        provider_id,
        true
    ));
}

fn record(event: RuntimeEvent) -> EventRecord<RuntimeEvent, H256> {
    EventRecord {
        phase: Phase::Initialization,
        event,
        topics: vec![],
    }
}

fn generate_deployment_hash() -> HexHash {
    let hash: [u8; 32] = H256::random().to_fixed_bytes();
    hash
}

fn get_deployment_data() -> crate::DeploymentDataInput<TestRuntime> {
    BoundedVec::<u8, crate::MaxDeploymentDataLength<TestRuntime>>::try_from(b"some_data".to_vec())
        .unwrap()
}

fn create_service_consumer_contract() {
    create_twin(alice());
    create_twin(bob());

    // create contract between service (Alice) and consumer (Bob)
    assert_ok!(SmartContractModule::service_contract_create(
        RuntimeOrigin::signed(alice()),
        alice(),
        bob(),
    ));
}

fn prepare_service_consumer_contract() {
    create_service_consumer_contract();
    let service_contract_id = 1;

    assert_ok!(SmartContractModule::service_contract_set_metadata(
        RuntimeOrigin::signed(alice()),
        service_contract_id,
        b"some_metadata".to_vec(),
    ));

    assert_ok!(SmartContractModule::service_contract_set_fees(
        RuntimeOrigin::signed(alice()),
        service_contract_id,
        BASE_FEE,
        VARIABLE_FEE,
    ));
}

fn approve_service_consumer_contract(service_contract_id: u64) {
    // Service approves
    assert_ok!(SmartContractModule::service_contract_approve(
        RuntimeOrigin::signed(alice()),
        service_contract_id,
    ));
    // Consumer approves
    assert_ok!(SmartContractModule::service_contract_approve(
        RuntimeOrigin::signed(bob()),
        service_contract_id,
    ));
}

fn get_service_contract() -> types::ServiceContract {
    types::ServiceContract {
        service_contract_id: 1,
        service_twin_id: 1,  //Alice
        consumer_twin_id: 2, //Bob
        base_fee: 0,
        variable_fee: 0,
        metadata: bounded_vec![],
        accepted_by_service: false,
        accepted_by_consumer: false,
        last_bill: 0,
        state: types::ServiceContractState::Created,
    }
}

fn get_timestamp_in_seconds_for_block(block_number: u64) -> u64 {
    1628082000 + (6 * block_number)
}
