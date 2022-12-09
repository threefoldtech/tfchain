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
use tfchain_support::types::{
    CapacityReservationPolicy, ConsumableResources, NodeFeatures, PowerTarget, PublicConfig, IP,
};
use tfchain_support::types::{FarmCertification, NodeCertification, PublicIP};

const GIGABYTE: u64 = 1024 * 1024 * 1024;

//  GROUP TESTS //
// -------------------- //

#[test]
fn test_create_group_then_capacity_reservation_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::group_create(RuntimeOrigin::signed(
            alice()
        )));
        let group = SmartContractModule::groups(1).unwrap();
        assert_eq!(group.twin_id, 1);
        assert_eq!(group.capacity_reservation_contract_ids.len(), 0);

        create_capacity_reservation_and_add_to_group(1, resources_c1(), None, 1, 1);

        let our_events = System::events();
        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::GroupCreated {
                    group_id: 1,
                    twin_id: 1,
                }
            ))),
            true
        );
    });
}

#[test]
fn test_remove_group_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::group_create(RuntimeOrigin::signed(
            alice()
        )));

        assert_ok!(SmartContractModule::group_delete(
            RuntimeOrigin::signed(alice()),
            1
        ));

        assert_eq!(SmartContractModule::groups(1), None);

        let our_events = System::events();
        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::GroupDeleted { group_id: 1 }
            ))),
            true
        );
    });
}

#[test]
fn test_remove_group_unauthorized_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::group_create(RuntimeOrigin::signed(
            alice()
        )));

        assert_noop!(
            SmartContractModule::group_delete(RuntimeOrigin::signed(bob()), 1),
            Error::<TestRuntime>::TwinNotAuthorizedToDeleteGroup
        );
    });
}

#[test]
fn test_remove_group_active_capacity_reservation_contracts_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::group_create(RuntimeOrigin::signed(
            alice()
        )));
        let group = SmartContractModule::groups(1).unwrap();
        assert_eq!(group.twin_id, 1);
        assert_eq!(group.capacity_reservation_contract_ids.len(), 0);

        create_capacity_reservation_and_add_to_group(1, resources_c1(), None, 1, 1);

        assert_noop!(
            SmartContractModule::group_delete(RuntimeOrigin::signed(alice()), 1),
            Error::<TestRuntime>::GroupHasActiveMembers
        );
    });
}

#[test]
fn test_create_capacity_contract_reservation_finding_node_using_group() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_with_three_nodes();

        assert_ok!(SmartContractModule::group_create(RuntimeOrigin::signed(
            alice()
        )));

        // although there is still place to add the contract on node 1 all contracts are in the same group
        // so they should not go on the same node
        create_capacity_reservation_and_add_to_group(1, resources_c1(), None, 1, 1);
        create_capacity_reservation_and_add_to_group(1, resources_c1(), None, 1, 2);
        create_capacity_reservation_and_add_to_group(1, resources_c1(), None, 1, 3);

        // only three nodes so no more node left that doesn't contain a capacity reservation contract that is in the same group
        assert_noop!(
            SmartContractModule::contract_capacity_reservation_create(
                RuntimeOrigin::signed(alice()),
                1,
                CapacityReservationPolicy::Exclusive {
                    group_id: 1,
                    resources: resources_c1(),
                    features: None,
                },
                None,
            ),
            Error::<TestRuntime>::NoSuitableNodeInFarm
        );
        // let's add it without a group
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(alice()),
            1,
            CapacityReservationPolicy::Any {
                resources: resources_c1(),
                features: None,
            },
            None,
        ));
        assert_eq!(
            SmartContractModule::contracts(4).unwrap().contract_type,
            types::ContractData::CapacityReservationContract(types::CapacityReservationContract {
                node_id: 1,
                group_id: None,
                public_ips: 0,
                resources: ConsumableResources {
                    total_resources: resources_c1(),
                    used_resources: ResourcesInput::empty(),
                },
                deployments: vec![],
            })
        );
        assert_eq!(
            TfgridModule::nodes(1).unwrap().resources.used_resources,
            ResourcesInput::sum(&resources_c1(), &resources_c1())
        );
    });
}

#[test]
fn test_capacity_reservation_contract_with_policy_any_and_features_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_with_three_nodes();
        add_public_config(1, 3, alice());

        // Contract should go to node 3 (the only node with a public config) and thus bring it up
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(alice()),
            1,
            CapacityReservationPolicy::Any {
                resources: resources_c3(),
                features: Some(vec![NodeFeatures::PublicNode]),
            },
            None,
        ));

        assert_eq!(
            SmartContractModule::contracts(1).unwrap().contract_type,
            types::ContractData::CapacityReservationContract(types::CapacityReservationContract {
                node_id: 3,
                group_id: None,
                public_ips: 0,
                resources: ConsumableResources {
                    total_resources: resources_c3(),
                    used_resources: ResourcesInput::empty(),
                },
                deployments: vec![],
            })
        );

        assert_eq!(
            TfgridModule::nodes(3).unwrap().power.target,
            PowerTarget::Up
        );
    });
}

#[test]
fn test_capacity_reservation_contract_with_policy_exclusive_and_features_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_with_three_nodes();
        // node 2 and 3 have public config
        add_public_config(1, 2, alice());
        add_public_config(1, 3, alice());

        assert_ok!(SmartContractModule::group_create(RuntimeOrigin::signed(
            alice()
        )));

        // Contract should go to node 2 (enough resources + has public config) and thus bring it up
        let expected_node = 2;
        create_capacity_reservation_and_add_to_group(
            1,
            resources_c2(),
            Some(vec![NodeFeatures::PublicNode]),
            1,
            expected_node,
        );

        // Contract could go to node 2 but there is already a contract on that node that belongs to the same group
        // so the contract will go to node 3 which also has a public config
        let expected_node = 3;
        create_capacity_reservation_and_add_to_group(
            1,
            resources_c3(),
            Some(vec![NodeFeatures::PublicNode]),
            1,
            expected_node,
        );
    });
}

#[test]
fn test_capacity_reservation_contract_with_policy_exclusive_and_features_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_with_three_nodes();
        // node 2 has public config
        add_public_config(1, 2, alice());

        assert_ok!(SmartContractModule::group_create(RuntimeOrigin::signed(
            alice()
        )));

        // Contract should go to node 2 (enough resources + has public config) and thus bring it up
        let expected_node = 2;
        create_capacity_reservation_and_add_to_group(
            1,
            resources_c2(),
            Some(vec![NodeFeatures::PublicNode]),
            1,
            expected_node,
        );

        // Contract could go to node 2 (if we only look at resources) but the contract we want to create
        // belongs to the same group as prior contract so we can't add it on node 2. As node 2 is the only
        // node with a public config this call shoul fail
        assert_noop!(
            SmartContractModule::contract_capacity_reservation_create(
                RuntimeOrigin::signed(alice()),
                1,
                CapacityReservationPolicy::Exclusive {
                    group_id: 1,
                    resources: resources_c3(),
                    features: Some(vec![NodeFeatures::PublicNode]),
                },
                None,
            ),
            Error::<TestRuntime>::NoSuitableNodeInFarm
        );
    });
}

#[test]
fn test_capacity_reservation_contract_create_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(alice()),
            1,
            CapacityReservationPolicy::Any {
                resources: get_resources(),
                features: None,
            },
            None,
        ));

        assert_eq!(
            TfgridModule::nodes(1).unwrap().resources.used_resources,
            get_resources()
        );

        assert_eq!(
            TfgridModule::nodes(1).unwrap().power.target,
            PowerTarget::Up,
        )
    });
}

#[test]
fn test_deployment_create_with_public_ips_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_node_and_capacity_reservation();

        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(alice()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            get_resources(),
            2,
        ));

        let deployment = SmartContractModule::deployments(1).unwrap();

        let farm = TfgridModule::farms(1).unwrap();
        assert_eq!(farm.public_ips[0].contract_id, 1);

        assert_eq!(deployment.public_ips, 2);

        let pub_ip = PublicIP {
            ip: get_public_ip_ip(b"185.206.122.33/24"),
            gateway: get_public_ip_gw(b"185.206.122.1"),
            contract_id: 1,
        };

        let pub_ip_2 = PublicIP {
            ip: get_public_ip_ip(b"185.206.122.34/24"),
            gateway: get_public_ip_gw(b"185.206.122.1"),
            contract_id: 1,
        };
        assert_eq!(deployment.public_ips_list[0], pub_ip);
        assert_eq!(deployment.public_ips_list[1], pub_ip_2);
    });
}

#[test]
fn test_capacity_reservation_contract_create_with_nonexisting_farm_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_noop!(
            SmartContractModule::contract_capacity_reservation_create(
                RuntimeOrigin::signed(alice()),
                2,
                CapacityReservationPolicy::Any {
                    resources: get_resources(),
                    features: None,
                },
                None,
            ),
            Error::<TestRuntime>::FarmNotExists
        );
    });
}

#[test]
fn test_deployment_create_with_same_hash_and_node_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_node_and_capacity_reservation();

        let h = generate_deployment_hash();
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(alice()),
            1,
            h,
            get_deployment_data(),
            get_resources(),
            0,
        ));

        assert_noop!(
            SmartContractModule::deployment_create(
                RuntimeOrigin::signed(alice()),
                1,
                h,
                get_deployment_data(),
                get_resources(),
                0,
            ),
            Error::<TestRuntime>::ContractIsNotUnique
        );
    });
}

#[test]
fn test_deployment_create_which_was_canceled_before_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_node_and_capacity_reservation();

        let h = generate_deployment_hash();
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(alice()),
            1,
            h,
            get_deployment_data(),
            get_resources(),
            0,
        ));
        let contract_id = SmartContractModule::node_contract_by_hash(1, h);
        assert_eq!(contract_id, 1);

        assert_ok!(SmartContractModule::deployment_cancel(
            RuntimeOrigin::signed(alice()),
            1
        ));

        let h = generate_deployment_hash();
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(alice()),
            1,
            h,
            get_deployment_data(),
            get_resources(),
            0,
        ));
        let contract_id = SmartContractModule::node_contract_by_hash(1, h);
        assert_eq!(contract_id, 2);
    });
}

