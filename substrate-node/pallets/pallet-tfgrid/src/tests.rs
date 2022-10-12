use super::Event as TfgridEvent;
use crate::{
    mock::Event as MockEvent, mock::*, types::PublicIpInput, Error, InterfaceInput,
    InterfaceIpsInput, PublicIpListInput,
};
use frame_support::{assert_noop, assert_ok, bounded_vec, BoundedVec};
use frame_system::{EventRecord, Phase, RawOrigin};
use sp_core::H256;
use tfchain_support::resources;
use tfchain_support::types::{
    FarmCertification, FarmingPolicyLimit, Interface, Location, NodeCertification, PublicConfig,
    Resources, IP,
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

        let country = "Belgium".as_bytes().to_vec();
        let city = "Ghent".as_bytes().to_vec();
        // Change name to barfoo
        let name = "barfoo".as_bytes().to_vec();

        assert_ok!(TfgridModule::update_entity(
            Origin::signed(test_ed25519()),
            name,
            country,
            city
        ));
    });
}

#[test]
fn test_update_entity_fails_if_signed_by_someone_else() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();

        let country = "Belgium".as_bytes().to_vec();
        let city = "Ghent".as_bytes().to_vec();
        // Change name to barfoo
        let name = "barfoo".as_bytes().to_vec();

        assert_noop!(
            TfgridModule::update_entity(Origin::signed(bob()), name, country, city),
            Error::<TestRuntime>::EntityNotExists
        );
    });
}

#[test]
fn test_create_entity_double_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();

        let name = "foobar".as_bytes().to_vec();
        let country = "Belgium".as_bytes().to_vec();
        let city = "Ghent".as_bytes().to_vec();
        let signature = sign_create_entity(name.clone(), country.clone(), city.clone());

        assert_noop!(
            TfgridModule::create_entity(
                Origin::signed(alice()),
                test_ed25519(),
                name,
                country,
                city,
                signature
            ),
            Error::<TestRuntime>::EntityWithNameExists
        );
    });
}

#[test]
fn test_create_entity_double_fails_with_same_pubkey() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();

        let name = "barfoo".as_bytes().to_vec();
        let country = "Belgium".as_bytes().to_vec();
        let city = "Ghent".as_bytes().to_vec();

        let signature = sign_create_entity(name.clone(), country.clone(), city.clone());

        assert_noop!(
            TfgridModule::create_entity(
                Origin::signed(alice()),
                test_ed25519(),
                name,
                country,
                city,
                signature
            ),
            Error::<TestRuntime>::EntityWithPubkeyExists
        );
    });
}

#[test]
fn test_delete_entity_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();

        assert_ok!(TfgridModule::delete_entity(Origin::signed(test_ed25519())));
    });
}

#[test]
fn test_delete_entity_fails_if_signed_by_someone_else() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();

        assert_noop!(
            TfgridModule::delete_entity(Origin::signed(bob())),
            Error::<TestRuntime>::EntityNotExists
        );
    });
}

#[test]
fn test_create_twin_works() {
    ExternalityBuilder::build().execute_with(|| {
        let document = "some_link".as_bytes().to_vec();
        let hash = "some_hash".as_bytes().to_vec();

        assert_ok!(TfgridModule::user_accept_tc(
            Origin::signed(test_ed25519()),
            document,
            hash,
        ));

        let ip = get_twin_ip(b"::1");
        assert_ok!(TfgridModule::create_twin(
            Origin::signed(test_ed25519()),
            ip.clone().0
        ));
    });
}

#[test]
fn test_delete_twin_works() {
    ExternalityBuilder::build().execute_with(|| {
        let document = "some_link".as_bytes().to_vec();
        let hash = "some_hash".as_bytes().to_vec();

        assert_ok!(TfgridModule::user_accept_tc(
            Origin::signed(alice()),
            document,
            hash,
        ));

        let ip = get_twin_ip(b"::1");
        assert_ok!(TfgridModule::create_twin(
            Origin::signed(alice()),
            ip.clone().0
        ));

        let twin_id = 1;
        assert_ok!(TfgridModule::delete_twin(Origin::signed(alice()), twin_id));
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

        assert_ok!(TfgridModule::delete_node_farm(Origin::signed(alice()), 1));

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
            TfgridModule::delete_node_farm(Origin::signed(bob()), 1),
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
            Origin::signed(bob()),
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
            TfgridModule::add_twin_entity(Origin::signed(bob()), twin_id, entity_id, signature),
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
            Origin::signed(bob()),
            twin_id,
            entity_id,
            signature.clone()
        ));

        assert_noop!(
            TfgridModule::add_twin_entity(Origin::signed(bob()), twin_id, entity_id, signature),
            Error::<TestRuntime>::EntityWithSignatureAlreadyExists
        );
    });
}

