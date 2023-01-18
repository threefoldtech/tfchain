use super::Event as MintingEvent;
use crate::{mock::RuntimeEvent as MockEvent, mock::*, types, Error};
use frame_support::{assert_noop, assert_ok, bounded_vec};
use frame_system::{EventRecord, Phase, RawOrigin};
// use log::info;
use pallet_tfgrid::{types::LocationInput, PublicIpListInput, ResourcesInput};
use sp_core::H256;
use std::convert::TryInto;
use tfchain_support::types::{FarmCertification, NodeCertification, IP4};

#[test]
fn pushing_uptime_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        Timestamp::set_timestamp(1628082000000);

        create_farming_policies();

        prepare_twin_farm_and_node(10, "farm1".as_bytes().to_vec(), 1);

        assert_ok!(MintingModule::report_uptime(
            RuntimeOrigin::signed(10),
            1000,
        ));

        Timestamp::set_timestamp(162808300000);

        assert_ok!(MintingModule::report_uptime(
            RuntimeOrigin::signed(10),
            2000,
        ));

        let our_events = System::events();
        assert_eq!(
            our_events.contains(&record(MockEvent::MintingModule(
                MintingEvent::<Test>::UptimeReportReceived {
                    node_id: 1,
                    uptime: 1000
                }
            ))),
            true
        );
    });
}

#[test]
fn pushing_0_uptime_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        Timestamp::set_timestamp(1628082000000);

        create_farming_policies();

        prepare_twin_farm_and_big_node(10, "farm1".as_bytes().to_vec(), 1);

        assert_noop!(
            MintingModule::report_uptime(RuntimeOrigin::signed(10), 0,),
            Error::<Test>::UptimeReportInvalid
        );
    });
}

#[test]
fn pushing_uptime_timedrift_boundrary_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        Timestamp::set_timestamp(1628082000000);

        create_farming_policies();

        prepare_twin_farm_and_node(10, "farm1".as_bytes().to_vec(), 1);

        assert_ok!(MintingModule::report_uptime(
            RuntimeOrigin::signed(10),
            1000,
        ));

        Timestamp::set_timestamp(1628082010000);

        assert_noop!(
            MintingModule::report_uptime(RuntimeOrigin::signed(10), 2000),
            Error::<Test>::UptimeReportInvalid
        );
    });
}

#[test]
fn period_rotation_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        Timestamp::set_timestamp(1628082000000);

        create_farming_policies();

        prepare_twin_farm_and_node(10, "farm1".as_bytes().to_vec(), 1);

        // period lenght is 10 blocks
        for i in 1..12 {
            Timestamp::set_timestamp((1628082000 * 1000) + (6000 * i));

            assert_ok!(MintingModule::report_uptime(
                RuntimeOrigin::signed(10),
                6 * i,
            ));
        }

        let our_events = System::events();
        assert_eq!(
            our_events.contains(&record(MockEvent::MintingModule(
                MintingEvent::<Test>::NodePeriodEnded { node_id: 1 }
            ))),
            true
        );

        let payable_periods = MintingModule::payable_periods(1);
        assert_eq!(payable_periods.len(), 1);
    })
}