#[test]
fn test_capacity_reservation_contract_create_no_node_in_farm_with_enough_resources() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_noop!(
            SmartContractModule::contract_capacity_reservation_create(
                RuntimeOrigin::signed(alice()),
                1,
                CapacityReservationPolicy::Any {
                    resources: ResourcesInput {
                        cru: 10,
                        hru: 0,
                        mru: 2 * GIGABYTE,
                        sru: 60 * GIGABYTE
                    },
                    features: None,
                },
                None,
            ),
            Error::<TestRuntime>::NoSuitableNodeInFarm
        );
    });
}

#[test]
fn test_capacity_reservation_contract_create_finding_a_node() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);

        prepare_farm_three_nodes_three_capacity_reservation_contracts();
    });
}

#[test]
fn test_capacity_reservation_contract_create_finding_a_node_failure() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_three_nodes_three_capacity_reservation_contracts();
        // no available nodes anymore that meet the required resources
        assert_noop!(
            SmartContractModule::contract_capacity_reservation_create(
                RuntimeOrigin::signed(alice()),
                1,
                CapacityReservationPolicy::Any {
                    resources: ResourcesInput {
                        hru: 4096 * GIGABYTE,
                        sru: 2048 * GIGABYTE,
                        cru: 32,
                        mru: 48 * GIGABYTE,
                    },
                    features: None
                },
                None,
            ),
            Error::<TestRuntime>::NoSuitableNodeInFarm
        );
    });
}

#[test]
fn test_capacity_reservation_contract_create_reserving_full_node_then_deployment_contract_then_cancel_everything(
) {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_with_three_nodes();
        // node 2 should be down and when we create the capacity reservation contract the node should be woken up
        // we do not yet change the used resources until deployment contracts are created
        let node_id = 2;
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Node { node_id: node_id },
            None,
        ));
        assert_eq!(
            TfgridModule::nodes(node_id).unwrap().power.target,
            PowerTarget::Up
        );
        assert_eq!(
            TfgridModule::nodes(node_id)
                .unwrap()
                .resources
                .used_resources,
            TfgridModule::nodes(node_id)
                .unwrap()
                .resources
                .total_resources
        );
        // creating the deployment contract should claim resources from the node
        let hash = generate_deployment_hash();
        let data = get_deployment_data();
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(bob()),
            1,
            hash,
            data.clone(),
            get_resources(),
            0,
        ));
        // we expect the capacity reservation contract to look like this:
        assert_eq!(
            SmartContractModule::contracts(1).unwrap().contract_type,
            types::ContractData::CapacityReservationContract(types::CapacityReservationContract {
                group_id: None,
                public_ips: 0,
                resources: ConsumableResources {
                    total_resources: resources_n2(),
                    used_resources: get_resources(),
                },
                node_id: node_id,
                deployments: vec![1]
            })
        );
        // we expect the deployment contract to look like this:
        assert_eq!(
            SmartContractModule::deployments(1).unwrap(),
            types::Deployment {
                id: 1,
                twin_id: 2,
                capacity_reservation_id: 1,
                deployment_data: data,
                deployment_hash: hash,
                public_ips: 0,
                public_ips_list: Vec::new().try_into().unwrap(),
                resources: get_resources(),
            }
        );
        // canceling the deployment contract should unclaim the resources on that node and
        // remove the contract from the list of deployment contracts
        assert_ok!(SmartContractModule::deployment_cancel(
            RuntimeOrigin::signed(bob()),
            1
        ));
        assert_eq!(
            SmartContractModule::contracts(1).unwrap().contract_type,
            types::ContractData::CapacityReservationContract(types::CapacityReservationContract {
                group_id: None,
                public_ips: 0,
                resources: ConsumableResources {
                    total_resources: resources_n2(),
                    used_resources: ResourcesInput::empty(),
                },
                node_id: node_id,
                deployments: vec![]
            })
        );
        // canceling capacity reservation contract should shut down the node (as it is not the first in the list
        // of nodes from that farm)
        assert_ok!(SmartContractModule::contract_cancel(
            RuntimeOrigin::signed(bob()),
            1
        ));
        assert_eq!(
            TfgridModule::nodes(node_id).unwrap().power.target,
            PowerTarget::Down
        );
        assert_eq!(
            TfgridModule::nodes(node_id).unwrap().resources,
            ConsumableResources {
                total_resources: resources_n2(),
                used_resources: ResourcesInput::empty(),
            }
        );

        let our_events = System::events();
        for event in our_events.clone().iter() {
            log::info!("Event: {:?}", event);
        }
        // should have emitted one power up event and one power down
        assert_eq!(
            our_events.contains(&record(MockEvent::TfgridModule(pallet_tfgrid::Event::<
                TestRuntime,
            >::PowerTargetChanged {
                farm_id: 1,
                node_id: 2,
                power_target: PowerTarget::Up,
            }))),
            true
        );
        assert_eq!(
            our_events.contains(&record(MockEvent::TfgridModule(pallet_tfgrid::Event::<
                TestRuntime,
            >::PowerTargetChanged {
                farm_id: 1,
                node_id: 2,
                power_target: PowerTarget::Down,
            }))),
            true
        );
    });
}

#[test]
fn test_cancel_capacity_reservation_contract_should_not_shutdown_first_node() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_with_three_nodes();
        assert_eq!(
            TfgridModule::nodes(1).unwrap().power.target,
            PowerTarget::Down
        );
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Any {
                resources: resources_c1(),
                features: None,
            },
            None,
        ));
        assert_eq!(
            TfgridModule::nodes(1).unwrap().power.target,
            PowerTarget::Up
        );
        assert_ok!(SmartContractModule::contract_cancel(
            RuntimeOrigin::signed(bob()),
            1
        ));
        // node should try to shut down
        assert_eq!(
            TfgridModule::nodes(1).unwrap().power.target,
            PowerTarget::Down
        );
    });
}

#[test]
fn test_cancel_capacity_reservation_contract_shutdown_node() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_three_nodes_three_capacity_reservation_contracts();
        // node 1 => capacity contract 1 and 2
        // cancel contract 2 = nothing should change
        assert_ok!(SmartContractModule::contract_cancel(
            RuntimeOrigin::signed(alice()),
            2
        ));
        assert_eq!(
            TfgridModule::nodes(1).unwrap().power.target,
            PowerTarget::Up
        );
        assert_eq!(
            TfgridModule::nodes(2).unwrap().power.target,
            PowerTarget::Up
        );
        assert_eq!(
            TfgridModule::nodes(3).unwrap().power.target,
            PowerTarget::Down
        );
        // on node 1 there is only one contract left => used resources of node 1 should equal resources of contract 1
        assert_eq!(
            TfgridModule::nodes(1).unwrap().resources.used_resources,
            resources_c1()
        );

        // cancel contract 3 = node 2 should shutdown
        assert_ok!(SmartContractModule::contract_cancel(
            RuntimeOrigin::signed(alice()),
            3
        ));
        assert_eq!(
            TfgridModule::nodes(1).unwrap().power.target,
            PowerTarget::Up
        );
        assert_eq!(
            TfgridModule::nodes(2).unwrap().power.target,
            PowerTarget::Down
        );
        assert_eq!(
            TfgridModule::nodes(3).unwrap().power.target,
            PowerTarget::Down
        );
        // nothing else running on node 2 => used resources should be 0
        assert_eq!(
            TfgridModule::nodes(2).unwrap().resources.used_resources,
            ResourcesInput::empty()
        );

        // cancel contract 1 (last contract running on node 1) => node may not be shutdown as it is the only
        // one left running in the farm
        assert_ok!(SmartContractModule::contract_cancel(
            RuntimeOrigin::signed(alice()),
            1
        ));
        assert_eq!(
            TfgridModule::nodes(1).unwrap().power.target,
            PowerTarget::Down
        );
        assert_eq!(
            TfgridModule::nodes(2).unwrap().power.target,
            PowerTarget::Down
        );
        assert_eq!(
            TfgridModule::nodes(3).unwrap().power.target,
            PowerTarget::Down
        );
        // nothing else running on node 1 => used resources should be 0
        assert_eq!(
            TfgridModule::nodes(1).unwrap().resources.used_resources,
            ResourcesInput::empty()
        );

        // check the power target events
        let our_events = System::events();
        for event in our_events.clone().iter() {
            log::info!("Event: {:?}", event);
        }
        assert_eq!(
            our_events.contains(&record(MockEvent::TfgridModule(pallet_tfgrid::Event::<
                TestRuntime,
            >::PowerTargetChanged {
                farm_id: 1,
                node_id: 2,
                power_target: PowerTarget::Down,
            }))),
            true
        );
    });
}

#[test]
fn test_capacity_reservation_contract_update_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(alice()),
            1,
            CapacityReservationPolicy::Any {
                resources: get_resources(),
                features: None,
            },
            None,
        ));

        let updated_resources = ResourcesInput {
            cru: 1,             // decrease
            hru: 1 * GIGABYTE,  // increase
            mru: 2 * GIGABYTE,  // unmodified
            sru: 90 * GIGABYTE, // increase
        };
        assert_ok!(SmartContractModule::contract_capacity_reservation_update(
            RuntimeOrigin::signed(alice()),
            1,
            updated_resources,
        ));
        // Used resources on node should be updated!
        assert_eq!(
            TfgridModule::nodes(1).unwrap().resources.used_resources,
            updated_resources
        );
        // contract should look like this:
        let capacity_reservation_contract = types::CapacityReservationContract {
            node_id: 1,
            group_id: None,
            public_ips: 0,
            resources: ConsumableResources {
                total_resources: updated_resources,
                used_resources: ResourcesInput::empty(),
            },
            deployments: vec![],
        };
        let contract_type =
            types::ContractData::CapacityReservationContract(capacity_reservation_contract);
        let expected_contract_value = types::Contract {
            contract_id: 1,
            state: types::ContractState::Created,
            twin_id: 1,
            version: crate::CONTRACT_VERSION,
            contract_type,
            solution_provider_id: None,
        };

        let capacity_contract = SmartContractModule::contracts(1).unwrap();
        assert_eq!(capacity_contract, expected_contract_value);

        let contracts = SmartContractModule::active_node_contracts(1);
        assert_eq!(contracts.len(), 1);

        assert_eq!(contracts[0], 1);
    });
}
#[test]
fn test_capacity_reservation_contract_update_too_much_resources() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(alice()),
            1,
            CapacityReservationPolicy::Any {
                resources: get_resources(),
                features: None,
            },
            None,
        ));
        // asking for too much resources
        assert_noop!(
            SmartContractModule::contract_capacity_reservation_update(
                RuntimeOrigin::signed(alice()),
                1,
                ResourcesInput {
                    hru: 1024 * GIGABYTE,
                    sru: 512 * GIGABYTE,
                    cru: 10,
                    mru: 16 * GIGABYTE
                },
            ),
            pallet_tfgrid::Error::<TestRuntime>::NotEnoughResourcesOnNode
        );
    });
}

