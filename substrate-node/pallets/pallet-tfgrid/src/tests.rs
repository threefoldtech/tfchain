use super::Event as TfgridEvent;
use crate::{
    mock::RuntimeEvent as MockEvent, mock::*, types::LocationInput, Error, InterfaceInput,
    InterfaceIpsInput, PublicIpListInput, ResourcesInput,
};
use frame_support::{assert_noop, assert_ok, bounded_vec};
use frame_system::{EventRecord, Phase, RawOrigin};
use sp_core::H256;
use tfchain_support::types::{
    FarmCertification, FarmingPolicyLimit, Interface, NodeCertification, Power, PowerState,
    PublicConfig, PublicIpError, IP4, IP6,
};
const GIGABYTE: u64 = 1024 * 1024 * 1024;

#[test]
fn test_create_entity_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
    });
}

#[test]
fn test_create_entity_sr_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity_sr();
    });
}

#[test]
fn test_update_entity_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();

        // Change name to barfoo
        let name = b"barfoo".to_vec();

        assert_ok!(TfgridModule::update_entity(
            RuntimeOrigin::signed(test_ed25519()),
            name,
            get_country_name_input(b"Belgium"),
            get_city_name_input(b"Ghent"),
        ));
    });
}

#[test]
fn test_update_entity_fails_if_signed_by_someone_else() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();

        // Change name to barfoo
        let name = b"barfoo".to_vec();

        assert_noop!(
            TfgridModule::update_entity(
                RuntimeOrigin::signed(bob()),
                name,
                get_country_name_input(b"Belgium"),
                get_city_name_input(b"Ghent"),
            ),
            Error::<TestRuntime>::EntityNotExists
        );
    });
}

#[test]
fn test_create_entity_double_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();

        let name = b"foobar".to_vec();
        let country = get_country_name_input(b"Belgium");
        let city = get_city_name_input(b"Ghent");

        let signature = sign_create_entity(name.clone(), country.to_vec(), city.to_vec());

        assert_noop!(
            TfgridModule::create_entity(
                RuntimeOrigin::signed(alice()),
                test_ed25519(),
                name,
                country,
                city,
                signature,
            ),
            Error::<TestRuntime>::EntityWithNameExists
        );
    });
}

#[test]
fn test_create_entity_double_fails_with_same_pubkey() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();

        let name = b"barfoo".to_vec();
        let country = get_country_name_input(b"Belgium");
        let city = get_city_name_input(b"Ghent");

        let signature = sign_create_entity(name.clone(), country.to_vec(), city.to_vec());

        assert_noop!(
            TfgridModule::create_entity(
                RuntimeOrigin::signed(alice()),
                test_ed25519(),
                name,
                country,
                city,
                signature,
            ),
            Error::<TestRuntime>::EntityWithPubkeyExists
        );
    });
}

#[test]
fn test_delete_entity_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();

        assert_ok!(TfgridModule::delete_entity(RuntimeOrigin::signed(
            test_ed25519()
        )));
    });
}

#[test]
fn test_delete_entity_fails_if_signed_by_someone_else() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();

        assert_noop!(
            TfgridModule::delete_entity(RuntimeOrigin::signed(bob())),
            Error::<TestRuntime>::EntityNotExists
        );
    });
}

#[test]
fn test_create_twin_works() {
    ExternalityBuilder::build().execute_with(|| {
        assert_ok!(TfgridModule::user_accept_tc(
            RuntimeOrigin::signed(test_ed25519()),
            get_document_link_input(b"some_link"),
            get_document_hash_input(b"some_hash"),
        ));

        log::info!("relay from tesT: {:?}", b"somerelay.io");
        let relay = get_relay_input(b"somerelay.io");
        let pk = get_public_key_input(
            b"0x6c8fd181adc178cea218e168e8549f0b0ff30627c879db9eac4318927e87c901",
        );

        assert_ok!(TfgridModule::create_twin(
            RuntimeOrigin::signed(test_ed25519()),
            relay,
            pk,
        ));
    });
}

#[test]
fn test_delete_node_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_twin();
        create_farm();
        create_node();

        let nodes = TfgridModule::nodes_by_farm_id(1);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0], 1);

        assert_ok!(TfgridModule::delete_node_farm(
            RuntimeOrigin::signed(alice()),
            1
        ));

        let nodes = TfgridModule::nodes_by_farm_id(1);
        assert_eq!(nodes.len(), 0);
    });
}

#[test]
fn test_delete_node_fails_if_not_authorized() {
    ExternalityBuilder::build().execute_with(|| {
        create_twin();
        create_twin_bob();
        create_farm();
        create_node();

        assert_noop!(
            TfgridModule::delete_node_farm(RuntimeOrigin::signed(bob()), 1),
            Error::<TestRuntime>::FarmerNotAuthorized
        );
    });
}

#[test]
fn test_add_entity_to_twin() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin_bob();

        // Signature of the entityid (1) and twinid (1) signed with test_ed25519 account
        let signature = sign_add_entity_to_twin(1, 1);

        let twin_id = 1;
        let entity_id = 1;

        // Bob adds someone as entity to his twin
        assert_ok!(TfgridModule::add_twin_entity(
            RuntimeOrigin::signed(bob()),
            twin_id,
            entity_id,
            signature
        ));
    });
}

#[test]
fn test_add_entity_to_twin_fails_with_invalid_signature() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin_bob();

        // Signature of the entityid (1) and twinid (2) signed with test_ed25519 account
        let signature = sign_add_entity_to_twin(1, 2);

        let twin_id = 1;
        let entity_id = 1;

        assert_noop!(
            TfgridModule::add_twin_entity(
                RuntimeOrigin::signed(bob()),
                twin_id,
                entity_id,
                signature
            ),
            Error::<TestRuntime>::EntitySignatureDoesNotMatch
        );
    });
}

#[test]
fn test_add_entity_to_twin_fails_if_entity_is_added_twice() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin_bob();

        // Add Alice as entity to bob's twin

        // Signature of the entityid (1) and twinid (1) signed with test_ed25519 account
        let signature = sign_add_entity_to_twin(1, 1);

        let twin_id = 1;
        let entity_id = 1;

        assert_ok!(TfgridModule::add_twin_entity(
            RuntimeOrigin::signed(bob()),
            twin_id,
            entity_id,
            signature.clone()
        ));

        assert_noop!(
            TfgridModule::add_twin_entity(
                RuntimeOrigin::signed(bob()),
                twin_id,
                entity_id,
                signature
            ),
            Error::<TestRuntime>::EntityWithSignatureAlreadyExists
        );
    });
}

#[test]
fn test_create_twin_double_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_twin();

        let relay = get_relay_input(b"somerelay.io");
        let pk = get_public_key_input(
            b"0x6c8fd181adc178cea218e168e8549f0b0ff30627c879db9eac4318927e87c901",
        );

        assert_noop!(
            TfgridModule::create_twin(RuntimeOrigin::signed(alice()), relay, pk),
            Error::<TestRuntime>::TwinWithPubkeyExists
        );
    });
}

#[test]
fn test_create_farm_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
    });
}

#[test]
fn test_create_farm_invalid_name_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();

        let farm_name = get_farm_name_input(b"test.farm");
        assert_noop!(
            TfgridModule::create_farm(RuntimeOrigin::signed(alice()), farm_name, bounded_vec![]),
            Error::<TestRuntime>::InvalidFarmName
        );

        let farm_name = get_farm_name_input(b"test farm");
        assert_noop!(
            TfgridModule::create_farm(RuntimeOrigin::signed(alice()), farm_name, bounded_vec![]),
            Error::<TestRuntime>::InvalidFarmName
        );

        let farm_name = get_farm_name_input(b"");
        assert_noop!(
            TfgridModule::create_farm(RuntimeOrigin::signed(alice()), farm_name, bounded_vec![]),
            Error::<TestRuntime>::FarmNameTooShort
        );

        let farm_name = get_farm_name_input(b"12");
        assert_noop!(
            TfgridModule::create_farm(RuntimeOrigin::signed(alice()), farm_name, bounded_vec![]),
            Error::<TestRuntime>::FarmNameTooShort
        );
    });
}