#[test]
fn period_rotation_results_in_payable_period_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        Timestamp::set_timestamp(1628082000000);

        create_farming_policies();

        prepare_twin_farm_and_node(10, "farm1".as_bytes().to_vec(), 1);

        // period lenght is 10 blocks
        for i in 1..12 {
            Timestamp::set_timestamp((1628082000 * 1000) + (6000 * i));

            assert_ok!(MintingModule::report_uptime(
                RuntimeOrigin::signed(10),
                6 * i,
            ));
        }

        let our_events = System::events();
        assert_eq!(
            our_events.contains(&record(MockEvent::MintingModule(
                MintingEvent::<Test>::NodePeriodEnded { node_id: 1 }
            ))),
            true
        );

        let payable_periods = MintingModule::payable_periods(1);
        assert_eq!(payable_periods.len(), 1);

        let node_resources = ResourcesInput {
            hru: 1024 * GIGABYTE,
            sru: 512 * GIGABYTE,
            cru: 8,
            mru: 16 * GIGABYTE,
        };
        assert_eq!(
            payable_periods[0],
            types::NodePeriodInformation {
                uptime: 60,
                farming_policy: 1,
                ipu: 0,
                nru: 0,
                max_capacity: ResourcesInput::default(),
                min_capacity: node_resources,
                running_capacity: types::ResourceSeconds {
                    cru: (node_resources.cru * 60) as u128,
                    hru: (node_resources.hru * 60) as u128,
                    mru: (node_resources.mru * 60) as u128,
                    sru: (node_resources.sru * 60) as u128
                },
                used_capacity: types::ResourceSeconds::default()
            }
        );
    })
}

fn record(event: RuntimeEvent) -> EventRecord<RuntimeEvent, H256> {
    EventRecord {
        phase: Phase::Initialization,
        event,
        topics: vec![],
    }
}

pub fn prepare_twin_farm_and_node(account_id: u64, farm_name: Vec<u8>, farm_id: u32) {
    prepare_twin(account_id);
    prepare_farm(account_id, farm_name);
    prepare_node(account_id, farm_id);
}

pub fn prepare_twin_farm_and_big_node(account_id: u64, farm_name: Vec<u8>, farm_id: u32) {
    prepare_twin(account_id);
    prepare_farm(account_id, farm_name);
    prepare_big_node(account_id, farm_id);
}

pub fn prepare_twin(account_id: u64) {
    assert_ok!(TfgridModule::user_accept_tc(
        RuntimeOrigin::signed(account_id),
        get_document_link_input(b"some_link"),
        get_document_hash_input(b"some_hash"),
    ));

    let ip = get_twin_ip_input(b"::1");
    assert_ok!(TfgridModule::create_twin(
        RuntimeOrigin::signed(account_id),
        ip
    ));
}

const GIGABYTE: u64 = 1024 * 1024 * 1024;
fn prepare_node(account_id: u64, farm_id: u32) {
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

    assert_ok!(TfgridModule::create_node(
        RuntimeOrigin::signed(account_id),
        farm_id,
        resources,
        location,
        bounded_vec![],
        false,
        false,
        None,
    ));
}

fn prepare_big_node(account_id: u64, farm_id: u32) {
    let resources = ResourcesInput {
        hru: 20024 * GIGABYTE,
        sru: 2024 * GIGABYTE,
        cru: 16,
        mru: 64 * GIGABYTE,
    };

    // random location
    let location = LocationInput {
        city: get_city_name_input(b"Ghent"),
        country: get_country_name_input(b"Belgium"),
        latitude: get_latitude_input(b"12.233213231"),
        longitude: get_longitude_input(b"32.323112123"),
    };

    assert_ok!(TfgridModule::create_node(
        RuntimeOrigin::signed(account_id),
        farm_id,
        resources,
        location,
        bounded_vec![],
        false,
        false,
        None,
    ));
}

pub fn prepare_farm(account_id: u64, farm_name: Vec<u8>) {
    let mut pub_ips: PublicIpListInput<Test> = bounded_vec![];

    let ip = get_public_ip_ip_input(b"185.206.122.33/24");
    let gw = get_public_ip_gw_input(b"185.206.122.1");

    pub_ips.try_push(IP4 { ip, gw }).unwrap();

    assert_ok!(TfgridModule::create_farm(
        RuntimeOrigin::signed(account_id),
        farm_name.try_into().unwrap(),
        pub_ips,
    ));
}

fn create_farming_policies() {
    let name = "farm_1".as_bytes().to_vec();
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

    let name = "farm2".as_bytes().to_vec();
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

    let name = "farm3".as_bytes().to_vec();
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

    let name = "farm1".as_bytes().to_vec();
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