#[test]
fn test_capacity_reservation_contract_decrease_resources_fails_resources_used_by_active_contracts()
{
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(alice()),
            1,
            CapacityReservationPolicy::Any {
                resources: get_resources(),
                features: None,
            },
            None,
        ));
        // deployment contract using half of the resources
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(alice()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            ResourcesInput {
                cru: 1,
                hru: 0,
                mru: 1 * GIGABYTE,
                sru: 30 * GIGABYTE,
            },
            0
        ));
        // update the resources: sru is lower then what the deployment contract is using => failure
        let updated_resources = ResourcesInput {
            cru: 1,
            hru: 0,
            mru: 1 * GIGABYTE,
            sru: 20 * GIGABYTE,
        };
        assert_noop!(
            SmartContractModule::contract_capacity_reservation_update(
                RuntimeOrigin::signed(alice()),
                1,
                updated_resources,
            ),
            Error::<TestRuntime>::ResourcesUsedByActiveContracts
        );
    });
}

#[test]
fn test_capacity_reservation_contract_update_not_exists_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_noop!(
            SmartContractModule::contract_capacity_reservation_update(
                RuntimeOrigin::signed(alice()),
                1,
                get_resources()
            ),
            Error::<TestRuntime>::CapacityReservationNotExists
        );
    });
}

#[test]
fn test_capacity_reservation_contract_update_wrong_twins_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(alice()),
            1,
            CapacityReservationPolicy::Any {
                resources: get_resources(),
                features: None,
            },
            None,
        ));

        assert_noop!(
            SmartContractModule::contract_capacity_reservation_update(
                RuntimeOrigin::signed(bob()),
                1,
                ResourcesInput {
                    cru: 1,
                    hru: 0,
                    mru: 1 * GIGABYTE,
                    sru: 10 * GIGABYTE
                },
            ),
            Error::<TestRuntime>::TwinNotAuthorized
        );
    });
}

#[test]
fn test_cancel_capacity_reservation_contract_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(alice()),
            1,
            CapacityReservationPolicy::Any {
                resources: get_resources(),
                features: None,
            },
            None,
        ));

        assert_ok!(SmartContractModule::contract_cancel(
            RuntimeOrigin::signed(alice()),
            1
        ));

        assert_eq!(
            TfgridModule::nodes(1).unwrap().resources.used_resources,
            ResourcesInput::empty()
        );

        let capacity_contract = SmartContractModule::contracts(1);
        assert_eq!(capacity_contract, None);

        let contracts = SmartContractModule::active_node_contracts(1);
        assert_eq!(contracts.len(), 0);
    });
}

#[test]
fn test_deployment_cancel_contract_free_resources_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_node_and_capacity_reservation();

        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(alice()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            resources_c1(),
            0,
        ));

        assert_ok!(SmartContractModule::deployment_cancel(
            RuntimeOrigin::signed(alice()),
            1
        ));
        // used resources should be empty and deployment contracts should be an empty list
        assert_eq!(
            SmartContractModule::contracts(1).unwrap().contract_type,
            types::ContractData::CapacityReservationContract(types::CapacityReservationContract {
                node_id: 1,
                group_id: None,
                public_ips: 0,
                resources: ConsumableResources {
                    total_resources: resources_c1(),
                    used_resources: ResourcesInput::empty(),
                },
                deployments: vec![],
            })
        );
    });
}

#[test]
fn test_deployment_cancel_contract_frees_public_ips_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_node_and_capacity_reservation();
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(alice()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            resources_c1(),
            2,
        ));

        let farm = TfgridModule::farms(1).unwrap();
        assert_eq!(farm.public_ips[0].contract_id, 1);
        assert_eq!(farm.public_ips[1].contract_id, 1);

        assert_ok!(SmartContractModule::deployment_cancel(
            RuntimeOrigin::signed(alice()),
            1
        ));

        let farm = TfgridModule::farms(1).unwrap();
        assert_eq!(farm.public_ips[0].contract_id, 0);
        assert_eq!(farm.public_ips[1].contract_id, 0);
    });
}

#[test]
fn test_deployment_cancel_contract_not_exists_fails() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        assert_noop!(
            SmartContractModule::contract_cancel(RuntimeOrigin::signed(alice()), 1),
            Error::<TestRuntime>::ContractNotExists
        );
    });
}

#[test]
fn test_deployment_cancel_contract_wrong_twins_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_node_and_capacity_reservation();

        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(alice()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            resources_c1(),
            0,
        ));

        assert_noop!(
            SmartContractModule::deployment_cancel(RuntimeOrigin::signed(bob()), 1),
            Error::<TestRuntime>::TwinNotAuthorized
        );
    });
}

#[test]
fn test_deployment_update_increase_resources_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(alice()),
            1,
            CapacityReservationPolicy::Any {
                resources: resources_c1(),
                features: None,
            },
            None,
        ));
        let data = get_deployment_data();
        let hash = generate_deployment_hash();
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(alice()),
            1,
            hash,
            data.clone(),
            half_resources_c1(),
            0
        ));
        assert_eq!(
            SmartContractModule::deployments(1).unwrap(),
            types::Deployment {
                id: 1,
                twin_id: 1,
                capacity_reservation_id: 1,
                deployment_data: data,
                deployment_hash: hash,
                public_ips: 0,
                resources: half_resources_c1(),
                public_ips_list: vec![].try_into().unwrap(),
            }
        );
        let updated_data = get_updated_deployment_data();
        let updated_hash = generate_deployment_hash();
        assert_ok!(SmartContractModule::deployment_update(
            RuntimeOrigin::signed(alice()),
            1,
            updated_hash,
            updated_data.clone(),
            Some(resources_c1()),
        ));
        assert_eq!(
            SmartContractModule::deployments(1).unwrap(),
            types::Deployment {
                id: 1,
                twin_id: 1,
                capacity_reservation_id: 1,
                deployment_data: updated_data,
                deployment_hash: updated_hash,
                public_ips: 0,
                resources: resources_c1(),
                public_ips_list: vec![].try_into().unwrap(),
            }
        );
    });
}

#[test]
fn test_deployment_update_decrease_resources_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(alice()),
            1,
            CapacityReservationPolicy::Any {
                resources: resources_c1(),
                features: None,
            },
            None,
        ));
        let data = get_deployment_data();
        let hash = generate_deployment_hash();
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(alice()),
            1,
            hash,
            data.clone(),
            resources_c1(),
            0
        ));
        assert_eq!(
            SmartContractModule::deployments(1).unwrap(),
            types::Deployment {
                id: 1,
                twin_id: 1,
                capacity_reservation_id: 1,
                deployment_data: data.clone(),
                deployment_hash: hash,
                public_ips: 0,
                resources: resources_c1(),
                public_ips_list: vec![].try_into().unwrap(),
            }
        );
        assert_ok!(SmartContractModule::deployment_update(
            RuntimeOrigin::signed(alice()),
            1,
            hash,
            data.clone(),
            Some(half_resources_c1()),
        ));
        assert_eq!(
            SmartContractModule::deployments(1).unwrap(),
            types::Deployment {
                id: 1,
                twin_id: 1,
                capacity_reservation_id: 1,
                deployment_data: data,
                deployment_hash: hash,
                public_ips: 0,
                resources: half_resources_c1(),
                public_ips_list: vec![].try_into().unwrap(),
            }
        );
    });
}

#[test]
fn test_deployment_update_unauthorized_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(alice()),
            1,
            CapacityReservationPolicy::Any {
                resources: resources_c1(),
                features: None,
            },
            None,
        ));
        let data = get_deployment_data();
        let hash = generate_deployment_hash();
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(alice()),
            1,
            hash,
            data.clone(),
            half_resources_c1(),
            0
        ));
        assert_noop!(
            SmartContractModule::deployment_update(
                RuntimeOrigin::signed(bob()),
                1,
                generate_deployment_hash(),
                get_updated_deployment_data(),
                Some(resources_c1()),
            ),
            Error::<TestRuntime>::TwinNotAuthorized
        );
    });
}

#[test]
fn test_deployment_update_notenoughresourcesonnode_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(alice()),
            1,
            CapacityReservationPolicy::Any {
                resources: resources_c1(),
                features: None,
            },
            None,
        ));
        let data = get_deployment_data();
        let hash = generate_deployment_hash();
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(alice()),
            1,
            hash,
            data.clone(),
            half_resources_c1(),
            0
        ));
        assert_eq!(
            SmartContractModule::deployments(1).unwrap(),
            types::Deployment {
                id: 1,
                twin_id: 1,
                capacity_reservation_id: 1,
                deployment_data: data,
                deployment_hash: hash,
                public_ips: 0,
                resources: half_resources_c1(),
                public_ips_list: vec![].try_into().unwrap(),
            }
        );
        let updated_data = get_updated_deployment_data();
        let updated_hash = generate_deployment_hash();
        assert_noop!(
            SmartContractModule::deployment_update(
                RuntimeOrigin::signed(alice()),
                1,
                updated_hash,
                updated_data,
                Some(resources_n1()),
            ),
            Error::<TestRuntime>::NotEnoughResourcesInCapacityReservation
        );
    });
}