#[test]
fn test_update_farm_name_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_twin();
        create_farm();

        create_twin_bob();

        let farm_name = get_farm_name_input(b"bob_farm");
        assert_ok!(TfgridModule::create_farm(
            RuntimeOrigin::signed(bob()),
            farm_name,
            bounded_vec![]
        ));

        let farm_name = get_farm_name_input(b"bob_updated_farm");
        assert_ok!(TfgridModule::update_farm(
            RuntimeOrigin::signed(bob()),
            2,
            farm_name,
        ));
    });
}

#[test]
fn test_update_farm_existing_name_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_twin();

        let farm_name = get_farm_name_input(b"alice_farm");
        assert_ok!(TfgridModule::create_farm(
            RuntimeOrigin::signed(alice()),
            farm_name,
            bounded_vec![]
        ));

        create_twin_bob();

        let farm_name = get_farm_name_input(b"bob_farm");
        assert_ok!(TfgridModule::create_farm(
            RuntimeOrigin::signed(bob()),
            farm_name,
            bounded_vec![]
        ));

        let farm_name = get_farm_name_input(b"alice_farm");
        assert_noop!(
            TfgridModule::update_farm(RuntimeOrigin::signed(bob()), 2, farm_name),
            Error::<TestRuntime>::InvalidFarmName
        );
    });
}

#[test]
fn test_create_farm_with_double_ip_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();

        let farm_name = get_farm_name_input(b"test_farm");

        let mut pub_ips: PublicIpListInput<TestRuntime> = bounded_vec![];

        let ip = get_public_ip_ip_input(b"185.206.122.33/24");
        let gw = get_public_ip_gw_input(b"185.206.122.1");

        pub_ips
            .try_push(IP4 {
                ip: ip.clone(),
                gw: gw.clone(),
            })
            .unwrap();
        pub_ips.try_push(IP4 { ip, gw }).unwrap();

        assert_noop!(
            TfgridModule::create_farm(RuntimeOrigin::signed(alice()), farm_name, pub_ips),
            Error::<TestRuntime>::IpExists
        );
    });
}

#[test]
fn test_adding_ip_to_farm_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();

        assert_ok!(TfgridModule::add_farm_ip(
            RuntimeOrigin::signed(alice()),
            1,
            get_public_ip_ip_input(b"185.206.122.125/16"),
            get_public_ip_gw_input(b"185.206.122.1"),
        ));
    });
}

#[test]
fn test_adding_misformatted_ip_to_farm_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();

        assert_noop!(
            TfgridModule::add_farm_ip(
                RuntimeOrigin::signed(alice()),
                1,
                get_public_ip_ip_input(b"185.206.122.125"),
                get_public_ip_gw_input(b"185.206.122.1"),
            ),
            Error::<TestRuntime>::InvalidPublicIP
        );
    });
}

#[test]
fn test_adding_ip_duplicate_to_farm_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();

        assert_ok!(TfgridModule::add_farm_ip(
            RuntimeOrigin::signed(alice()),
            1,
            get_public_ip_ip_input(b"185.206.122.125/16"),
            get_public_ip_gw_input(b"185.206.122.1"),
        ));

        assert_noop!(
            TfgridModule::add_farm_ip(
                RuntimeOrigin::signed(alice()),
                1,
                get_public_ip_ip_input(b"185.206.122.125/16"),
                get_public_ip_gw_input(b"185.206.122.1"),
            ),
            Error::<TestRuntime>::IpExists
        );
    });
}

#[test]
fn test_set_farm_dedicated() {
    ExternalityBuilder::build().execute_with(|| {
        create_twin();
        create_farm();

        assert_ok!(TfgridModule::set_farm_dedicated(
            RawOrigin::Root.into(),
            1,
            true
        ));

        let farm = TfgridModule::farms(1).unwrap();
        assert_eq!(farm.dedicated_farm, true);
    })
}

#[test]
fn test_toggle_farm_dedicated() {
    ExternalityBuilder::build().execute_with(|| {
        create_twin();
        create_farm();

        assert_ok!(TfgridModule::set_farm_dedicated(
            RawOrigin::Root.into(),
            1,
            true
        ));

        let farm = TfgridModule::farms(1).unwrap();
        assert_eq!(farm.dedicated_farm, true);

        assert_ok!(TfgridModule::set_farm_dedicated(
            RawOrigin::Root.into(),
            1,
            false
        ));

        let farm = TfgridModule::farms(1).unwrap();
        assert_eq!(farm.dedicated_farm, false);
    })
}

#[test]
fn test_update_twin_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_twin();

        let relay = get_relay_input(b"somerelay.io");
        let pk = get_public_key_input(
            b"0x6c8fd181adc178cea218e168e8549f0b0ff30627c879db9eac4318927e87c901",
        );

        assert_ok!(TfgridModule::update_twin(
            RuntimeOrigin::signed(alice()),
            relay,
            pk,
        ));
    });
}

#[test]
fn test_update_twin_fails_if_signed_by_someone_else() {
    ExternalityBuilder::build().execute_with(|| {
        assert_ok!(TfgridModule::user_accept_tc(
            RuntimeOrigin::signed(alice()),
            get_document_link_input(b"some_link"),
            get_document_hash_input(b"some_hash"),
        ));

        let relay = get_relay_input(b"somerelay.io");
        let pk = get_public_key_input(
            b"0x6c8fd181adc178cea218e168e8549f0b0ff30627c879db9eac4318927e87c901",
        );

        assert_ok!(TfgridModule::create_twin(
            RuntimeOrigin::signed(alice()),
            relay.clone(),
            pk.clone()
        ));

        assert_noop!(
            TfgridModule::update_twin(RuntimeOrigin::signed(bob()), relay, pk),
            Error::<TestRuntime>::TwinNotExists
        );
    });
}

#[test]
fn test_create_farm_with_same_name_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();

        let farm_name = get_farm_name_input(b"test_farm");

        let pub_ips: PublicIpListInput<TestRuntime> = bounded_vec![];

        assert_noop!(
            TfgridModule::create_farm(RuntimeOrigin::signed(alice()), farm_name, pub_ips),
            Error::<TestRuntime>::FarmExists
        );
    });
}

#[test]
fn test_farm_add_stellar_payout_address() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();

        let addr = "some_address".as_bytes().to_vec();
        assert_ok!(TfgridModule::add_stellar_payout_v2address(
            RuntimeOrigin::signed(alice()),
            1,
            addr
        ));

        let addr2 = "some_other_address".as_bytes().to_vec();
        assert_ok!(TfgridModule::add_stellar_payout_v2address(
            RuntimeOrigin::signed(alice()),
            1,
            addr2
        ));
    });
}

#[test]
fn test_set_farm_certification_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();

        assert_ok!(TfgridModule::set_farm_certification(
            RawOrigin::Root.into(),
            1,
            FarmCertification::Gold
        ));

        let f1 = TfgridModule::farms(1).unwrap();
        assert_eq!(f1.certification, FarmCertification::Gold);
    });
}

#[test]
fn create_node_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();
    });
}

#[test]
fn create_node_added_to_farm_list_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();

        let nodes = TfgridModule::nodes_by_farm_id(1);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0], 1);
    });
}

