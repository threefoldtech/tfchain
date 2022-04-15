use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;

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

        let ip = "10.2.3.3";
        assert_ok!(TfgridModule::create_twin(
            Origin::signed(test_ed25519()),
            ip.as_bytes().to_vec()
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

        let ip = "10.2.3.3";
        assert_ok!(TfgridModule::create_twin(
            Origin::signed(alice()),
            ip.as_bytes().to_vec()
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

        assert_ok!(TfgridModule::delete_node_farm(Origin::signed(alice()), 1));
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

        let ip = "10.2.3.3";
        assert_noop!(
            TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()),
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
fn test_create_farm_fails_invalid_name() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();

        let farm_name = "test.farm".as_bytes().to_vec();

        assert_noop!(
            TfgridModule::create_farm(Origin::signed(alice()), farm_name, Vec::new(),),
            Error::<TestRuntime>::InvalidFarmName
        );
    });
}

#[test]
fn test_create_farm_fails_empty_name() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();

        let farm_name = "".as_bytes().to_vec();

        assert_noop!(
            TfgridModule::create_farm(Origin::signed(alice()), farm_name, Vec::new(),),
            Error::<TestRuntime>::InvalidFarmName
        );
    });
}

#[test]
fn test_create_farm_with_double_ip_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();

        let farm_name = "test_farm".as_bytes().to_vec();
        let mut pub_ips = Vec::new();
        pub_ips.push(super::types::PublicIP {
            ip: "1.1.1.0".as_bytes().to_vec(),
            gateway: "1.1.1.1".as_bytes().to_vec(),
            contract_id: 0,
        });
        pub_ips.push(super::types::PublicIP {
            ip: "1.1.1.0".as_bytes().to_vec(),
            gateway: "1.1.1.1".as_bytes().to_vec(),
            contract_id: 0,
        });

        assert_noop!(
            TfgridModule::create_farm(Origin::signed(alice()), farm_name, pub_ips),
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
            "1.1.1.2".as_bytes().to_vec(),
            "1.1.1.1".as_bytes().to_vec()
        ));
    });
}

#[test]
fn test_delete_farm_with_publicips_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        assert_noop!(
            TfgridModule::delete_farm(Origin::signed(alice()), 1),
            Error::<TestRuntime>::CannotDeleteFarmWithPublicIPs
        );
    });
}

#[test]
fn test_delete_farm_without_publicips_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        assert_ok!(TfgridModule::remove_farm_ip(
            Origin::signed(alice()),
            1,
            "1.1.1.0".as_bytes().to_vec()
        ));
        assert_ok!(TfgridModule::delete_farm(Origin::signed(alice()), 1));
    });
}

#[test]
fn test_delete_farm_with_nodes_fails() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();
        // make sure to delete ip first to test only farm with node
        assert_ok!(TfgridModule::remove_farm_ip(
            Origin::signed(alice()),
            1,
            "1.1.1.0".as_bytes().to_vec()
        ));
        assert_noop!(
            TfgridModule::delete_farm(Origin::signed(alice()), 1),
            Error::<TestRuntime>::CannotDeleteFarmWithNodesAssigned
        );
    });
}

#[test]
fn test_delete_farm_without_nodes_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        // make sure to remove public ips
        assert_ok!(TfgridModule::remove_farm_ip(
            Origin::signed(alice()),
            1,
            "1.1.1.0".as_bytes().to_vec()
        ));
        assert_ok!(TfgridModule::delete_farm(Origin::signed(alice()), 1));
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
            "1.1.1.2".as_bytes().to_vec(),
            "1.1.1.1".as_bytes().to_vec()
        ));

        assert_noop!(
            TfgridModule::add_farm_ip(
                Origin::signed(alice()),
                1,
                "1.1.1.2".as_bytes().to_vec(),
                "1.1.1.1".as_bytes().to_vec()
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

        let farm = TfgridModule::farms(1);
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

        let farm = TfgridModule::farms(1);
        assert_eq!(farm.dedicated_farm, true);

        assert_ok!(TfgridModule::set_farm_dedicated(
            RawOrigin::Root.into(),
            1,
            false
        ));

        let farm = TfgridModule::farms(1);
        assert_eq!(farm.dedicated_farm, false);
    })
}

#[test]
fn test_update_twin_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_twin();

        let ip = "some_other_ip".as_bytes().to_vec();
        assert_ok!(TfgridModule::update_twin(Origin::signed(alice()), ip));
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

        let mut ip = "some_ip".as_bytes().to_vec();
        assert_ok!(TfgridModule::create_twin(Origin::signed(alice()), ip));

        ip = "some_other_ip".as_bytes().to_vec();
        assert_noop!(
            TfgridModule::update_twin(Origin::signed(bob()), ip),
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

        let farm_name = "test_farm".as_bytes().to_vec();
        let mut pub_ips = Vec::new();
        pub_ips.push(super::types::PublicIP {
            ip: "1.1.1.0".as_bytes().to_vec(),
            gateway: "1.1.1.1".as_bytes().to_vec(),
            contract_id: 0,
        });

        assert_noop!(
            TfgridModule::create_farm(Origin::signed(alice()), farm_name, pub_ips),
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
fn create_node_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();
    });
}