#[test]
fn test_update_contract_in_grace_state_fails() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(charlie()),
            1,
            CapacityReservationPolicy::Any {
                resources: resources_c1(),
                features: None,
            },
            None,
        ));
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(charlie()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            half_resources_c1(),
            0
        ));

        // cycle 1
        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

        // cycle 2
        // user does not have enough funds to pay for 2 cycles
        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 21);
        run_to_block(21, Some(&mut pool_state));

        assert_eq!(
            SmartContractModule::contracts(1).unwrap().state,
            types::ContractState::GracePeriod(21)
        );
        // assert_eq!(
        //     SmartContractModule::contracts(2).unwrap().state,
        //     types::ContractState::GracePeriod(21)
        // );
        assert_noop!(
            SmartContractModule::contract_capacity_reservation_update(
                RuntimeOrigin::signed(charlie()),
                1,
                resources_c1()
            ),
            Error::<TestRuntime>::CannotUpdateContractInGraceState
        );
        // assert_noop!(
        //     SmartContractModule::deployment_update(
        //         RuntimeOrigin::signed(charlie()),
        //         1,
        //         generate_deployment_hash(),
        //         get_updated_deployment_data(),
        //         Some(resources_n1()),
        //     ),
        //     Error::<TestRuntime>::CannotUpdateContractInGraceState
        // );
    });
}

//  NAME CONTRACT TESTS //
// -------------------- //

#[test]
fn test_create_name_contract_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::contract_name_create(
            RuntimeOrigin::signed(bob()),
            "foobar".as_bytes().to_vec()
        ));
    });
}

#[test]
fn test_cancel_name_contract_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::contract_name_create(
            RuntimeOrigin::signed(alice()),
            "some_name".as_bytes().to_vec()
        ));

        assert_ok!(SmartContractModule::contract_cancel(
            RuntimeOrigin::signed(alice()),
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
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::contract_name_create(
            RuntimeOrigin::signed(bob()),
            "foobar".as_bytes().to_vec()
        ));
        assert_noop!(
            SmartContractModule::contract_name_create(
                RuntimeOrigin::signed(alice()),
                "foobar".as_bytes().to_vec()
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

        assert_ok!(SmartContractModule::contract_name_create(
            RuntimeOrigin::signed(bob()),
            "foobar".as_bytes().to_vec()
        ));

        assert_ok!(SmartContractModule::contract_cancel(
            RuntimeOrigin::signed(bob()),
            1
        ));

        assert_ok!(SmartContractModule::contract_name_create(
            RuntimeOrigin::signed(bob()),
            "foobar".as_bytes().to_vec()
        ));
    });
}

#[test]
fn test_create_name_contract_with_invalid_dns_name_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        assert_noop!(
            SmartContractModule::contract_name_create(
                RuntimeOrigin::signed(alice()),
                "foo.bar".as_bytes().to_vec()
            ),
            Error::<TestRuntime>::NameNotValid
        );

        assert_noop!(
            SmartContractModule::contract_name_create(
                RuntimeOrigin::signed(alice()),
                "foo!".as_bytes().to_vec()
            ),
            Error::<TestRuntime>::NameNotValid
        );

        assert_noop!(
            SmartContractModule::contract_name_create(
                RuntimeOrigin::signed(alice()),
                "foo;'".as_bytes().to_vec()
            ),
            Error::<TestRuntime>::NameNotValid
        );

        assert_noop!(
            SmartContractModule::contract_name_create(
                RuntimeOrigin::signed(alice()),
                "foo123.%".as_bytes().to_vec()
            ),
            Error::<TestRuntime>::NameNotValid
        );
    });
}

//  CAPACITY CONTRACT RESERVING ALL RESOURCES OF NODE TESTS //
// -------------------------------------------- //

#[test]
fn test_capacity_reservation_contract_create_reserving_all_resources_node_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_dedicated_farm_and_node();

        let node_id = 1;
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Node { node_id: node_id },
            None,
        ));

        let contract = SmartContractModule::contracts(1).unwrap();
        let rent_contract = types::CapacityReservationContract {
            group_id: None,
            public_ips: 0,
            resources: ConsumableResources {
                total_resources: resources_n1(),
                used_resources: ResourcesInput::empty(),
            },
            deployments: vec![],
            node_id: 1,
        };
        assert_eq!(
            TfgridModule::nodes(1).unwrap().resources.used_resources,
            resources_n1()
        );
        assert_eq!(
            contract.contract_type,
            types::ContractData::CapacityReservationContract(rent_contract)
        );
    });
}

#[test]
fn test_cancel_capacity_reservation_contract_all_resources_of_node_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_dedicated_farm_and_node();

        let node_id = 1;
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Node { node_id: node_id },
            None,
        ));

        assert_eq!(
            SmartContractModule::contracts(1).unwrap().contract_type,
            types::ContractData::CapacityReservationContract(types::CapacityReservationContract {
                node_id: node_id,
                group_id: None,
                public_ips: 0,
                resources: ConsumableResources {
                    total_resources: resources_n1(),
                    used_resources: ResourcesInput::empty(),
                },
                deployments: vec![],
            })
        );

        assert_ok!(SmartContractModule::contract_cancel(
            RuntimeOrigin::signed(bob()),
            1
        ));

        let contract = SmartContractModule::contracts(1);
        assert_eq!(contract, None);
    });
}

#[test]
fn test_capacity_reservation_contract_create_reserving_all_resources_on_node_in_use_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        // Alice is reserving the node 1 for herself
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(alice()),
            1,
            CapacityReservationPolicy::Node { node_id: 1 },
            None,
        ));

        assert_noop!(
            SmartContractModule::contract_capacity_reservation_create(
                RuntimeOrigin::signed(bob()),
                1,
                CapacityReservationPolicy::Node { node_id: 1 },
                None,
            ),
            Error::<TestRuntime>::NodeNotAvailableToDeploy
        );
    })
}

#[test]
fn test_capacity_reservation_contract_non_dedicated_empty_node_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        let node_id = 1;
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(alice()),
            1,
            CapacityReservationPolicy::Node { node_id: node_id },
            None,
        ));
    })
}

#[test]
fn test_capacity_reservation_contract_create_on_dedicated_farm_without_reserving_all_resources_of_node_fails(
) {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_dedicated_farm_and_node();

        assert_noop!(
            SmartContractModule::contract_capacity_reservation_create(
                RuntimeOrigin::signed(bob()),
                1,
                CapacityReservationPolicy::Any {
                    resources: resources_c1(), // not requesting the all the resources of the node should not be possible for dedicated farms!
                    features: None,
                },
                None,
            ),
            Error::<TestRuntime>::NodeNotAvailableToDeploy
        );
    })
}

#[test]
fn test_deployment_create_when_having_a_capacity_reservation_reserving_all_resources_of_node_works()
{
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_dedicated_farm_and_node();

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Node { node_id: 1 },
            None,
        ));

        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            get_resources(),
            1,
        ));
    })
}

#[test]
fn test_deployment_create_using_someone_elses_capacity_reservation_contract_fails() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_dedicated_farm_and_node();

        // create capacity reservation contract with bob
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Node { node_id: 1 },
            None,
        ));
        // Alice not the owner of the capacity reservation contract so she is unauthorized to deploy a deployment contract
        assert_noop!(
            SmartContractModule::deployment_create(
                RuntimeOrigin::signed(alice()),
                1,
                generate_deployment_hash(),
                get_deployment_data(),
                get_resources(),
                1,
            ),
            Error::<TestRuntime>::TwinNotAuthorized
        );
    })
}

#[test]
fn test_cancel_capacity_reservation_contract_with_active_deployment_contracts_fails() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Node { node_id: node_id },
            None,
        ));
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            get_resources(),
            0,
        ));

        assert_noop!(
            SmartContractModule::contract_cancel(RuntimeOrigin::signed(bob()), 1,),
            Error::<TestRuntime>::CapacityReservationHasActiveContracts
        );
        // node 1 should still be up after failed attempt to cancel capacity contract
        assert_eq!(
            TfgridModule::nodes(1).unwrap().power.target,
            PowerTarget::Up
        );
    });
}

//  CONTRACT BILLING TESTS //
// ----------------------- //

#[test]
fn test_deployment_contract_billing_details() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        let twin = TfgridModule::twins(2).unwrap();
        let initial_twin_balance = Balances::free_balance(&twin.account_id);

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Any {
                resources: resources_c1(),
                features: None,
            },
            None,
        ));
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            get_resources(),
            1,
        ));

        push_nru_report_for_contract(1, 10);

        let contract_to_bill = SmartContractModule::contract_to_bill_at_block(1);
        assert_eq!(contract_to_bill, [1]);

        let initial_total_issuance = Balances::total_issuance();
        // advance 25 cycles
        for i in 0..25 {
            pool_state
                .write()
                .should_call_bill_contract(1, Ok(Pays::Yes.into()), 11 + i * 10);
            run_to_block(11 + i * 10, Some(&mut pool_state));
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
        let contract_billing_info = SmartContractModule::contract_billing_information_by_id(1);
        assert_eq!(contract_billing_info.amount_unbilled, 0);
    });
}