#[test]
fn update_node_moved_from_farm_list_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_farm2();
        create_node();

        let nodes = TfgridModule::nodes_by_farm_id(1);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0], 1);

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

        assert_ok!(TfgridModule::update_node(
            RuntimeOrigin::signed(alice()),
            1,
            2,
            resources,
            location,
            bounded_vec![],
            true,
            true,
            None,
        ));

        // should be removed from farm 1 nodes
        let nodes = TfgridModule::nodes_by_farm_id(1);
        assert_eq!(nodes.len(), 0);

        // Should be part of farm 2 nodes
        let nodes = TfgridModule::nodes_by_farm_id(2);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0], 1);
    });
}

#[test]
fn update_certified_node_resources_loses_certification_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();

        assert_ok!(TfgridModule::add_node_certifier(
            RawOrigin::Root.into(),
            alice()
        ));

        assert_ok!(TfgridModule::set_node_certification(
            RuntimeOrigin::signed(alice()),
            1,
            NodeCertification::Certified
        ));

        let node = TfgridModule::nodes(1).unwrap();
        assert_eq!(node.certification, NodeCertification::Certified);

        // Change cores to 2
        let mut node_resources: ResourcesInput = node.resources;
        node_resources.cru = 2;

        let node_location = LocationInput {
            city: node.location.city.0,
            country: node.location.country.0,
            latitude: node.location.latitude,
            longitude: node.location.longitude,
        };

        assert_ok!(TfgridModule::update_node(
            RuntimeOrigin::signed(alice()),
            1,
            1,
            node_resources,
            node_location,
            bounded_vec![],
            true,
            true,
            None,
        ));

        let our_events = System::events();
        assert_eq!(
            our_events.contains(&record(MockEvent::TfgridModule(
                TfgridEvent::<TestRuntime>::NodeCertificationSet(1, NodeCertification::Diy)
            ))),
            true
        );

        let node = TfgridModule::nodes(1).unwrap();
        assert_eq!(node.certification, NodeCertification::Diy);
    });
}

#[test]
fn update_certified_node_same_resources_keeps_certification_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();

        assert_ok!(TfgridModule::add_node_certifier(
            RawOrigin::Root.into(),
            alice()
        ));

        assert_ok!(TfgridModule::set_node_certification(
            RuntimeOrigin::signed(alice()),
            1,
            NodeCertification::Certified
        ));

        let node = TfgridModule::nodes(1).unwrap();
        assert_eq!(node.certification, NodeCertification::Certified);

        let node_resources: ResourcesInput = node.resources;

        let node_location = LocationInput {
            city: node.location.city.0,
            country: node.location.country.0,
            latitude: node.location.latitude,
            longitude: node.location.longitude,
        };

        // Don't change resources
        assert_ok!(TfgridModule::update_node(
            RuntimeOrigin::signed(alice()),
            1,
            1,
            node_resources,
            node_location,
            bounded_vec![],
            true,
            true,
            None,
        ));

        let node = TfgridModule::nodes(1).unwrap();
        assert_eq!(node.certification, NodeCertification::Certified);
    });
}

#[test]
fn create_node_with_interfaces_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();

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

        let mut interface_ips: InterfaceIpsInput<TestRuntime> = bounded_vec![];
        let intf_ip = get_interface_ip_input(b"10.2.3.3");
        interface_ips.try_push(intf_ip).unwrap();

        let interface = Interface {
            name: get_interface_name_input(b"zos"),
            mac: get_interface_mac_input(b"00:00:5e:00:53:af"),
            ips: interface_ips,
        };

        let mut interfaces: InterfaceInput<TestRuntime> = bounded_vec![];
        interfaces.try_push(interface).unwrap();

        assert_ok!(TfgridModule::create_node(
            RuntimeOrigin::signed(alice()),
            1,
            resources,
            location,
            interfaces,
            true,
            true,
            None,
        ));
    });
}

#[test]
fn add_node_certifier_works() {
    ExternalityBuilder::build().execute_with(|| {
        assert_ok!(TfgridModule::add_node_certifier(
            RawOrigin::Root.into(),
            alice()
        ));

        if let Some(node_certifiers) = TfgridModule::allowed_node_certifiers() {
            assert_eq!(node_certifiers[0], alice());
        }
    });
}

#[test]
fn add_node_certifier_double_fails() {
    ExternalityBuilder::build().execute_with(|| {
        assert_ok!(TfgridModule::add_node_certifier(
            RawOrigin::Root.into(),
            alice()
        ));
        if let Some(node_certifiers) = TfgridModule::allowed_node_certifiers() {
            assert_eq!(node_certifiers[0], alice());
        }

        assert_noop!(
            TfgridModule::add_node_certifier(RawOrigin::Root.into(), alice()),
            Error::<TestRuntime>::AlreadyCertifier
        );
    });
}

#[test]
fn remove_node_certifier_works() {
    ExternalityBuilder::build().execute_with(|| {
        assert_ok!(TfgridModule::add_node_certifier(
            RawOrigin::Root.into(),
            alice()
        ));

        if let Some(node_certifiers) = TfgridModule::allowed_node_certifiers() {
            assert_eq!(node_certifiers[0], alice());
        }

        assert_ok!(TfgridModule::remove_node_certifier(
            RawOrigin::Root.into(),
            alice()
        ));
    });
}

#[test]
fn set_certification_type_node_allowed_certifier_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();

        assert_ok!(TfgridModule::add_node_certifier(
            RawOrigin::Root.into(),
            alice()
        ));

        assert_ok!(TfgridModule::set_node_certification(
            RuntimeOrigin::signed(alice()),
            1,
            NodeCertification::Certified
        ));
        let node = TfgridModule::nodes(1).unwrap();
        assert_eq!(node.certification, NodeCertification::Certified);

        assert_ok!(TfgridModule::set_node_certification(
            RuntimeOrigin::signed(alice()),
            1,
            NodeCertification::Diy
        ));
        let node = TfgridModule::nodes(1).unwrap();
        assert_eq!(node.certification, NodeCertification::Diy);
    });
}

#[test]
fn set_certification_type_node_not_allowed_certifier_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();

        assert_noop!(
            TfgridModule::set_node_certification(
                RuntimeOrigin::signed(alice()),
                1,
                NodeCertification::Certified
            ),
            Error::<TestRuntime>::NotAllowedToCertifyNode
        );
    });
}

#[test]
fn set_certification_type_node_council_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();

        assert_ok!(TfgridModule::set_node_certification(
            RawOrigin::Root.into(),
            1,
            NodeCertification::Certified
        ));
        let node = TfgridModule::nodes(1).unwrap();
        assert_eq!(node.certification, NodeCertification::Certified);

        assert_ok!(TfgridModule::set_node_certification(
            RawOrigin::Root.into(),
            1,
            NodeCertification::Diy
        ));
        let node = TfgridModule::nodes(1).unwrap();
        assert_eq!(node.certification, NodeCertification::Diy);
    });
}

#[test]
fn set_certification_type_node_not_exists_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();

        assert_noop!(
            TfgridModule::set_node_certification(
                RawOrigin::Root.into(),
                1,
                NodeCertification::Certified
            ),
            Error::<TestRuntime>::NodeNotExists
        );
    });
}

#[test]
fn node_report_uptime_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();

        Timestamp::set_timestamp(1628082000);
        assert_ok!(TfgridModule::report_uptime(
            RuntimeOrigin::signed(alice()),
            500
        ));
    });
}

#[test]
fn change_power_state_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();

        assert_ok!(TfgridModule::change_power_state(
            RuntimeOrigin::signed(alice()),
            Power::Down
        ));

        assert_eq!(TfgridModule::node_power_state(1).state, PowerState::Down(1));

        let our_events = System::events();
        assert_eq!(
            our_events.contains(&record(MockEvent::TfgridModule(
                TfgridEvent::<TestRuntime>::PowerStateChanged {
                    farm_id: 1,
                    node_id: 1,
                    power_state: PowerState::Down(1)
                }
            ))),
            true
        );
    });
}