#[test]
fn test_create_twin_double_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_twin();

        let ip = get_twin_ip(b"::1");
        assert_noop!(
            TfgridModule::create_twin(Origin::signed(alice()), ip.clone().0),
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

        let farm_name = BoundedVec::try_from(b"test.farm".to_vec()).unwrap();
        assert_noop!(
            TfgridModule::create_farm(Origin::signed(alice()), farm_name, bounded_vec![]),
            Error::<TestRuntime>::InvalidFarmName
        );

        let farm_name = BoundedVec::try_from(b"test farm".to_vec()).unwrap();
        assert_noop!(
            TfgridModule::create_farm(Origin::signed(alice()), farm_name, bounded_vec![]),
            Error::<TestRuntime>::InvalidFarmName
        );

        let farm_name = BoundedVec::try_from(b"".to_vec()).unwrap();
        assert_noop!(
            TfgridModule::create_farm(Origin::signed(alice()), farm_name, bounded_vec![]),
            Error::<TestRuntime>::FarmNameTooShort
        );

        let farm_name = BoundedVec::try_from(b"12".to_vec()).unwrap();
        assert_noop!(
            TfgridModule::create_farm(Origin::signed(alice()), farm_name, bounded_vec![]),
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
        let farm_name = get_farm_name(b"bob_farm");
        assert_ok!(TfgridModule::create_farm(
            Origin::signed(bob()),
            farm_name.0.clone(),
            bounded_vec![]
        ));

        let farm_name = get_farm_name(b"bob_updated_farm");
        assert_ok!(TfgridModule::update_farm(
            Origin::signed(bob()),
            2,
            farm_name.0.clone(),
            1
        ));
    });
}

#[test]
fn test_update_farm_existing_name_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_twin();

        let farm_name = get_farm_name(b"alice_farm");

        assert_ok!(TfgridModule::create_farm(
            Origin::signed(alice()),
            farm_name.0.clone(),
            bounded_vec![]
        ));

        create_twin_bob();
        let farm_name = get_farm_name(b"bob_farm");
        assert_ok!(TfgridModule::create_farm(
            Origin::signed(bob()),
            farm_name.0.clone(),
            bounded_vec![]
        ));

        let farm_name = get_farm_name(b"alice_farm");
        assert_noop!(
            TfgridModule::update_farm(Origin::signed(bob()), 2, farm_name.0.clone(), 1),
            Error::<TestRuntime>::InvalidFarmName
        );
    });
}

#[test]
fn test_create_farm_with_double_ip_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();

        let farm_name = get_farm_name(b"test_farm");

        let mut pub_ips: PublicIpListInput<TestRuntime> = bounded_vec![];

        let ip = get_public_ip_ip(&"185.206.122.33/24".as_bytes().to_vec()).0;
        let gw = get_public_ip_gateway(&"185.206.122.1".as_bytes().to_vec()).0;

        pub_ips
            .try_push(PublicIpInput {
                ip: ip.clone(),
                gw: gw.clone(),
            })
            .unwrap();
        pub_ips.try_push(PublicIpInput { ip, gw }).unwrap();

        assert_noop!(
            TfgridModule::create_farm(Origin::signed(alice()), farm_name.0.clone(), pub_ips),
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
            Origin::signed(alice()),
            1,
            get_public_ip_ip(&"185.206.122.125/16".as_bytes().to_vec()).0,
            get_public_ip_gateway(&"185.206.122.1".as_bytes().to_vec()).0
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
                Origin::signed(alice()),
                1,
                "185.206.122.125".as_bytes().to_vec().try_into().unwrap(),
                get_public_ip_gateway(&"185.206.122.1".as_bytes().to_vec()).0
            ),
            Error::<TestRuntime>::InvalidPublicIP
        );
    });
}