#[test]
fn test_deployment_contract_billing_details_with_solution_provider() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_farm_and_node();

        prepare_solution_provider();

        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        let twin = TfgridModule::twins(2).unwrap();
        let initial_twin_balance = Balances::free_balance(&twin.account_id);
        let initial_total_issuance = Balances::total_issuance();

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Any {
                resources: resources_c1(),
                features: None,
            },
            Some(1),
        ));
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            get_resources(),
            1,
        ));

        push_nru_report_for_contract(1, 10);

        let contract_to_bill = SmartContractModule::contract_to_bill_at_block(1);
        assert_eq!(contract_to_bill, [1]);

        // advance 25 cycles
        for i in 0..25 {
            pool_state
                .write()
                .should_call_bill_contract(1, Ok(Pays::Yes.into()), 11 + i * 10);
            run_to_block(11 + i * 10, Some(&mut pool_state));
        }

        let free_balance = Balances::free_balance(&twin.account_id);
        let total_amount_billed = initial_twin_balance - free_balance;

        validate_distribution_rewards(initial_total_issuance, total_amount_billed, true);

        // amount unbilled should have been reset after a transfer between contract owner and farmer
        let contract_billing_info = SmartContractModule::contract_billing_information_by_id(1);
        assert_eq!(contract_billing_info.amount_unbilled, 0);
    });
}

#[test]
fn test_multiple_contracts_billing_loop_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Any {
                resources: resources_c1(),
                features: None,
            },
            None,
        ));
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            get_resources(),
            1,
        ));
        assert_ok!(SmartContractModule::contract_name_create(
            RuntimeOrigin::signed(bob()),
            "some_name".as_bytes().to_vec(),
        ));

        let contracts_to_bill_at_block = SmartContractModule::contract_to_bill_at_block(1);
        assert_eq!(contracts_to_bill_at_block.len(), 2);

        // 3 contracts => 2 billings (capacity reservation and name contract)
        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 11);
        pool_state
            .write()
            .should_call_bill_contract(2, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

        // Test that the expected events were emitted
        let our_events = System::events();
        for event in our_events.clone().iter() {
            info!("{:?}", event);
        }
        // PriceStored
        // AveragePriceStored
        // PowerTargetChanged
        // Node updated
        // Contract Created (capacity reservation)
        // Contract Updated (capacity reservation)
        // Contract Created (deployment contract)
        // Contract Created (name contract)
        // Contract Billed (capacity contract)
        // Contract Billed (name contract)
        assert_eq!(our_events.len(), 10);
    })
}

#[test]
fn test_deployment_contract_billing_cycles() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Any {
                resources: resources_c1(),
                features: None,
            },
            None,
        ));

        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            get_resources(),
            0,
        ));
        let contract_id = 1;
        let twin_id = 2;

        let (amount_due_1, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 11);
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
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 21);
        run_to_block(21, Some(&mut pool_state));
        check_report_cost(1, amount_due_2, 21, discount_received);

        let (amount_due_3, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 31);
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
        prepare_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();
        // CAPACITY RESERVATION 1 with 2 deployment contracts
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Any {
                resources: resources_c1(),
                features: None,
            },
            None,
        ));
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            half_resources_c1(),
            0,
        ));
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            half_resources_c1(),
            0,
        ));
        // CAPACITY RESERVATION 2 with 1 deployment contract
        let rest_of_the_resources_on_node_1 =
            ResourcesInput::subtraction(&resources_n1(), &resources_c1());
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Any {
                resources: rest_of_the_resources_on_node_1.clone(),
                features: None,
            },
            None,
        ));
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(bob()),
            2,
            generate_deployment_hash(),
            get_deployment_data(),
            rest_of_the_resources_on_node_1,
            0,
        ));

        let twin_id = 2;
        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 11);
        pool_state
            .write()
            .should_call_bill_contract(2, Ok(Pays::Yes.into()), 11);

        let (cost_1st_capacity_reservation, discount_1) = calculate_tft_cost(1, twin_id, 11);
        let (cost_2nd_capacity_reservation, discount_2) = calculate_tft_cost(2, twin_id, 11);
        run_to_block(12, Some(&mut pool_state));
        check_report_cost(1, cost_1st_capacity_reservation, 12, discount_1);
        check_report_cost(2, cost_2nd_capacity_reservation, 12, discount_2);

        let twin = TfgridModule::twins(twin_id).unwrap();
        let usable_balance = Balances::usable_balance(&twin.account_id);
        let free_balance = Balances::free_balance(&twin.account_id);

        let locked_balance = free_balance - usable_balance;
        assert_eq!(
            locked_balance.saturated_into::<u128>(),
            cost_1st_capacity_reservation as u128 + cost_2nd_capacity_reservation as u128
        );
    });
}

#[test]
fn test_deployment_contract_billing_cycles_delete_node_cancels_contract() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Any {
                resources: resources_c1(),
                features: None,
            },
            None
        ));
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            get_resources(),
            1,
        ));
        let contract_id = 1;
        let twin_id = 2;

        for i in 0..5 {
            pool_state
                .write()
                .should_call_bill_contract(1, Ok(Pays::Yes.into()), 11 + i * 10);
        }

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
            ip: get_public_ip_ip(b"185.206.122.33/24"),
            gateway: get_public_ip_gw(b"185.206.122.1"),
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
                SmartContractEvent::<TestRuntime>::DeploymentCanceled {
                    deployment_id: 1,
                    capacity_reservation_id: 1,
                    node_id: 1,
                    twin_id: 2,
                }
            ))),
            true
        );
        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::CapacityReservationContractCanceled {
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
fn test_deployment_contract_only_public_ip_billing_cycles() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Any {
                resources: ResourcesInput::empty(), // no resources required
                features: None,
            },
            None
        ));
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            ResourcesInput::empty(),
            1,
        ));
        let contract_id = 1;
        let twin_id = 2;

        for i in 0..5 {
            pool_state.write().should_call_bill_contract(
                contract_id,
                Ok(Pays::Yes.into()),
                11 + i * 10,
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
fn test_deployment_contract_billing_cycles_cancel_contract_during_cycle_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Any {
                resources: resources_c1(),
                features: None,
            },
            None,
        ));
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            get_resources(),
            0,
        ));
        // only capacity reservation contract should be billed
        let contract_id = 1;
        let twin_id = 2;

        // 2 cycles for billing
        for i in 0..2 {
            pool_state.write().should_call_bill_contract(
                contract_id,
                Ok(Pays::Yes.into()),
                11 + i * 10,
            );
        }

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(11, Some(&mut pool_state));
        check_report_cost(1, amount_due_as_u128, 11, discount_received);

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 10);
        run_to_block(21, Some(&mut pool_state));
        check_report_cost(1, amount_due_as_u128, 21, discount_received);

        run_to_block(28, Some(&mut pool_state));
        let (amount_due_as_u128, discount_received) = calculate_tft_cost(contract_id, twin_id, 7);
        // cancel deployment contract then capacity reservation
        assert_ok!(SmartContractModule::deployment_cancel(
            RuntimeOrigin::signed(bob()),
            1
        ));
        assert_ok!(SmartContractModule::contract_cancel(
            RuntimeOrigin::signed(bob()),
            1
        ));

        run_to_block(29, Some(&mut pool_state));
        check_report_cost(1, amount_due_as_u128, 28, discount_received);

        let contract = SmartContractModule::contracts(1);
        assert_eq!(contract, None);

        let billing_info = SmartContractModule::contract_billing_information_by_id(1);
        assert_eq!(billing_info.amount_unbilled, 0);
    });
}

#[test]
fn test_deployment_contract_billing_fails() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        run_to_block(1, Some(&mut pool_state));
        // Creates a farm and node and sets the price of tft to 0 which raises an error later
        prepare_farm_and_node();

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Any {
                resources: resources_c1(),
                features: None,
            },
            None,
        ));
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            get_resources(),
            1,
        ));

        let contracts_to_bill_at_block = SmartContractModule::contract_to_bill_at_block(1);
        assert_eq!(contracts_to_bill_at_block.len(), 1);

        let contract_id = contracts_to_bill_at_block[0];

        // delete twin to make the billing fail
        assert_ok!(TfgridModule::delete_twin(
            RuntimeOrigin::signed(bob()),
            SmartContractModule::contracts(contract_id).unwrap().twin_id,
        ));

        // the offchain worker should save the failed ids in local storage and try again
        // in subsequent blocks (which will also fail)
        for i in 1..3 {
            pool_state.write().should_call_bill_contract(
                1,
                Err(Error::<TestRuntime>::TwinNotExists.into()),
                1 + i * 10,
            );
            run_to_block(11 * i, Some(&mut pool_state));
        }
    });
}

#[test]
fn test_deployment_contract_billing_cycles_cancel_contract_during_cycle_without_balance_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        let twin = TfgridModule::twins(2).unwrap();
        let initial_twin_balance = Balances::free_balance(&twin.account_id);
        info!("initial twin balance: {:?}", initial_twin_balance);
        let initial_total_issuance = Balances::total_issuance();
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Any {
                resources: resources_c1(),
                features: None,
            },
            None,
        ));

        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            get_resources(),
            0,
        ));

        // contract id 1 is our capacity reservation contract
        let contract_id = 1;
        let twin_id = 2;

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

        // cancel deployment contract
        assert_ok!(SmartContractModule::deployment_cancel(
            RuntimeOrigin::signed(bob()),
            1
        ));
        // cancel capacity reservation
        assert_ok!(SmartContractModule::contract_cancel(
            RuntimeOrigin::signed(bob()),
            1
        ));

        pool_state
            .write()
            .should_call_bill_contract(contract_id, Ok(Pays::Yes.into()), 31);
        run_to_block(31, Some(&mut pool_state));

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
fn test_deployment_contract_out_of_funds_should_move_state_to_graceperiod_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(charlie()),
            1,
            CapacityReservationPolicy::Any {
                resources: resources_c1(),
                features: None,
            },
            None
        ));
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(charlie()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            get_resources(),
            0,
        ));

        // cycle 1
        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

        // cycle 2
        // user does not have enough funds to pay for 2 cycles
        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 21);
        run_to_block(21, Some(&mut pool_state));

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
fn test_restore_deployment_contract_in_grace_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(charlie()),
            1,
            CapacityReservationPolicy::Any {
                resources: resources_c1(),
                features: None,
            },
            None,
        ));
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(charlie()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            get_resources(),
            0,
        ));

        for i in 0..6 {
            pool_state
                .write()
                .should_call_bill_contract(1, Ok(Pays::Yes.into()), 11 + i * 10);
        }

        // cycle 1
        run_to_block(11, Some(&mut pool_state));

        // cycle 2
        // user does not have enough funds to pay for 2 cycles
        run_to_block(21, Some(&mut pool_state));

        let c1 = SmartContractModule::contracts(1).unwrap();
        assert_eq!(c1.state, types::ContractState::GracePeriod(21));

        // resources should still be reserved
        assert_eq!(
            SmartContractModule::contracts(1).unwrap().contract_type,
            types::ContractData::CapacityReservationContract(types::CapacityReservationContract {
                node_id: 1,
                public_ips: 0,
                deployments: vec![1],
                group_id: None,
                resources: ConsumableResources {
                    total_resources: resources_c1(),
                    used_resources: get_resources(),
                },
            })
        );
        assert_eq!(
            TfgridModule::nodes(1).unwrap().resources.used_resources,
            resources_c1()
        );
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

        run_to_block(31, Some(&mut pool_state));
        run_to_block(41, Some(&mut pool_state));
        // Transfer some balance to the owner of the contract to trigger the grace period to stop
        Balances::transfer(RuntimeOrigin::signed(bob()), charlie(), 100000000).unwrap();
        run_to_block(52, Some(&mut pool_state));
        run_to_block(62, Some(&mut pool_state));

        let c1 = SmartContractModule::contracts(1).unwrap();
        assert_eq!(c1.state, types::ContractState::Created);
    });
}