#[test]
fn change_power_state_twin_has_no_node_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();
        create_twin_bob();
        //try changing the power state using another twin_id
        assert_noop!(
            TfgridModule::change_power_state(RuntimeOrigin::signed(bob()), Power::Down),
            Error::<TestRuntime>::NodeNotExists
        );
    });
}

#[test]
fn change_power_target_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();

        assert_eq!(TfgridModule::node_power_state(1).target, Power::Up,);

        assert_ok!(TfgridModule::change_power_target(
            RuntimeOrigin::signed(alice()),
            1,
            Power::Down,
        ));

        assert_eq!(TfgridModule::node_power_state(1).target, Power::Down,);

        let our_events = System::events();
        assert_eq!(
            our_events.contains(&record(MockEvent::TfgridModule(
                TfgridEvent::<TestRuntime>::PowerTargetChanged {
                    farm_id: 1,
                    node_id: 1,
                    power_target: Power::Down
                }
            ))),
            true
        );
    });
}

#[test]
fn change_power_target_unauthorized_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();

        // Create farms with Bob
        create_twin_bob();
        create_farm_bob();

        // Try to change power target as Bob on Alice's farm
        assert_noop!(
            TfgridModule::change_power_target(RuntimeOrigin::signed(bob()), 1, Power::Down,),
            Error::<TestRuntime>::UnauthorizedToChangePowerTarget
        );
    });
}

#[test]
fn node_add_public_config_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();

        let ipv4 = get_pub_config_ip4_input(b"185.206.122.33/24");
        let ipv6 = get_pub_config_ip6_input(b"2a10:b600:1::0cc4:7a30:65b5/64");
        let gw4 = get_pub_config_gw4_input(b"185.206.122.1");
        let gw6 = get_pub_config_gw6_input(b"2a10:b600:1::1");
        let domain = get_pub_config_domain_input(b"some-domain");

        let pub_config_input = PublicConfig {
            ip4: IP4 {
                ip: ipv4.clone(),
                gw: gw4.clone(),
            },
            ip6: Some(IP6 {
                ip: ipv6.clone(),
                gw: gw6.clone(),
            }),
            domain: Some(domain.clone()),
        };

        assert_ok!(TfgridModule::add_node_public_config(
            RuntimeOrigin::signed(alice()),
            1,
            1,
            Some(pub_config_input)
        ));

        let node = TfgridModule::nodes(1).unwrap();

        assert_eq!(
            node.public_config,
            Some(PublicConfig {
                ip4: IP4 { ip: ipv4, gw: gw4 },
                ip6: Some(IP6 { ip: ipv6, gw: gw6 }),
                domain: Some(domain),
            })
        );
    });
}

#[test]
fn node_add_public_config_without_ipv6_and_domain_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();

        let ipv4 = get_pub_config_ip4_input(b"185.206.122.33/24");
        let gw4 = get_pub_config_gw4_input(b"185.206.122.1");

        let pub_config_input = PublicConfig {
            ip4: IP4 {
                ip: ipv4.clone(),
                gw: gw4.clone(),
            },
            ip6: None,
            domain: None,
        };

        assert_ok!(TfgridModule::add_node_public_config(
            RuntimeOrigin::signed(alice()),
            1,
            1,
            Some(pub_config_input)
        ));

        let node = TfgridModule::nodes(1).unwrap();
        assert_eq!(
            node.public_config,
            Some(PublicConfig {
                ip4: IP4 { ip: ipv4, gw: gw4 },
                ip6: None,
                domain: None,
            })
        );
    });
}

#[test]
fn node_add_public_config_fails_if_signature_incorrect() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();

        let pub_config_input = PublicConfig {
            ip4: IP4 {
                ip: get_pub_config_ip4_input(b"185.206.122.33/24"),
                gw: get_pub_config_gw4_input(b"185.206.122.1"),
            },
            ip6: Some(IP6 {
                ip: get_pub_config_ip6_input(b"2a10:b600:1::0cc4:7a30:65b5/64"),
                gw: get_pub_config_gw6_input(b"2a10:b600:1::1"),
            }),
            domain: Some(get_pub_config_domain_input(b"some-domain")),
        };

        assert_noop!(
            TfgridModule::add_node_public_config(
                RuntimeOrigin::signed(bob()),
                1,
                1,
                Some(pub_config_input)
            ),
            Error::<TestRuntime>::CannotUpdateFarmWrongTwin
        );
    });
}

#[test]
fn test_unsetting_node_public_config_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();

        let ipv4 = get_pub_config_ip4_input(b"185.206.122.33/24");
        let ipv6 = get_pub_config_ip6_input(b"2a10:b600:1::0cc4:7a30:65b5/64");
        let gw4 = get_pub_config_gw4_input(b"185.206.122.1");
        let gw6 = get_pub_config_gw6_input(b"2a10:b600:1::1");
        let domain = get_pub_config_domain_input(b"some-domain");

        let pub_config_input = PublicConfig {
            ip4: IP4 {
                ip: ipv4.clone(),
                gw: gw4.clone(),
            },
            ip6: Some(IP6 {
                ip: ipv6.clone(),
                gw: gw6.clone(),
            }),
            domain: Some(domain.clone()),
        };

        assert_ok!(TfgridModule::add_node_public_config(
            RuntimeOrigin::signed(alice()),
            1,
            1,
            Some(pub_config_input)
        ));

        let node = TfgridModule::nodes(1).unwrap();

        assert_eq!(
            node.public_config,
            Some(PublicConfig {
                ip4: IP4 { ip: ipv4, gw: gw4 },
                ip6: Some(IP6 { ip: ipv6, gw: gw6 }),
                domain: Some(domain),
            })
        );

        assert_ok!(TfgridModule::add_node_public_config(
            RuntimeOrigin::signed(alice()),
            1,
            1,
            None
        ));

        let node = TfgridModule::nodes(1).unwrap();
        assert_eq!(node.public_config, None);
    });
}

#[test]
fn test_node_public_config_falsy_values_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();

        let pub_config_input = PublicConfig {
            ip4: IP4 {
                ip: get_pub_config_ip4_input(b"1.1.1.1"), // Too short
                gw: get_pub_config_gw4_input(b"185.206.122.1"),
            },
            ip6: Some(IP6 {
                ip: get_pub_config_ip6_input(b"2a10:b600:1::0cc4:7a30:65b5/64"),
                gw: get_pub_config_gw6_input(b"2a10:b600:1::1"),
            }),
            domain: Some(get_pub_config_domain_input(b"some-domain")),
        };

        assert_noop!(
            TfgridModule::add_node_public_config(
                RuntimeOrigin::signed(alice()),
                1,
                1,
                Some(pub_config_input)
            ),
            Error::<TestRuntime>::InvalidPublicConfig
        );
    });
}

#[test]
fn test_validate_pub_config_invalid_ip4() {
    ExternalityBuilder::build().execute_with(|| {
        let ipv4 = get_pub_config_ip4_input(b"185.206.122.33");
        let ipv6 = get_pub_config_ip6_input(b"2a10:b600:1::0cc4:7a30:65b5/64");
        let gw4 = get_pub_config_gw4_input(b"185.206.122.1");
        let gw6 = get_pub_config_gw6_input(b"2a10:b600:1::1");
        let domain = get_pub_config_domain_input(b"some-domain");

        let pub_conf = PublicConfig {
            ip4: IP4 {
                ip: ipv4.clone(),
                gw: gw4.clone(),
            },
            ip6: Some(IP6 {
                ip: ipv6.clone(),
                gw: gw6.clone(),
            }),
            domain: Some(domain.clone()),
        };

        assert_noop!(pub_conf.is_valid(), PublicIpError::InvalidIp4);
    });
}