#[test]
fn set_certification_type_node_works() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();

        assert_ok!(TfgridModule::set_node_certification(
            RawOrigin::Root.into(),
            1,
            super::types::CertificationType::Certified
        ));
        let node = TfgridModule::nodes(1);
        assert_eq!(
            node.certification_type,
            super::types::CertificationType::Certified
        );

        assert_ok!(TfgridModule::set_node_certification(
            RawOrigin::Root.into(),
            1,
            super::types::CertificationType::Diy
        ));
        let node = TfgridModule::nodes(1);
        assert_eq!(
            node.certification_type,
            super::types::CertificationType::Diy
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

        let pub_config = super::types::PublicConfig {
            ipv4: "some_ip".as_bytes().to_vec(),
            ipv6: "some_ip".as_bytes().to_vec(),
            gw4: "some_ip".as_bytes().to_vec(),
            gw6: "some_ip".as_bytes().to_vec(),
            domain: "some_domain".as_bytes().to_vec(),
        };

        assert_ok!(TfgridModule::add_node_public_config(
            Origin::signed(alice()),
            1,
            1,
            pub_config.clone()
        ));

        let node = TfgridModule::nodes(1);
        assert_eq!(node.public_config, Some(pub_config));
    });
}

#[test]
fn node_add_public_config_fails_if_signature_incorrect() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();
        create_node();

        let pub_config = super::types::PublicConfig {
            ipv4: "some_ip".as_bytes().to_vec(),
            ipv6: "some_ip".as_bytes().to_vec(),
            gw4: "some_ip".as_bytes().to_vec(),
            gw6: "some_ip".as_bytes().to_vec(),
            domain: "some_domain".as_bytes().to_vec(),
        };

        assert_noop!(
            TfgridModule::add_node_public_config(Origin::signed(bob()), 1, 1, pub_config.clone()),
            Error::<TestRuntime>::CannotUpdateFarmWrongTwin
        );
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
        let location = super::types::Location {
            longitude: "12.233213231".as_bytes().to_vec(),
            latitude: "32.323112123".as_bytes().to_vec(),
        };

        let resources = super::types::Resources {
            hru: 1,
            sru: 1,
            cru: 1,
            mru: 1,
        };

        let country = "Belgium".as_bytes().to_vec();
        let city = "Ghent".as_bytes().to_vec();

        assert_noop!(
            TfgridModule::create_node(
                Origin::signed(alice()),
                1,
                resources,
                location,
                country,
                city,
                Vec::new(),
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
        let name = "test".as_bytes().to_vec();

        assert_ok!(TfgridModule::create_farming_policy(
            RawOrigin::Root.into(),
            name,
            12,
            15,
            10,
            8,
            super::types::CertificationType::Diy
        ));
    });
}

#[test]
fn create_farming_policy_certified_works() {
    ExternalityBuilder::build().execute_with(|| {
        let name = "test".as_bytes().to_vec();

        assert_ok!(TfgridModule::create_farming_policy(
            RawOrigin::Root.into(),
            name,
            12,
            15,
            10,
            8,
            super::types::CertificationType::Certified
        ));
    });
}

#[test]
fn node_auto_attach_farming_policy() {
    ExternalityBuilder::build().execute_with(|| {
        create_entity();
        create_twin();
        create_farm();

        // Create farming policies first
        let name = "d1_test".as_bytes().to_vec();
        assert_ok!(TfgridModule::create_farming_policy(
            RawOrigin::Root.into(),
            name,
            12,
            15,
            10,
            8,
            super::types::CertificationType::Diy
        ));
        let name = "c1_test".as_bytes().to_vec();
        assert_ok!(TfgridModule::create_farming_policy(
            RawOrigin::Root.into(),
            name,
            12,
            15,
            10,
            8,
            super::types::CertificationType::Certified
        ));
        let name = "d2_test".as_bytes().to_vec();
        assert_ok!(TfgridModule::create_farming_policy(
            RawOrigin::Root.into(),
            name,
            12,
            15,
            10,
            8,
            super::types::CertificationType::Diy
        ));
        let name = "c2_test".as_bytes().to_vec();
        assert_ok!(TfgridModule::create_farming_policy(
            RawOrigin::Root.into(),
            name,
            12,
            15,
            10,
            8,
            super::types::CertificationType::Certified
        ));

        create_node();

        let node = TfgridModule::nodes(1);
        // farming policy set on the node should be 3
        // as we created the last DIY policy with id 3
        assert_eq!(node.farming_policy_id, 3);
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
        let policy = TfgridModule::pricing_policies(policy_id.clone());
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
        let policy = TfgridModule::pricing_policies(policy_id.clone());
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

    let ip = "10.2.3.3";
    assert_ok!(TfgridModule::create_twin(
        Origin::signed(alice()),
        ip.as_bytes().to_vec()
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

    let ip = "10.2.3.3";
    assert_ok!(TfgridModule::create_twin(
        Origin::signed(bob()),
        ip.as_bytes().to_vec()
    ));
}

fn create_farm() {
    let farm_name = "test_farm".as_bytes().to_vec();
    let mut pub_ips = Vec::new();
    pub_ips.push(super::types::PublicIP {
        ip: "1.1.1.0".as_bytes().to_vec(),
        gateway: "1.1.1.1".as_bytes().to_vec(),
        contract_id: 0,
    });
    assert_ok!(TfgridModule::create_farm(
        Origin::signed(alice()),
        farm_name,
        pub_ips.clone()
    ));
}

fn create_node() {
    let country = "Belgium".as_bytes().to_vec();
    let city = "Ghent".as_bytes().to_vec();

    // random location
    let location = super::types::Location {
        longitude: "12.233213231".as_bytes().to_vec(),
        latitude: "32.323112123".as_bytes().to_vec(),
    };

    let resources = super::types::Resources {
        hru: 1,
        sru: 1,
        cru: 1,
        mru: 1,
    };

    assert_ok!(TfgridModule::create_node(
        Origin::signed(alice()),
        1,
        resources,
        location,
        country,
        city,
        Vec::new(),
        true,
        true,
        "some_serial".as_bytes().to_vec()
    ));
}