#[test]
fn test_deployment_contract_grace_period_cancels_contract_when_grace_period_ends_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();
        let twin = TfgridModule::twins(3).unwrap();
        let initial_total_issuance = Balances::total_issuance();
        let initial_twin_balance = Balances::free_balance(&twin.account_id);

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(charlie()),
            1,
            CapacityReservationPolicy::Any {
                resources: resources_c1(),
                features: None,
            },
            None,
        ));
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(charlie()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            get_resources(),
            0,
        ));

        // cycle 1
        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

        // cycle 2
        // user does not have enough funds to pay for 2 cycles
        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 21);
        run_to_block(21, Some(&mut pool_state));

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

        // grace period stops after 100 blocknumbers, so after 121
        for i in 1..11 {
            pool_state
                .write()
                .should_call_bill_contract(1, Ok(Pays::Yes.into()), 21 + i * 10);
        }

        for i in 1..11 {
            run_to_block(21 + i * 10, Some(&mut pool_state));
        }

        // pool_state
        //     .write()
        //     .should_call_bill_contract(1, Ok(Pays::Yes.into()), 131);
        // run_to_block(131, Some(&mut pool_state));

        // The user's total free balance should be distributed
        let free_balance = Balances::free_balance(&twin.account_id);
        let total_amount_billed = initial_twin_balance - free_balance;

        validate_distribution_rewards(initial_total_issuance, total_amount_billed, false);

        let c1 = SmartContractModule::contracts(1);
        assert_eq!(c1, None);
    });
}

#[test]
fn test_name_contract_billing() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        assert_ok!(SmartContractModule::contract_name_create(
            RuntimeOrigin::signed(bob()),
            "foobar".as_bytes().to_vec()
        ));

        let contracts_to_bill = SmartContractModule::contract_to_bill_at_block(1);
        assert_eq!(contracts_to_bill, [1]);

        // let mature 11 blocks
        // because we bill every 10 blocks
        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

        // the contractbill event should look like:
        let contract_bill_event = types::ContractBill {
            contract_id: 1,
            timestamp: 1628082066,
            discount_level: types::DiscountLevel::Gold,
            amount_billed: 1848,
        };
        let our_events = System::events();
        info!("events: {:?}", our_events.clone());
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
fn test_capacity_reservation_contract_full_node_billing() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Node { node_id: node_id },
            None,
        ));

        assert_eq!(
            SmartContractModule::contracts(1).unwrap().contract_type,
            types::ContractData::CapacityReservationContract(types::CapacityReservationContract {
                node_id: node_id,
                public_ips: 0,
                deployments: vec![],
                group_id: None,
                resources: ConsumableResources {
                    total_resources: resources_n1(),
                    used_resources: ResourcesInput::empty()
                },
            })
        );

        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(1, 2, 10);
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(1, amount_due_as_u128, 11, discount_received);
    });
}

#[test]
fn test_capacity_reservation_contract_full_node_billing_cancel_should_bill_reserved_balance() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Node { node_id: node_id },
            None,
        ));

        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(1, 2, 10);
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(1, amount_due_as_u128, 11, discount_received.clone());

        let twin = TfgridModule::twins(2).unwrap();
        let usable_balance = Balances::usable_balance(&twin.account_id);
        let free_balance = Balances::free_balance(&twin.account_id);
        assert_ne!(usable_balance, free_balance);

        run_to_block(13, Some(&mut pool_state));
        // cancel contract
        // it will bill before removing the contract and it should bill all
        // reserverd balance
        let (amount_due_as_u128, discount_received) = calculate_tft_cost(1, 2, 2);
        assert_ok!(SmartContractModule::contract_cancel(
            RuntimeOrigin::signed(bob()),
            1
        ));

        let twin = TfgridModule::twins(2).unwrap();
        let usable_balance = Balances::usable_balance(&twin.account_id);
        assert_ne!(usable_balance, 0);
        Balances::transfer(RuntimeOrigin::signed(bob()), alice(), usable_balance).unwrap();

        // we do not call bill contract here as the contract is removed during
        // cancel_contract. The contract id will still be in ContractsToBillAt
        // but the contract itself will no longer exist
        // But the
        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 21);
        run_to_block(22, Some(&mut pool_state));

        // Last amount due is the same as the first one
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(1, amount_due_as_u128, 13, discount_received);

        let usable_balance = Balances::usable_balance(&twin.account_id);
        let free_balance = Balances::free_balance(&twin.account_id);
        assert_eq!(usable_balance, free_balance);
    });
}

#[test]
fn test_capacity_reservation_contract_full_node_canceled_mid_cycle_should_bill_for_remainder() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Node { node_id: node_id },
            None,
        ));

        let twin = TfgridModule::twins(2).unwrap();
        let usable_balance = Balances::usable_balance(&twin.account_id);
        let free_balance = Balances::free_balance(&twin.account_id);

        let locked_balance = free_balance - usable_balance;
        info!("locked balance: {:?}", locked_balance);

        run_to_block(8, Some(&mut pool_state));
        // Calculate the cost for 7 blocks of runtime (created a block 1, canceled at block 8)
        let (amount_due_as_u128, discount_received) = calculate_tft_cost(1, 2, 7);
        // cancel rent contract at block 8
        assert_ok!(SmartContractModule::contract_cancel(
            RuntimeOrigin::signed(bob()),
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
fn test_create_capacity_contract_full_node_and_deployment_contract_should_bill_full_node_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Node { node_id: node_id },
            None,
        ));

        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            get_resources(),
            0,
        ));
        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

        let (amount_due_as_u128, discount_received) = calculate_tft_cost(1, 2, 10);
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(1, amount_due_as_u128, 11, discount_received);

        let our_events = System::events();
        // PriceStored
        // AveragePriceStored
        // PowerTargetChanged
        // NodeUpdated
        // CapacityReservation Contract created
        // ContractUpdated (capacity reservation)
        // Deployment Contract created
        // ContractBilled (capacity reservation)
        for e in our_events.clone().iter() {
            log::info!("{:?}", e);
        }
        assert_eq!(our_events.len(), 8);
    });
}

#[test]
fn test_create_capacity_contract_full_node_canceled_due_to_out_of_funds_should_deployment_cancel_contracts_works(
) {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(charlie()),
            1,
            CapacityReservationPolicy::Node { node_id: node_id },
            None,
        ));

        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(charlie()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            get_resources(),
            0,
        ));

        // run 12 cycles, contracts should cancel after 11 due to lack of funds
        for i in 0..11 {
            pool_state
                .write()
                .should_call_bill_contract(1, Ok(Pays::Yes.into()), 11 + i * 10);
        }
        for i in 0..11 {
            run_to_block(12 + 10 * i, Some(&mut pool_state));
        }

        // let (amount_due_as_u128, discount_received) = calculate_tft_cost(1, 2, 11);
        // assert_ne!(amount_due_as_u128, 0);
        // check_report_cost(1, 3, amount_due_as_u128, 12, discount_received);

        let our_events = System::events();
        for e in our_events.clone().iter() {
            info!("event: {:?}", e);
        }
        // PriceStored
        // AveragePriceStored
        // PowerTargetChanged
        // NodeUpdated
        // ContractCreated (Capacity Reservation)
        // ContractUpdated (Capacity Reservation)
        // ContractCreated (Deployment Contract)
        // ContractGracePeriodStarted
        // ContractBilled (Capacity Reservation)
        // ContractUpdated (Capacity Reservation)
        // DeploymentCanceled
        // PowerTargetChanged
        // NodeUpdated
        // CapacityReservationContractCanceled
        assert_eq!(our_events.len(), 15);

        assert_eq!(
            our_events[7],
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
            our_events[11],
            record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::DeploymentCanceled {
                deployment_id: 1,
                capacity_reservation_id: 1,
                node_id: 1,
                twin_id: 3,
            }))
        );
        assert_eq!(
            our_events[12],
            record(MockEvent::TfgridModule(
                pallet_tfgrid::Event::<TestRuntime>::PowerTargetChanged {
                    farm_id: 1,
                    node_id: 1,
                    power_target: PowerTarget::Down,
                }
            ))
        );
        assert_eq!(
            our_events[14],
            record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::CapacityReservationContractCanceled {
                    contract_id: 1,
                    node_id: 1,
                    twin_id: 3
                }
            ))
        );
    });
}