#[test]
fn test_validate_pub_config_invalid_gw4() {
    ExternalityBuilder::build().execute_with(|| {
        let ipv4 = get_pub_config_ip4_input(b"185.206.122.33/24");
        let ipv6 = get_pub_config_ip6_input(b"2a10:b600:1::0cc4:7a30:65b5/64");
        let gw4 = get_pub_config_gw4_input(b"185.206.132.1");
        let gw6 = get_pub_config_gw6_input(b"2a10:b600:1::1");
        let domain = get_pub_config_domain_input(b"some-domain");

        let pub_conf = PublicConfig {
            ip4: IP4 {
                ip: ipv4.clone(),
                gw: gw4.clone(),
            },
            ip6: Some(IP6 {
                ip: ipv6.clone(),
                gw: gw6.clone(),
            }),
            domain: Some(domain.clone()),
        };

        assert_noop!(pub_conf.is_valid(), PublicIpError::InvalidPublicIp);
    });
}

#[test]
fn test_validate_pub_config_invalid_ip6() {
    ExternalityBuilder::build().execute_with(|| {
        let ipv4 = get_pub_config_ip4_input(b"185.206.122.33/24");
        let ipv6 = get_pub_config_ip6_input(b"2a10::0cc4:7a30:65b5/32");
        let gw4 = get_pub_config_gw4_input(b"185.206.122.1");
        let gw6 = get_pub_config_gw6_input(b"2a10:b600:1::1");
        let domain = get_pub_config_domain_input(b"some-domain");

        let pub_conf = PublicConfig {
            ip4: IP4 {
                ip: ipv4.clone(),
                gw: gw4.clone(),
            },
            ip6: Some(IP6 {
                ip: ipv6.clone(),
                gw: gw6.clone(),
            }),
            domain: Some(domain.clone()),
        };

        assert_noop!(pub_conf.is_valid(), PublicIpError::InvalidPublicIp);
    });
}

#[test]
fn create_node_with_same_pubkey_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();

        let resources = ResourcesInput {
            hru: 1,
            sru: 1,
            cru: 1,
            mru: 1,
        };

        // random location
        let location = LocationInput {
            city: get_city_name_input(b"Ghent"),
            country: get_country_name_input(b"Belgium"),
            latitude: get_latitude_input(b"12.233213231"),
            longitude: get_longitude_input(b"32.323112123"),
        };

        assert_noop!(
            TfgridModule::create_node(
                RuntimeOrigin::signed(alice()),
                1,
                resources,
                location,
                bounded_vec![],
                true,
                true,
                None,
            ),
            Error::<TestRuntime>::NodeWithTwinIdExists
        );
    });
}

#[test]
fn create_farming_policy_works() {
    ExternalityBuilder::build().execute_with(|| {
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
    });
}

#[test]
fn edit_farming_policy_works() {
    ExternalityBuilder::build().execute_with(|| {
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

        let name = "f1_updated".as_bytes().to_vec();
        assert_ok!(TfgridModule::update_farming_policy(
            RawOrigin::Root.into(),
            1,
            name,
            12,
            15,
            10,
            8,
            9999,
            System::block_number() + 100,
            true,
            NodeCertification::Diy,
            FarmCertification::NotCertified,
        ));
    });
}

#[test]
fn attach_farming_policy_to_farm_works() {
    ExternalityBuilder::build().execute_with(|| {
        // farming policy 2 is default and has no node/farm certification
        // see create_farming_policies()
        test_attach_farming_policy_flow(2);
    });
}

#[test]
fn attach_farming_policy_with_gold_farm_certification_works() {
    ExternalityBuilder::build().execute_with(|| {
        // farming policy 1 is default and has gold farm certification
        // see create_farming_policies()
        test_attach_farming_policy_flow(1);
    });
}

#[test]
fn attach_farming_policy_with_certified_node_certification_works() {
    ExternalityBuilder::build().execute_with(|| {
        // farming policy 3 is default and has certified node certification
        // see create_farming_policies()
        test_attach_farming_policy_flow(3);
    });
}

#[test]
fn attach_another_custom_farming_policy_to_farm_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_twin();
        create_farm();
        create_node();

        // Add custom policies 5 and 6
        create_custom_farming_policies();

        // Initially attach custom (non-default) farming policy 5 to farm
        let fp = TfgridModule::farming_policies_map(5);
        let limit = FarmingPolicyLimit {
            farming_policy_id: fp.id,
            cu: Some(20),
            su: Some(2),
            end: Some(1654058949),
            node_certification: false,
            node_count: Some(10),
        };

        // Link farming policy 5 to farm
        let farm_id = 1;
        assert_ok!(TfgridModule::attach_policy_to_farm(
            RawOrigin::Root.into(),
            farm_id,
            Some(limit.clone())
        ));

        // Check if node 1 received farming policy 5
        let node1_id = 1;
        let mut node1 = TfgridModule::nodes(node1_id).unwrap();
        assert_eq!(node1.farming_policy_id, fp.id);
        assert_eq!(node1.certification, fp.node_certification);

        // Set limit with new custom (non-default) farming policy 6
        // NB: no need to double CU and SU capacity here !!
        // Indeed, new limit will not considere node 1
        // because it already has farming policy 5 which is custom
        let new_fp = TfgridModule::farming_policies_map(6);
        let new_limit = FarmingPolicyLimit {
            farming_policy_id: new_fp.id,
            cu: Some(20),
            su: Some(2),
            end: Some(1654058949),
            node_certification: false,
            node_count: Some(10),
        };

        // Now link custom (non-default) farming policy 5 to farm
        assert_ok!(TfgridModule::attach_policy_to_farm(
            RawOrigin::Root.into(),
            farm_id,
            Some(new_limit.clone())
        ));

        // Get updated farm
        let mut farm = TfgridModule::farms(farm_id).unwrap();
        // Check updated farm limits, should be exactly the same as new farming policy
        // with full CU/SU capacity since node 1 has farming policy 5  which is custom
        let mut new_farm_limit = farm.clone().farming_policy_limits.unwrap();
        assert_eq!(new_farm_limit, new_limit);
        assert_eq!(farm.certification, new_fp.farm_certification);

        // Existing node 1 should not migrate to new farming policy
        node1 = TfgridModule::nodes(node1_id).unwrap();
        assert_eq!(node1.farming_policy_id, fp.id);
        assert_eq!(node1.certification, fp.node_certification);

        // Add extra node 2 to farm
        create_twin_bob();
        create_extra_node();

        // Extra node 2 should get new farming policy
        let node2_id = 2;
        let node2 = TfgridModule::nodes(node2_id).unwrap();
        assert_eq!(node2.farming_policy_id, new_fp.id);
        assert_eq!(node2.certification, new_fp.node_certification);

        // Get updated farm
        farm = TfgridModule::farms(farm_id).unwrap();
        // Check updated fields for farm limits
        new_farm_limit = farm.clone().farming_policy_limits.unwrap();
        assert_eq!(new_farm_limit.cu, Some(0)); // No more CU available
        assert_eq!(new_farm_limit.su, Some(0)); // No more SU available
        assert_eq!(new_farm_limit.node_count, Some(9)); // Remains 9 spots for nodes
    });
}

#[test]
fn add_farm_limits_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_twin();
        create_farm();

        let limit = FarmingPolicyLimit {
            farming_policy_id: 1,
            cu: Some(5),
            su: Some(10),
            end: Some(1654058949),
            node_certification: false,
            node_count: Some(10),
        };

        assert_ok!(TfgridModule::attach_policy_to_farm(
            RawOrigin::Root.into(),
            1,
            Some(limit.clone())
        ));

        let f1 = TfgridModule::farms(1).unwrap();
        assert_eq!(f1.farming_policy_limits, Some(limit));
    });
}