#[test]
fn test_delete_farm_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        assert_noop!(
            TfgridModule::delete_farm(Origin::signed(alice()), 1),
            Error::<TestRuntime>::MethodIsDeprecated
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
            Origin::signed(alice()),
            1,
            get_public_ip_ip(&"185.206.122.125/16".as_bytes().to_vec()).0,
            get_public_ip_gateway(&"185.206.122.1".as_bytes().to_vec()).0
        ));

        assert_noop!(
            TfgridModule::add_farm_ip(
                Origin::signed(alice()),
                1,
                get_public_ip_ip(&"185.206.122.125/16".as_bytes().to_vec()).0,
                get_public_ip_gateway(&"185.206.122.1".as_bytes().to_vec()).0
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

        let ip = get_twin_ip(b"::1");
        assert_ok!(TfgridModule::update_twin(
            Origin::signed(alice()),
            ip.clone().0
        ));
    });
}

#[test]
fn test_update_twin_fails_if_signed_by_someone_else() {
    ExternalityBuilder::build().execute_with(|| {
        let document = "some_link".as_bytes().to_vec();
        let hash = "some_hash".as_bytes().to_vec();

        assert_ok!(TfgridModule::user_accept_tc(
            Origin::signed(alice()),
            document,
            hash,
        ));

        let ip = get_twin_ip(b"::1");
        assert_ok!(TfgridModule::create_twin(
            Origin::signed(alice()),
            ip.clone().0
        ));

        let ip = get_twin_ip(b"::1");
        assert_noop!(
            TfgridModule::update_twin(Origin::signed(bob()), ip.clone().0),
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

        let farm_name = get_farm_name(b"test_farm");

        let pub_ips: PublicIpListInput<TestRuntime> = bounded_vec![];

        assert_noop!(
            TfgridModule::create_farm(Origin::signed(alice()), farm_name.0.clone(), pub_ips),
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
            Origin::signed(alice()),
            1,
            addr
        ));

        let addr2 = "some_other_address".as_bytes().to_vec();
        assert_ok!(TfgridModule::add_stellar_payout_v2address(
            Origin::signed(alice()),
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

        let country = "Belgium".as_bytes().to_vec();
        let city = "Ghent".as_bytes().to_vec();

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
        assert_ok!(TfgridModule::update_node(
            Origin::signed(alice()),
            1,
            2,
            resources,
            location,
            country,
            city,
            Vec::new().try_into().unwrap(),
            true,
            true,
            "some_serial".as_bytes().to_vec(),
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
            Origin::signed(alice()),
            1,
            NodeCertification::Certified
        ));

        let node = TfgridModule::nodes(1).unwrap();
        assert_eq!(node.certification, NodeCertification::Certified);

        // Change cores to 2
        let mut node_resources = node.resources;
        node_resources.cru = 2;

        assert_ok!(TfgridModule::update_node(
            Origin::signed(alice()),
            1,
            1,
            node_resources,
            node.location,
            node.country,
            node.city,
            Vec::new().try_into().unwrap(),
            true,
            true,
            "some_serial".as_bytes().to_vec(),
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
            Origin::signed(alice()),
            1,
            NodeCertification::Certified
        ));

        let node = TfgridModule::nodes(1).unwrap();
        assert_eq!(node.certification, NodeCertification::Certified);

        // Don't change resources
        assert_ok!(TfgridModule::update_node(
            Origin::signed(alice()),
            1,
            1,
            node.resources,
            node.location,
            node.country,
            node.city,
            Vec::new().try_into().unwrap(),
            true,
            true,
            "some_serial".as_bytes().to_vec(),
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

        let country = "Belgium".as_bytes().to_vec();
        let city = "Ghent".as_bytes().to_vec();

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

        let mut interface_ips: InterfaceIpsInput<TestRuntime> = bounded_vec![];
        let intf_ip_1 = get_interface_ip(&"10.2.3.3".as_bytes().to_vec());
        interface_ips.try_push(intf_ip_1.0).unwrap();

        let name = get_interface_name(&"zos".as_bytes().to_vec()).0;
        let mac = get_interface_mac(&"00:00:5e:00:53:af".as_bytes().to_vec()).0;

        let interface = Interface {
            name,
            mac,
            ips: interface_ips,
        };

        let mut interfaces: InterfaceInput<TestRuntime> = bounded_vec![];
        interfaces.try_push(interface).unwrap();

        assert_ok!(TfgridModule::create_node(
            Origin::signed(alice()),
            1,
            resources,
            location,
            country,
            city,
            interfaces,
            true,
            true,
            "some_serial".as_bytes().to_vec()
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
            Origin::signed(alice()),
            1,
            NodeCertification::Certified
        ));
        let node = TfgridModule::nodes(1).unwrap();
        assert_eq!(node.certification, NodeCertification::Certified);

        assert_ok!(TfgridModule::set_node_certification(
            Origin::signed(alice()),
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
                Origin::signed(alice()),
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
        assert_ok!(TfgridModule::report_uptime(Origin::signed(alice()), 500));
    });
}

#[test]
fn node_add_public_config_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();

        let ipv4 = get_pub_config_ip4(&"185.206.122.33/24".as_bytes().to_vec());
        let ipv6 = get_pub_config_ip6(&"2a10:b600:1::0cc4:7a30:65b5/64".as_bytes().to_vec());
        let gw4 = get_pub_config_gw4(&"185.206.122.1".as_bytes().to_vec());
        let gw6 = get_pub_config_gw6(&"2a10:b600:1::1".as_bytes().to_vec());

        let pub_config = PublicConfig {
            ip4: IP {
                ip: ipv4.clone().0,
                gw: gw4.clone().0,
            },
            ip6: Some(IP {
                ip: ipv6.clone().0,
                gw: gw6.clone().0,
            }),
            domain: Some("some-domain".as_bytes().to_vec().try_into().unwrap()),
        };

        assert_ok!(TfgridModule::add_node_public_config(
            Origin::signed(alice()),
            1,
            1,
            Some(pub_config.clone())
        ));

        let node = TfgridModule::nodes(1).unwrap();
        assert_eq!(
            node.public_config,
            Some(PublicConfig {
                ip4: IP { ip: ipv4, gw: gw4 },
                ip6: Some(IP { ip: ipv6, gw: gw6 }),
                domain: Some("some-domain".as_bytes().to_vec().try_into().unwrap()),
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

        let ipv4 = get_pub_config_ip4(&"185.206.122.33/24".as_bytes().to_vec());
        let gw4 = get_pub_config_gw4(&"185.206.122.1".as_bytes().to_vec());

        let pub_config = PublicConfig {
            ip4: IP {
                ip: ipv4.clone().0,
                gw: gw4.clone().0,
            },
            ip6: None,
            domain: None,
        };

        assert_ok!(TfgridModule::add_node_public_config(
            Origin::signed(alice()),
            1,
            1,
            Some(pub_config.clone())
        ));

        let node = TfgridModule::nodes(1).unwrap();
        assert_eq!(
            node.public_config,
            Some(PublicConfig {
                ip4: IP { ip: ipv4, gw: gw4 },
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

        let ipv4 = get_pub_config_ip4(&"185.206.122.33/24".as_bytes().to_vec());
        let ipv6 = get_pub_config_ip6(&"2a10:b600:1::0cc4:7a30:65b5/64".as_bytes().to_vec());
        let gw4 = get_pub_config_gw4(&"185.206.122.1".as_bytes().to_vec());
        let gw6 = get_pub_config_gw6(&"2a10:b600:1::1".as_bytes().to_vec());

        let pub_config = PublicConfig {
            ip4: IP {
                ip: ipv4.clone().0,
                gw: gw4.clone().0,
            },
            ip6: Some(IP {
                ip: ipv6.clone().0,
                gw: gw6.clone().0,
            }),
            domain: Some("some-domain".as_bytes().to_vec().try_into().unwrap()),
        };

        assert_noop!(
            TfgridModule::add_node_public_config(
                Origin::signed(bob()),
                1,
                1,
                Some(pub_config.clone())
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

        let ipv4 = get_pub_config_ip4(&"185.206.122.33/24".as_bytes().to_vec());
        let ipv6 = get_pub_config_ip6(&"2a10:b600:1::0cc4:7a30:65b5/64".as_bytes().to_vec());
        let gw4 = get_pub_config_gw4(&"185.206.122.1".as_bytes().to_vec());
        let gw6 = get_pub_config_gw6(&"2a10:b600:1::1".as_bytes().to_vec());

        let pub_config = PublicConfig {
            ip4: IP {
                ip: ipv4.clone().0,
                gw: gw4.clone().0,
            },
            ip6: Some(IP {
                ip: ipv6.clone().0,
                gw: gw6.clone().0,
            }),
            domain: Some("some-domain".as_bytes().to_vec().try_into().unwrap()),
        };

        assert_ok!(TfgridModule::add_node_public_config(
            Origin::signed(alice()),
            1,
            1,
            Some(pub_config.clone())
        ));

        let node = TfgridModule::nodes(1).unwrap();
        assert_eq!(
            node.public_config,
            Some(PublicConfig {
                ip4: IP { ip: ipv4, gw: gw4 },
                ip6: Some(IP { ip: ipv6, gw: gw6 }),
                domain: Some("some-domain".as_bytes().to_vec().try_into().unwrap()),
            })
        );

        assert_ok!(TfgridModule::add_node_public_config(
            Origin::signed(alice()),
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

        // let ipv4 = get_pub_config_ip4(&"1.1.1.1".as_bytes().to_vec());
        let ipv6 = get_pub_config_ip6(&"2a10:b600:1::0cc4:7a30:65b5/64".as_bytes().to_vec());
        let gw4 = get_pub_config_gw4(&"185.206.122.1".as_bytes().to_vec());
        let gw6 = get_pub_config_gw6(&"2a10:b600:1::1".as_bytes().to_vec());

        let pub_config = PublicConfig {
            ip4: IP {
                ip: "1.1.1.1".as_bytes().to_vec().try_into().unwrap(),
                gw: gw4.clone().0,
            },
            ip6: Some(IP {
                ip: ipv6.clone().0,
                gw: gw6.clone().0,
            }),
            domain: Some("some-domain".as_bytes().to_vec().try_into().unwrap()),
        };

        assert_noop!(
            TfgridModule::add_node_public_config(
                Origin::signed(alice()),
                1,
                1,
                Some(pub_config.clone())
            ),
            Error::<TestRuntime>::IP4ToShort
        );
    });
}

#[test]
#[should_panic(expected = "InvalidIP4")]
fn test_validate_invalid_ip4_1() {
    ExternalityBuilder::build().execute_with(|| {
        TestIP4::try_from("185.206.122.33".as_bytes().to_vec()).expect("fails");
    });
}

#[test]
#[should_panic(expected = "IP4ToShort")]
fn test_validate_invalid_ip4_2() {
    ExternalityBuilder::build().execute_with(|| {
        TestIP4::try_from("185.206".as_bytes().to_vec()).expect("fails");
    });
}

#[test]
#[should_panic(expected = "IP4ToLong")]
fn test_validate_invalid_ip4_3() {
    ExternalityBuilder::build().execute_with(|| {
        TestIP4::try_from("185.206.12.12.1232123".as_bytes().to_vec()).expect("fails");
    });
}

#[test]
#[should_panic(expected = "InvalidIP4")]
fn test_validate_invalid_ip4_4() {
    ExternalityBuilder::build().execute_with(|| {
        TestIP4::try_from("garbage data".as_bytes().to_vec()).expect("fails");
    });
}

#[test]
fn create_node_with_same_pubkey_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();

        // random location
        let location = Location {
            longitude: "12.233213231".as_bytes().to_vec(),
            latitude: "32.323112123".as_bytes().to_vec(),
        };

        let resources = Resources {
            hru: 1,
            sru: 1,
            cru: 1,
            mru: 1,
        };

        let country = "Belgium".as_bytes().to_vec();
        let city = "Ghent".as_bytes().to_vec();

        let interfaces: InterfaceInput<TestRuntime> = bounded_vec![];

        assert_noop!(
            TfgridModule::create_node(
                Origin::signed(alice()),
                1,
                resources,
                location,
                country,
                city,
                interfaces,
                true,
                true,
                "some_serial".as_bytes().to_vec()
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
            Origin::signed(alice()),
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
            Origin::signed(alice()),
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
            Origin::signed(alice()),
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
            Origin::signed(alice()),
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
    let name = "foobar".as_bytes().to_vec();
    let country = "Belgium".as_bytes().to_vec();
    let city = "Ghent".as_bytes().to_vec();

    let signature = sign_create_entity(name.clone(), country.clone(), city.clone());
    assert_ok!(TfgridModule::create_entity(
        Origin::signed(alice()),
        test_ed25519(),
        name,
        country,
        city,
        signature.clone()
    ));
}

fn create_entity_sr() {
    let name = "foobar".as_bytes().to_vec();
    let country = "Belgium".as_bytes().to_vec();
    let city = "Ghent".as_bytes().to_vec();

    let signature = sign_create_entity_sr(name.clone(), country.clone(), city.clone());
    assert_ok!(TfgridModule::create_entity(
        Origin::signed(alice()),
        test_sr25519(),
        name,
        country,
        city,
        signature.clone()
    ));
}

fn create_twin() {
    let document = "some_link".as_bytes().to_vec();
    let hash = "some_hash".as_bytes().to_vec();

    assert_ok!(TfgridModule::user_accept_tc(
        Origin::signed(alice()),
        document,
        hash,
    ));

    let ip = get_twin_ip(b"::1");
    assert_ok!(TfgridModule::create_twin(
        Origin::signed(alice()),
        ip.clone().0
    ));
}

fn create_twin_bob() {
    let document = "some_link".as_bytes().to_vec();
    let hash = "some_hash".as_bytes().to_vec();

    assert_ok!(TfgridModule::user_accept_tc(
        Origin::signed(bob()),
        document,
        hash,
    ));

    let ip = get_twin_ip(b"::1");
    assert_ok!(TfgridModule::create_twin(
        Origin::signed(bob()),
        ip.clone().0
    ));
}

fn create_farm() {
    let farm_name = get_farm_name(b"test_farm");

    let mut pub_ips: PublicIpListInput<TestRuntime> = bounded_vec![];

    let ip = get_public_ip_ip(&"185.206.122.33/24".as_bytes().to_vec()).0;
    let gw = get_public_ip_gateway(&"185.206.122.1".as_bytes().to_vec()).0;

    pub_ips.try_push(PublicIpInput { ip, gw }).unwrap();

    assert_ok!(TfgridModule::create_farm(
        Origin::signed(alice()),
        farm_name.0.clone(),
        pub_ips.clone()
    ));

    create_farming_policies()
}

fn create_farm2() {
    let farm_name = get_farm_name(b"test_farm2");

    let mut pub_ips: PublicIpListInput<TestRuntime> = bounded_vec![];

    let ip = get_public_ip_ip(&"185.206.122.33/24".as_bytes().to_vec()).0;
    let gw = get_public_ip_gateway(&"185.206.122.1".as_bytes().to_vec()).0;

    pub_ips.try_push(PublicIpInput { ip, gw }).unwrap();

    assert_ok!(TfgridModule::create_farm(
        Origin::signed(alice()),
        farm_name.0.clone(),
        pub_ips.clone()
    ));

    create_farming_policies()
}

fn create_node() {
    let country = "Belgium".as_bytes().to_vec();
    let city = "Ghent".as_bytes().to_vec();

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

    let interfaces: InterfaceInput<TestRuntime> = bounded_vec![];

    assert_ok!(TfgridModule::create_node(
        Origin::signed(alice()),
        1,
        resources,
        location,
        country,
        city,
        interfaces,
        true,
        true,
        "some_serial".as_bytes().to_vec()
    ));
}

fn create_extra_node() {
    let country = "Brazil".as_bytes().to_vec();
    let city = "Rio de Janeiro".as_bytes().to_vec();

    // random location
    let location = Location {
        longitude: "43.1868".as_bytes().to_vec(),
        latitude: "22.9694".as_bytes().to_vec(),
    };

    let resources = Resources {
        hru: 1024 * GIGABYTE,
        sru: 512 * GIGABYTE,
        cru: 8,
        mru: 16 * GIGABYTE,
    };

    assert_ok!(TfgridModule::create_node(
        Origin::signed(bob()),
        1,
        resources,
        location,
        country,
        city,
        Vec::new().try_into().unwrap(),
        true,
        true,
        "some_serial".as_bytes().to_vec()
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

fn record(event: Event) -> EventRecord<Event, H256> {
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
    assert_eq!(resources::get_cu(node.resources) <= limit.cu.unwrap(), true);
    assert_eq!(resources::get_su(node.resources) <= limit.su.unwrap(), true);

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