#[test]
fn test_capacity_reservation_contract_create_and_deployment_contract_with_ip_billing_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Node { node_id: node_id },
            None,
        ));

        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            get_resources(),
            1,
        ));

        // 2 contracts => we expect 2 calls to bill_contract
        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

        // check contract 1 costs (Capacity Reservation Contract)
        let (amount_due_as_u128, discount_received) = calculate_tft_cost(1, 2, 10);
        assert_ne!(amount_due_as_u128, 0);
        check_report_cost(1, amount_due_as_u128, 11, discount_received);
        let our_events = System::events();
        for event in our_events.clone().iter() {
            info!("{:?}", event);
        }
        // Price Stored
        // Avg price stored
        // PowerTargetChanged
        // NodeUpdated
        // Capacity Reservation contract created
        // Contract Updated (capacity reservation)
        // Deployment Contract created
        // Capacity Reservation contract billed
        assert_eq!(our_events.len(), 8);
    });
}

#[test]
fn test_capacity_reservation_contract_full_node_out_of_funds_should_move_state_to_graceperiod_works(
) {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(charlie()),
            1,
            CapacityReservationPolicy::Node { node_id: node_id },
            None,
        ));

        // cycle 1
        // user does not have enough funds to pay for 1 cycle
        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

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
fn test_restore_capacity_reservation_contract_in_grace_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(charlie()),
            1,
            CapacityReservationPolicy::Node { node_id: node_id },
            None,
        ));

        // cycle 1
        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

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

        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 21);
        run_to_block(21, Some(&mut pool_state));

        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 31);
        run_to_block(31, Some(&mut pool_state));

        // Transfer some balance to the owner of the contract to trigger the grace period to stop
        Balances::transfer(RuntimeOrigin::signed(bob()), charlie(), 100000000).unwrap();

        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 41);
        run_to_block(41, Some(&mut pool_state));

        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 51);
        run_to_block(51, Some(&mut pool_state));

        let c1 = SmartContractModule::contracts(1).unwrap();
        assert_eq!(c1.state, types::ContractState::Created);
    });
}

#[test]
fn test_restore_capacity_reservation_contract_and_deployment_contracts_in_grace_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1, None);
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(charlie()),
            1,
            CapacityReservationPolicy::Node { node_id: node_id },
            None,
        ));
        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(charlie()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            get_resources(),
            0,
        ));

        // cycle 1
        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

        let c1 = SmartContractModule::contracts(1).unwrap();
        assert_eq!(c1.state, types::ContractState::GracePeriod(11));

        let our_events = System::events();
        assert_eq!(
            our_events[7],
            record(MockEvent::SmartContractModule(SmartContractEvent::<
                TestRuntime,
            >::ContractGracePeriodStarted {
                contract_id: 1,
                node_id: 1,
                twin_id: 3,
                block_number: 11
            }))
        );

        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 21);
        run_to_block(22, Some(&mut pool_state));

        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 31);
        run_to_block(32, Some(&mut pool_state));

        // Transfer some balance to the owner of the contract to trigger the grace period to stop
        Balances::transfer(RuntimeOrigin::signed(bob()), charlie(), 100000000).unwrap();

        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 41);
        run_to_block(42, Some(&mut pool_state));

        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 51);
        run_to_block(52, Some(&mut pool_state));

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
    });
}

#[test]
fn test_capacity_reservation_contract_grace_period_cancels_contract_when_grace_period_ends_works() {
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(charlie()),
            1,
            CapacityReservationPolicy::Node { node_id: node_id },
            None,
        ));

        // cycle 1
        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

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

        // run 12 cycles, after 10 cycles grace period has finished so no more
        // billing!
        for i in 0..11 {
            pool_state
                .write()
                .should_call_bill_contract(1, Ok(Pays::Yes.into()), 21 + i * 10);
        }
        for i in 0..12 {
            run_to_block(21 + i * 10, Some(&mut pool_state));
        }

        let c1 = SmartContractModule::contracts(1);
        assert_eq!(c1, None);
    });
}