#[test]
fn add_farm_limits_on_expired_policy_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_twin();
        create_farm();

        let limit = FarmingPolicyLimit {
            farming_policy_id: 1,
            cu: Some(5),
            su: Some(10),
            end: Some(1654058949),
            node_certification: false,
            node_count: Some(10),
        };

        // Farming policies expire at block 101
        System::set_block_number(System::block_number() + 102);
        assert_noop!(
            TfgridModule::attach_policy_to_farm(RawOrigin::Root.into(), 1, Some(limit)),
            Error::<TestRuntime>::FarmingPolicyExpired
        );

        let f1 = TfgridModule::farms(1).unwrap();
        assert_eq!(f1.farming_policy_limits, None);
    });
}

#[test]
fn add_farm_limits_to_non_existent_farm_fails() {
    ExternalityBuilder::build().execute_with(|| {
        let limit = FarmingPolicyLimit {
            farming_policy_id: 1,
            cu: Some(5),
            su: Some(10),
            end: Some(1654058949),
            node_certification: false,
            node_count: Some(10),
        };

        assert_noop!(
            TfgridModule::attach_policy_to_farm(RawOrigin::Root.into(), 1, Some(limit)),
            Error::<TestRuntime>::FarmNotExists
        );
    });
}

#[test]
fn boot_node_when_farming_policy_has_limits_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_twin();
        create_farm();

        let limit = FarmingPolicyLimit {
            farming_policy_id: 1,
            cu: Some(21),
            su: Some(10),
            end: Some(1654058949),
            node_certification: false,
            node_count: Some(10),
        };

        assert_ok!(TfgridModule::attach_policy_to_farm(
            RawOrigin::Root.into(),
            1,
            Some(limit.clone())
        ));

        let f1 = TfgridModule::farms(1).unwrap();
        assert_eq!(f1.farming_policy_limits, Some(limit.clone()));

        create_node();

        let f1 = TfgridModule::farms(1).unwrap();
        assert_ne!(f1.farming_policy_limits, Some(limit.clone()));

        let n1 = TfgridModule::nodes(1).unwrap();
        assert_eq!(n1.farming_policy_id, limit.farming_policy_id);
    });
}

#[test]
fn boot_node_when_farming_policy_low_cu_limit_should_fall_back_to_a_default_policy_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_twin();
        create_farm();

        let limit = FarmingPolicyLimit {
            farming_policy_id: 1,
            cu: Some(5),
            su: Some(10),
            end: Some(1654058949),
            node_certification: false,
            node_count: Some(10),
        };

        assert_ok!(TfgridModule::attach_policy_to_farm(
            RawOrigin::Root.into(),
            1,
            Some(limit.clone())
        ));

        let f1 = TfgridModule::farms(1).unwrap();
        assert_eq!(f1.farming_policy_limits, Some(limit.clone()));

        create_node();

        let n1 = TfgridModule::nodes(1).unwrap();
        let default_p = TfgridModule::farming_policies_map(n1.farming_policy_id);
        assert_eq!(default_p.default, true);
    });
}

#[test]
fn node_switches_farming_policy_when_marked_as_certified_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_twin();
        create_farm();

        create_node();

        let n1 = TfgridModule::nodes(1).unwrap();
        let assigned_policy = TfgridModule::farming_policies_map(n1.farming_policy_id);
        assert_eq!(assigned_policy.node_certification, NodeCertification::Diy);

        assert_ok!(TfgridModule::add_node_certifier(
            RawOrigin::Root.into(),
            alice()
        ));
        assert_ok!(TfgridModule::set_node_certification(
            RuntimeOrigin::signed(alice()),
            1,
            NodeCertification::Certified,
        ));

        let n1 = TfgridModule::nodes(1).unwrap();
        let assigned_policy = TfgridModule::farming_policies_map(n1.farming_policy_id);
        assert_eq!(
            assigned_policy.node_certification,
            NodeCertification::Certified
        );
    });
}

#[test]
fn node_switches_farming_policy_when_marked_as_certified_toggle_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_twin();
        create_farm();

        create_node();

        let n1 = TfgridModule::nodes(1).unwrap();
        let assigned_policy = TfgridModule::farming_policies_map(n1.farming_policy_id);
        assert_eq!(assigned_policy.node_certification, NodeCertification::Diy);

        assert_ok!(TfgridModule::add_node_certifier(
            RawOrigin::Root.into(),
            alice()
        ));
        assert_ok!(TfgridModule::set_node_certification(
            RuntimeOrigin::signed(alice()),
            1,
            NodeCertification::Certified,
        ));

        let n1 = TfgridModule::nodes(1).unwrap();
        let assigned_policy = TfgridModule::farming_policies_map(n1.farming_policy_id);
        assert_eq!(
            assigned_policy.node_certification,
            NodeCertification::Certified
        );

        assert_ok!(TfgridModule::set_node_certification(
            RuntimeOrigin::signed(alice()),
            1,
            NodeCertification::Diy,
        ));
        let n1 = TfgridModule::nodes(1).unwrap();
        let assigned_policy = TfgridModule::farming_policies_map(n1.farming_policy_id);
        assert_eq!(assigned_policy.node_certification, NodeCertification::Diy);
    });
}

#[test]
fn node_switches_farming_policy_when_marked_as_certified_and_gold_farm_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_twin();
        create_farm();

        create_node();

        let n1 = TfgridModule::nodes(1).unwrap();
        let assigned_policy = TfgridModule::farming_policies_map(n1.farming_policy_id);
        assert_eq!(assigned_policy.node_certification, NodeCertification::Diy);
        assert_eq!(
            assigned_policy.farm_certification,
            FarmCertification::NotCertified
        );

        // Mark farm as gold
        assert_ok!(TfgridModule::set_farm_certification(
            RawOrigin::Root.into(),
            1,
            FarmCertification::Gold
        ));
        assert_ok!(TfgridModule::add_node_certifier(
            RawOrigin::Root.into(),
            alice()
        ));
        // Mark node as certified
        assert_ok!(TfgridModule::set_node_certification(
            RuntimeOrigin::signed(alice()),
            1,
            NodeCertification::Certified,
        ));

        let n1 = TfgridModule::nodes(1).unwrap();
        let assigned_policy = TfgridModule::farming_policies_map(n1.farming_policy_id);
        assert_eq!(
            assigned_policy.node_certification,
            NodeCertification::Certified
        );
        assert_eq!(assigned_policy.farm_certification, FarmCertification::Gold);
    });
}