#[test]
fn test_capacity_reservation_contract_and_deployment_contract_canceled_when_node_is_deleted_works()
{
    let (mut ext, mut pool_state) = new_test_ext_with_pool_state(0);
    ext.execute_with(|| {
        prepare_dedicated_farm_and_node();
        run_to_block(1, Some(&mut pool_state));
        TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

        let node_id = 1;
        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(bob()),
            1,
            CapacityReservationPolicy::Node { node_id: node_id },
            None,
        ));

        assert_ok!(SmartContractModule::deployment_create(
            RuntimeOrigin::signed(bob()),
            1,
            generate_deployment_hash(),
            get_deployment_data(),
            get_resources(),
            0,
        ));

        // 2 contracts => 2 calls to bill_contract
        pool_state
            .write()
            .should_call_bill_contract(1, Ok(Pays::Yes.into()), 11);
        run_to_block(11, Some(&mut pool_state));

        run_to_block(16, Some(&mut pool_state));

        // Delete node
        TfgridModule::delete_node_farm(RuntimeOrigin::signed(alice()), 1).unwrap();

        let our_events = System::events();

        let ip = "1.1.1.0".as_bytes().to_vec();
        let mut ips = Vec::new();
        ips.push(ip);

        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::CapacityReservationContractCanceled {
                    contract_id: 1,
                    node_id: 1,
                    twin_id: 2
                }
            ))),
            true
        );
        assert_eq!(
            our_events.contains(&record(MockEvent::SmartContractModule(
                SmartContractEvent::<TestRuntime>::DeploymentCanceled {
                    deployment_id: 1,
                    node_id: 1,
                    capacity_reservation_id: 1,
                    twin_id: 2,
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

        assert_ok!(SmartContractModule::solution_provider_create(
            RuntimeOrigin::signed(alice()),
            "some_description".as_bytes().to_vec(),
            "some_link".as_bytes().to_vec(),
            providers
        ));

        assert_ok!(SmartContractModule::solution_provider_approve(
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
            SmartContractModule::solution_provider_create(
                RuntimeOrigin::signed(alice()),
                "some_description".as_bytes().to_vec(),
                "some_link".as_bytes().to_vec(),
                providers
            ),
            Error::<TestRuntime>::InvalidProviderConfiguration
        );
    })
}

#[test]
fn test_capacity_reservation_contract_create_with_solution_provider_works() {
    new_test_ext().execute_with(|| {
        run_to_block(1, None);
        prepare_farm_and_node();

        prepare_solution_provider();

        assert_ok!(SmartContractModule::contract_capacity_reservation_create(
            RuntimeOrigin::signed(alice()),
            1,
            CapacityReservationPolicy::Node { node_id: 1 },
            Some(1),
        ));
    });
}

#[test]
fn test_capacity_reservation_contract_create_with_solution_provider_fails_if_not_approved() {
    new_test_ext().execute_with(|| {
        prepare_farm_and_node();

        let provider = super::types::Provider {
            take: 10,
            who: alice(),
        };
        let providers = vec![provider];

        assert_ok!(SmartContractModule::solution_provider_create(
            RuntimeOrigin::signed(alice()),
            "some_description".as_bytes().to_vec(),
            "some_link".as_bytes().to_vec(),
            providers
        ));

        assert_noop!(
            SmartContractModule::contract_capacity_reservation_create(
                RuntimeOrigin::signed(alice()),
                1,
                CapacityReservationPolicy::Node { node_id: 1 },
                Some(1),
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
        RuntimeOrigin::signed(alice()),
        consumption_reports
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
    TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();

    create_farming_policies();
    prepare_twins();
    prepare_farm(alice(), false);

    let location = LocationInput {
        city: get_city_name_input(b"Ghent"),
        country: get_country_name_input(b"Belgium"),
        latitude: get_latitude_input(b"12.233213231"),
        longitude: get_longitude_input(b"32.323112123"),
    };

    TfgridModule::create_node(
        RuntimeOrigin::signed(alice()),
        1,
        resources_n1(),
        location,
        bounded_vec![],
        false,
        false,
        None,
    )
    .unwrap();
    assert_eq!(
        TfgridModule::nodes(1).unwrap().resources,
        ConsumableResources {
            total_resources: resources_n1(),
            used_resources: ResourcesInput::empty(),
        }
    );
}

pub fn prepare_farm_node_and_capacity_reservation() {
    prepare_farm_and_node();

    assert_ok!(SmartContractModule::contract_capacity_reservation_create(
        RuntimeOrigin::signed(alice()),
        1,
        CapacityReservationPolicy::Any {
            resources: resources_c1(),
            features: None,
        },
        None,
    ));
    assert_eq!(
        TfgridModule::nodes(1).unwrap().resources,
        ConsumableResources {
            total_resources: resources_n1(),
            used_resources: resources_c1(),
        }
    );
}

pub fn add_public_config(farm_id: u32, node_id: u32, account_id: AccountId) {
    let ipv4 = get_pub_config_ip4_input(b"185.206.122.33/24");
    let ipv6 = get_pub_config_ip6_input(b"2a10:b600:1::0cc4:7a30:65b5/64");
    let gw4 = get_pub_config_gw4_input(b"185.206.122.1");
    let gw6 = get_pub_config_gw6_input(b"2a10:b600:1::1");

    let pub_config = PublicConfig {
        ip4: IP {
            ip: ipv4.clone(),
            gw: gw4.clone(),
        },
        ip6: Some(IP {
            ip: ipv6.clone(),
            gw: gw6.clone(),
        }),
        domain: Some("some-domain".as_bytes().to_vec().try_into().unwrap()),
    };

    assert_ok!(TfgridModule::add_node_public_config(
        RuntimeOrigin::signed(account_id),
        farm_id,
        node_id,
        Some(pub_config.clone())
    ));
    assert_eq!(
        TfgridModule::nodes(node_id)
            .unwrap()
            .public_config
            .is_some(),
        true
    );
}

pub fn prepare_farm_with_three_nodes() {
    prepare_farm_and_node();

    // SECOND NODE
    let location = LocationInput {
        city: get_city_name_input(b"Belgium"),
        country: get_country_name_input(b"Ghent"),
        latitude: get_latitude_input(b"41.323112123"),
        longitude: get_longitude_input(b"45.233213231"),
    };

    TfgridModule::create_node(
        RuntimeOrigin::signed(bob()),
        1,
        resources_n2(),
        location,
        bounded_vec![],
        false,
        false,
        None,
    )
    .unwrap();
    assert_eq!(
        TfgridModule::nodes(2).unwrap().resources,
        ConsumableResources {
            total_resources: resources_n2(),
            used_resources: ResourcesInput::empty(),
        }
    );

    // THIRD NODE
    let location = LocationInput {
        city: get_city_name_input(b"Belgium"),
        country: get_country_name_input(b"Ghent"),
        latitude: get_latitude_input(b"24.323112123"),
        longitude: get_longitude_input(b"64.233213231"),
    };
    TfgridModule::create_node(
        RuntimeOrigin::signed(charlie()),
        1,
        resources_n3(),
        location,
        bounded_vec![],
        false,
        false,
        None,
    )
    .unwrap();
    assert_eq!(
        TfgridModule::nodes(3).unwrap().resources,
        ConsumableResources {
            total_resources: resources_n3(),
            used_resources: ResourcesInput::empty(),
        }
    );

    let nodes_from_farm = TfgridModule::nodes_by_farm_id(1);
    // when creating a node it should try to go down
    assert_eq!(nodes_from_farm.len(), 3);
    assert_eq!(
        TfgridModule::nodes(1).unwrap().power.target,
        PowerTarget::Down
    );
    assert_eq!(
        TfgridModule::nodes(2).unwrap().power.target,
        PowerTarget::Down
    );
    assert_eq!(
        TfgridModule::nodes(3).unwrap().power.target,
        PowerTarget::Down
    );
}

pub fn prepare_dedicated_farm_and_node() {
    TFTPriceModule::set_prices(RuntimeOrigin::signed(bob()), 50, 101).unwrap();
    create_farming_policies();
    prepare_twins();
    prepare_farm(alice(), true);

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
        resources_n1(),
        location,
        bounded_vec![],
        false,
        false,
        None,
    )
    .unwrap();
}

pub fn create_capacity_reservation_and_add_to_group(
    farm_id: u32,
    resources: ResourcesInput,
    features: Option<Vec<NodeFeatures>>,
    group_id: u32,
    expected_node_id: u32,
) {
    let cnt_members_before = SmartContractModule::groups(group_id)
        .unwrap()
        .capacity_reservation_contract_ids
        .len();
    let cnt_contracts = SmartContractModule::contract_id();
    assert_ok!(SmartContractModule::contract_capacity_reservation_create(
        RuntimeOrigin::signed(alice()),
        farm_id,
        CapacityReservationPolicy::Exclusive {
            group_id: group_id,
            resources: resources,
            features: features,
        },
        None,
    ));

    assert_eq!(
        SmartContractModule::contracts(cnt_contracts + 1)
            .unwrap()
            .contract_type,
        types::ContractData::CapacityReservationContract(types::CapacityReservationContract {
            node_id: expected_node_id,
            group_id: Some(group_id),
            public_ips: 0,
            resources: ConsumableResources {
                total_resources: resources,
                used_resources: ResourcesInput::empty(),
            },
            deployments: vec![],
        })
    );

    assert_eq!(
        TfgridModule::nodes(expected_node_id).unwrap().power.target,
        PowerTarget::Up
    );

    let group = SmartContractModule::groups(group_id).unwrap();
    assert_eq!(
        group.capacity_reservation_contract_ids.len(),
        cnt_members_before + 1
    );
    assert_eq!(
        group.capacity_reservation_contract_ids[cnt_members_before],
        cnt_contracts + 1
    );

    assert_eq!(
        SmartContractModule::capacity_reservation_id_by_node_group_config(types::NodeGroupConfig {
            group_id: group_id,
            node_id: expected_node_id
        }),
        cnt_contracts + 1
    );
}

pub fn create_twin(origin: AccountId) {
    assert_ok!(TfgridModule::user_accept_tc(
        RuntimeOrigin::signed(origin.clone()),
        get_document_link_input(b"some_link"),
        get_document_hash_input(b"some_hash"),
    ));

    let ip = get_twin_ip_input(b"::1");
    assert_ok!(TfgridModule::create_twin(RuntimeOrigin::signed(origin), ip));
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

    assert_ok!(SmartContractModule::solution_provider_create(
        RuntimeOrigin::signed(dave()),
        "some_description".as_bytes().to_vec(),
        "some_link".as_bytes().to_vec(),
        providers
    ));

    assert_ok!(SmartContractModule::solution_provider_approve(
        RawOrigin::Root.into(),
        1,
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
    BoundedVec::<u8, crate::MaxDeploymentDataLength<TestRuntime>>::try_from(
        "some_data".as_bytes().to_vec(),
    )
    .unwrap()
}

fn get_updated_deployment_data() -> crate::DeploymentDataInput<TestRuntime> {
    BoundedVec::<u8, crate::MaxDeploymentDataLength<TestRuntime>>::try_from(
        "changedthedata".as_bytes().to_vec(),
    )
    .unwrap()
}

fn get_resources() -> ResourcesInput {
    ResourcesInput {
        cru: 2,
        hru: 0,
        mru: 2 * GIGABYTE,
        sru: 60 * GIGABYTE,
    }
}

pub fn resources_n1() -> ResourcesInput {
    ResourcesInput {
        hru: 1024 * GIGABYTE,
        sru: 1024 * GIGABYTE,
        cru: 8,
        mru: 16 * GIGABYTE,
    }
}

pub fn resources_n2() -> ResourcesInput {
    ResourcesInput {
        hru: 2048 * GIGABYTE,
        sru: 2048 * GIGABYTE,
        cru: 16,
        mru: 32 * GIGABYTE,
    }
}

pub fn resources_n3() -> ResourcesInput {
    ResourcesInput {
        hru: 512 * GIGABYTE,
        sru: 512 * GIGABYTE,
        cru: 4,
        mru: 8 * GIGABYTE,
    }
}

fn resources_c1() -> ResourcesInput {
    ResourcesInput {
        cru: 4,
        hru: 0,
        mru: 2 * GIGABYTE,
        sru: 60 * GIGABYTE,
    }
}

fn half_resources_c1() -> ResourcesInput {
    ResourcesInput {
        cru: 2,
        hru: 0,
        mru: 1 * GIGABYTE,
        sru: 30 * GIGABYTE,
    }
}

fn resources_c2() -> ResourcesInput {
    ResourcesInput {
        cru: 4,
        hru: 1000 * GIGABYTE,
        mru: 10 * GIGABYTE,
        sru: 100 * GIGABYTE,
    }
}

fn resources_c3() -> ResourcesInput {
    ResourcesInput {
        cru: 2,
        hru: 512 * GIGABYTE,
        mru: 4 * GIGABYTE,
        sru: 50 * GIGABYTE,
    }
}

fn prepare_farm_three_nodes_three_capacity_reservation_contracts() {
    prepare_farm_with_three_nodes();

    // first contract should go to node 1
    assert_ok!(SmartContractModule::contract_capacity_reservation_create(
        RuntimeOrigin::signed(alice()),
        1,
        CapacityReservationPolicy::Any {
            resources: resources_c1(),
            features: None,
        },
        None,
    ));

    assert_eq!(
        TfgridModule::nodes(1).unwrap().power.target,
        PowerTarget::Up
    );
    assert_eq!(
        TfgridModule::nodes(2).unwrap().power.target,
        PowerTarget::Down
    );
    assert_eq!(
        TfgridModule::nodes(3).unwrap().power.target,
        PowerTarget::Down
    );
    assert_eq!(
        TfgridModule::nodes(1).unwrap().resources.used_resources,
        resources_c1()
    );
    assert_eq!(
        SmartContractModule::contracts(1).unwrap().contract_type,
        types::ContractData::CapacityReservationContract(types::CapacityReservationContract {
            node_id: 1,
            group_id: None,
            public_ips: 0,
            resources: ConsumableResources {
                total_resources: resources_c1(),
                used_resources: ResourcesInput::empty(),
            },
            deployments: vec![],
        })
    );

    // second contract will take most resources but can still go to node 1
    assert_ok!(SmartContractModule::contract_capacity_reservation_create(
        RuntimeOrigin::signed(alice()),
        1,
        CapacityReservationPolicy::Any {
            resources: resources_c2(),
            features: None,
        },
        None,
    ));
    assert_eq!(
        TfgridModule::nodes(1).unwrap().resources.used_resources,
        ResourcesInput::sum(&resources_c1(), &resources_c2())
    );
    assert_eq!(
        SmartContractModule::contracts(2).unwrap().contract_type,
        types::ContractData::CapacityReservationContract(types::CapacityReservationContract {
            node_id: 1,
            group_id: None,
            public_ips: 0,
            resources: ConsumableResources {
                total_resources: resources_c2(),
                used_resources: ResourcesInput::empty(),
            },
            deployments: vec![],
        })
    );

    // third can no longer go on node 1 so should start node 2 up
    assert_ok!(SmartContractModule::contract_capacity_reservation_create(
        RuntimeOrigin::signed(alice()),
        1,
        CapacityReservationPolicy::Any {
            resources: resources_c3(),
            features: None,
        },
        None,
    ),);

    assert_eq!(
        TfgridModule::nodes(1).unwrap().power.target,
        PowerTarget::Up
    );
    assert_eq!(
        TfgridModule::nodes(2).unwrap().power.target,
        PowerTarget::Up
    );
    assert_eq!(
        TfgridModule::nodes(3).unwrap().power.target,
        PowerTarget::Down
    );
    assert_eq!(
        TfgridModule::nodes(2).unwrap().resources.used_resources,
        resources_c3()
    );
    assert_eq!(
        SmartContractModule::contracts(3).unwrap().contract_type,
        types::ContractData::CapacityReservationContract(types::CapacityReservationContract {
            node_id: 2,
            group_id: None,
            public_ips: 0,
            resources: ConsumableResources {
                total_resources: resources_c3(),
                used_resources: ResourcesInput::empty(),
            },
            deployments: vec![],
        })
    );

    assert_eq!(SmartContractModule::active_node_contracts(1).len(), 2);
    assert_eq!(SmartContractModule::active_node_contracts(2).len(), 1);
}