#[test]
fn sort_policies() {
    ExternalityBuilder::build().execute_with(|| {
        let name = "f1".as_bytes().to_vec();
        let f1 = super::types::FarmingPolicy {
            version: 1,
            id: 1,
            name,
            cu: 12,
            su: 15,
            nu: 10,
            ipv4: 8,
            minimal_uptime: 9999,
            policy_created: System::block_number(),
            policy_end: System::block_number() + 100,
            immutable: true,
            default: true,
            node_certification: NodeCertification::Diy,
            farm_certification: FarmCertification::Gold,
        };

        let name = "f2".as_bytes().to_vec();
        let f2 = super::types::FarmingPolicy {
            version: 1,
            id: 2,
            name,
            cu: 12,
            su: 15,
            nu: 10,
            ipv4: 8,
            minimal_uptime: 9999,
            policy_created: System::block_number(),
            policy_end: System::block_number() + 100,
            immutable: true,
            default: true,
            node_certification: NodeCertification::Diy,
            farm_certification: FarmCertification::NotCertified,
        };

        let name = "f3".as_bytes().to_vec();
        let f3 = super::types::FarmingPolicy {
            version: 1,
            id: 3,
            name,
            cu: 12,
            su: 15,
            nu: 10,
            ipv4: 8,
            minimal_uptime: 9999,
            policy_created: System::block_number(),
            policy_end: System::block_number() + 100,
            immutable: true,
            default: true,
            node_certification: NodeCertification::Certified,
            farm_certification: FarmCertification::Gold,
        };

        let name = "f4".as_bytes().to_vec();
        let f4 = super::types::FarmingPolicy {
            version: 1,
            id: 4,
            name,
            cu: 12,
            su: 15,
            nu: 10,
            ipv4: 8,
            minimal_uptime: 9999,
            policy_created: System::block_number(),
            policy_end: System::block_number() + 100,
            immutable: true,
            default: true,
            node_certification: NodeCertification::Certified,
            farm_certification: FarmCertification::NotCertified,
        };

        println!("\nbefore sort");
        let mut policies = vec![f1, f2, f3, f4];
        for p in policies.iter() {
            println!("policy: {:?}", p);
        }

        println!("\nafter sort");
        policies.sort();
        policies.reverse();
        for p in policies.iter() {
            println!("policy: {:?}", p);
        }

        // let name = "c1_test".as_bytes().to_vec();
        // assert_ok!(TfgridModule::create_farming_policy(
        //     RawOrigin::Root.into(),
        //     name,
        //     12,
        //     15,
        //     10,
        //     8,
        //     NodeCertification::Certified
        // ));

        // create_node();

        // let node = TfgridModule::nodes(1).unwrap();
        // // farming policy set on the node should be 3
        // // as we created the last DIY policy with id 3
        // assert_eq!(node.farming_policy_id, 3);
    });
}

#[test]
fn test_create_and_update_policy() {
    new_test_ext().execute_with(|| {
        let su_policy = super::types::Policy {
            value: 150000,
            unit: super::types::Unit::Gigabytes,
        };
        let nu_policy = super::types::Policy {
            value: 1000,
            unit: super::types::Unit::Gigabytes,
        };
        let cu_policy = super::types::Policy {
            value: 300000,
            unit: super::types::Unit::Gigabytes,
        };
        let ipu_policy = super::types::Policy {
            value: 50000,
            unit: super::types::Unit::Gigabytes,
        };
        let unique_name_policy = super::types::Policy {
            value: 10000,
            unit: super::types::Unit::Gigabytes,
        };
        let domain_name_policy = super::types::Policy {
            value: 20000,
            unit: super::types::Unit::Gigabytes,
        };
        let name = String::from("policy_1").as_bytes().to_vec();
        TfgridModule::create_pricing_policy(
            RawOrigin::Root.into(),
            name.clone(),
            su_policy.clone(),
            cu_policy.clone(),
            nu_policy.clone(),
            ipu_policy.clone(),
            unique_name_policy.clone(),
            domain_name_policy.clone(),
            bob(),
            bob(),
            50,
        )
        .unwrap();

        // get policy id
        let policy_id = TfgridModule::pricing_policies_by_name_id(name.clone());
        // Try updating policy with the same name
        let updated_nu_policy = super::types::Policy {
            value: 900,
            unit: super::types::Unit::Gigabytes,
        };
        TfgridModule::update_pricing_policy(
            RawOrigin::Root.into(),
            policy_id.clone(),
            name.clone(),
            su_policy.clone(),
            cu_policy.clone(),
            updated_nu_policy.clone(),
            ipu_policy.clone(),
            unique_name_policy.clone(),
            domain_name_policy.clone(),
            bob(),
            bob(),
            50,
        )
        .unwrap();
        // Get policy and make sure it is updated
        let policy = TfgridModule::pricing_policies(policy_id.clone()).unwrap();
        assert_eq!(
            policy.name.clone(),
            name.clone(),
            "policy name didn't match"
        );
        assert_eq!(policy.id.clone(), policy_id.clone());
        assert_eq!(policy.nu, updated_nu_policy);

        // test updating policy name
        let new_name = String::from("policy_1_updated").as_bytes().to_vec();
        let updated_su_policy = super::types::Policy {
            value: 500,
            unit: super::types::Unit::Gigabytes,
        };
        TfgridModule::update_pricing_policy(
            RawOrigin::Root.into(),
            policy_id.clone(),
            new_name.clone(),
            updated_su_policy.clone(),
            cu_policy.clone(),
            updated_nu_policy.clone(),
            ipu_policy.clone(),
            unique_name_policy.clone(),
            domain_name_policy.clone(),
            bob(),
            bob(),
            50,
        )
        .unwrap();
        let policy = TfgridModule::pricing_policies(policy_id.clone()).unwrap();
        assert_eq!(
            policy.name.clone(),
            new_name.clone(),
            "policy name didn't match"
        );
        assert_eq!(policy.id.clone(), policy_id.clone());
        assert_eq!(policy.su, updated_su_policy);

        // Test updating the name that conflicts with existing policy
        let policy2_name = String::from("policy_2").as_bytes().to_vec();
        TfgridModule::create_pricing_policy(
            RawOrigin::Root.into(),
            policy2_name.clone(),
            su_policy.clone(),
            cu_policy.clone(),
            nu_policy.clone(),
            ipu_policy.clone(),
            unique_name_policy.clone(),
            domain_name_policy.clone(),
            bob(),
            bob(),
            50,
        )
        .unwrap();
        let policy2_id = TfgridModule::pricing_policies_by_name_id(policy2_name.clone());
        //update name to existing name should fail
        assert_noop!(
            TfgridModule::update_pricing_policy(
                RawOrigin::Root.into(),
                policy2_id.clone(),
                new_name.clone(),
                updated_su_policy.clone(),
                cu_policy.clone(),
                updated_nu_policy.clone(),
                ipu_policy.clone(),
                unique_name_policy.clone(),
                domain_name_policy.clone(),
                bob(),
                bob(),
                50,
            ),
            Error::<TestRuntime>::PricingPolicyWithDifferentIdExists
        );
    });
}

#[test]
fn test_set_zos_version() {
    ExternalityBuilder::build().execute_with(|| {
        let zos_version = "1.0.0".as_bytes().to_vec();
        assert_ok!(TfgridModule::set_zos_version(
            RawOrigin::Root.into(),
            zos_version.clone(),
        ));

        let saved_zos_version = TfgridModule::zos_version();
        assert_eq!(saved_zos_version, zos_version);

        let our_events = System::events();
        assert_eq!(
            our_events.contains(&record(MockEvent::TfgridModule(
                TfgridEvent::<TestRuntime>::ZosVersionUpdated(zos_version)
            ))),
            true
        );
    })
}

#[test]
fn test_set_invalid_zos_version_fails() {
    ExternalityBuilder::build().execute_with(|| {
        let zos_version = "1.0.0".as_bytes().to_vec();
        assert_ok!(TfgridModule::set_zos_version(
            RawOrigin::Root.into(),
            zos_version.clone(),
        ));

        // try to set zos version with the same version that is already set
        assert_noop!(
            TfgridModule::set_zos_version(RawOrigin::Root.into(), TfgridModule::zos_version()),
            Error::<TestRuntime>::InvalidZosVersion,
        );
    })
}

fn create_entity() {
    let name = b"foobar".to_vec();
    let country = get_country_name_input(b"Belgium");
    let city = get_city_name_input(b"Ghent");

    let signature = sign_create_entity(name.clone(), country.to_vec(), city.to_vec());
    assert_ok!(TfgridModule::create_entity(
        RuntimeOrigin::signed(alice()),
        test_ed25519(),
        name,
        country,
        city,
        signature,
    ));
}

fn create_entity_sr() {
    let name = b"foobar".to_vec();
    let country = get_country_name_input(b"Belgium");
    let city = get_city_name_input(b"Ghent");

    let signature = sign_create_entity_sr(name.clone(), country.to_vec(), city.to_vec());
    assert_ok!(TfgridModule::create_entity(
        RuntimeOrigin::signed(alice()),
        test_sr25519(),
        name,
        country,
        city,
        signature,
    ));
}

fn create_twin() {
    assert_ok!(TfgridModule::user_accept_tc(
        RuntimeOrigin::signed(alice()),
        get_document_link_input(b"some_link"),
        get_document_hash_input(b"some_hash"),
    ));

    let relay = get_relay_input(b"somerelay.io");
    let pk =
        get_public_key_input(b"0x6c8fd181adc178cea218e168e8549f0b0ff30627c879db9eac4318927e87c901");

    assert_ok!(TfgridModule::create_twin(
        RuntimeOrigin::signed(alice()),
        relay,
        pk,
    ));
}

fn create_twin_bob() {
    assert_ok!(TfgridModule::user_accept_tc(
        RuntimeOrigin::signed(bob()),
        get_document_link_input(b"some_link"),
        get_document_hash_input(b"some_hash"),
    ));

    let relay = get_relay_input(b"somerelay.io");
    let pk =
        get_public_key_input(b"0x6c8fd181adc178cea218e168e8549f0b0ff30627c879db9eac4318927e87c901");

    assert_ok!(TfgridModule::create_twin(
        RuntimeOrigin::signed(bob()),
        relay,
        pk
    ));
}

fn create_farm() {
    let farm_name = get_farm_name_input(b"test_farm");

    let mut pub_ips: PublicIpListInput<TestRuntime> = bounded_vec![];

    let ip = get_public_ip_ip_input(b"185.206.122.33/24");
    let gw = get_public_ip_gw_input(b"185.206.122.1");

    pub_ips.try_push(IP4 { ip, gw }).unwrap();

    assert_ok!(TfgridModule::create_farm(
        RuntimeOrigin::signed(alice()),
        farm_name,
        pub_ips,
    ));

    create_farming_policies()
}

fn create_farm2() {
    let farm_name = get_farm_name_input(b"test_farm2");

    let mut pub_ips: PublicIpListInput<TestRuntime> = bounded_vec![];

    let ip = get_public_ip_ip_input(b"185.206.122.33/24");
    let gw = get_public_ip_gw_input(b"185.206.122.1");

    pub_ips.try_push(IP4 { ip, gw }).unwrap();

    assert_ok!(TfgridModule::create_farm(
        RuntimeOrigin::signed(alice()),
        farm_name,
        pub_ips,
    ));

    create_farming_policies()
}

fn create_farm_bob() {
    let farm_name = get_farm_name_input(b"bob_farm");

    let mut pub_ips: PublicIpListInput<TestRuntime> = bounded_vec![];

    let ip = get_public_ip_ip_input(b"185.206.122.33/24");
    let gw = get_public_ip_gw_input(b"185.206.122.1");

    pub_ips.try_push(IP4 { ip, gw }).unwrap();

    assert_ok!(TfgridModule::create_farm(
        RuntimeOrigin::signed(bob()),
        farm_name,
        pub_ips,
    ));
}

fn create_node() {
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

    let interfaces: InterfaceInput<TestRuntime> = bounded_vec![];

    assert_ok!(TfgridModule::create_node(
        RuntimeOrigin::signed(alice()),
        1,
        resources,
        location,
        interfaces,
        true,
        true,
        Some(b"Default String".to_vec().try_into().unwrap()),
    ));
}

fn create_extra_node() {
    let resources = ResourcesInput {
        hru: 1024 * GIGABYTE,
        sru: 512 * GIGABYTE,
        cru: 8,
        mru: 16 * GIGABYTE,
    };

    // random location
    let location = LocationInput {
        city: get_city_name_input(b"Rio de Janeiro"),
        country: get_country_name_input(b"Brazil"),
        latitude: get_latitude_input(b"43.1868"),
        longitude: get_longitude_input(b"22.9694"),
    };

    assert_ok!(TfgridModule::create_node(
        RuntimeOrigin::signed(bob()),
        1,
        resources,
        location,
        bounded_vec![],
        true,
        true,
        None,
    ));
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

    let name = "f4".as_bytes().to_vec();
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

fn create_custom_farming_policies() {
    let name = "f5".as_bytes().to_vec();
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
        false,
        NodeCertification::Diy,
        FarmCertification::NotCertified,
    ));

    let name = "f6".as_bytes().to_vec();
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
        false,
        NodeCertification::Certified,
        FarmCertification::Gold,
    ));
}

fn record(event: RuntimeEvent) -> EventRecord<RuntimeEvent, H256> {
    EventRecord {
        phase: Phase::Initialization,
        event,
        topics: vec![],
    }
}

// Attach given farming policy to farm 1 that contains node 1
fn test_attach_farming_policy_flow(farming_policy_id: u32) {
    create_twin();
    create_farm();
    create_node();

    let farm_id = 1;
    let node_id = 1;

    // Set limit with farming policy to be attached to farm
    let fp = TfgridModule::farming_policies_map(farming_policy_id);
    let limit = FarmingPolicyLimit {
        farming_policy_id: fp.id,
        cu: Some(20),
        su: Some(2),
        end: Some(1654058949),
        node_certification: false,
        node_count: Some(10),
    };

    // farm has no farming policy limits at this stage
    let mut farm = TfgridModule::farms(farm_id).unwrap();
    assert_eq!(farm.farming_policy_limits.is_none(), true);

    // farm is 'not certified'
    // node is 'diy'
    // So without farming policy attached to farm
    // the auto-defined farming policy id for node is 2
    // see fn get_farming_policy()
    // and fn create_farming_policies()
    let mut node = TfgridModule::nodes(node_id).unwrap();
    assert_eq!(node.farming_policy_id, 2);
    assert_eq!(node.certification, NodeCertification::default());

    // Provide enough CU and SU limits to avoid attaching default policy to node
    // For node: [CU = 20; SU = 2]
    assert_eq!(node.resources.get_cu() <= limit.cu.unwrap(), true);
    assert_eq!(node.resources.get_su() <= limit.su.unwrap(), true);

    // Link farming policy to farm
    assert_ok!(TfgridModule::attach_policy_to_farm(
        RawOrigin::Root.into(),
        farm.id,
        Some(limit.clone())
    ));

    // Get updated farm
    farm = TfgridModule::farms(farm_id).unwrap();
    // Check updated farm limits
    let farm_limit = farm.clone().farming_policy_limits.unwrap();
    assert_eq!(farm_limit.farming_policy_id, limit.farming_policy_id);
    assert_eq!(farm_limit.cu, Some(0)); // No more CU available
    assert_eq!(farm_limit.su, Some(0)); // No more SU available
    assert_eq!(farm_limit.end, limit.end);
    assert_eq!(farm_limit.node_certification, limit.node_certification);
    assert_eq!(farm_limit.node_count, Some(9)); // Remains 9 spots for nodes
    assert_eq!(farm.certification, fp.farm_certification);

    // Get updated node
    node = TfgridModule::nodes(node_id).unwrap();
    // farming policy id 2 is a default farming policy
    // so it should have been overriden
    // see attach_policy_to_farm()
    assert_eq!(node.farming_policy_id, fp.id);
    assert_eq!(node.certification, fp.node_certification);

    // Check events sequence
    let our_events = System::events();
    assert_eq!(
        our_events[our_events.len() - 3],
        record(MockEvent::TfgridModule(
            TfgridEvent::<TestRuntime>::FarmUpdated(farm.clone())
        ))
    );
    assert_eq!(
        our_events[our_events.len() - 2],
        record(MockEvent::TfgridModule(
            TfgridEvent::<TestRuntime>::NodeUpdated(node)
        ))
    );
    assert_eq!(
        our_events[our_events.len() - 1],
        record(MockEvent::TfgridModule(
            TfgridEvent::<TestRuntime>::FarmingPolicySet(farm.id, Some(limit))
        ))
    );
}
