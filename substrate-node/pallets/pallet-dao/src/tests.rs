use crate::{mock::*, Error, mock::Event as MockEvent};
use frame_support::{
	assert_noop, assert_ok,
	weights::GetDispatchInfo,
	pallet_prelude::*,
};
use sp_runtime::{
	traits::{BlakeTwo256, Hash},
};
use pallet_tfgrid::types as pallet_tfgrid_types;
use frame_system::{EventRecord, Phase};
use super::{Event as DaoEvent};
use sp_core::H256;

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal("some_remark".as_bytes().to_vec());
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let _proposal_weight = proposal.get_dispatch_info().weight;
		let hash = BlakeTwo256::hash_of(&proposal);

		assert_ok!(DaoModule::propose(
			Origin::signed(1),
			3,
			Box::new(proposal.clone()),
			"some_description".as_bytes().to_vec(),
			"some_link".as_bytes().to_vec(),
			proposal_len
		));

		assert_noop!(
			DaoModule::vote(
				Origin::signed(10),
				1,
				hash.clone(),
				true
			),
			Error::<Test>::FarmNotExists
		);
	});
}

#[test]
fn farmers_vote_proposal_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal("some_remark".as_bytes().to_vec());
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let _proposal_weight = proposal.get_dispatch_info().weight;
		let hash = BlakeTwo256::hash_of(&proposal);

		assert_ok!(DaoModule::propose(
			Origin::signed(1),
			3,
			Box::new(proposal.clone()),
			"some_description".as_bytes().to_vec(),
			"some_link".as_bytes().to_vec(),
			proposal_len
		));

		prepare_twin_farm_and_node(10, "f1".as_bytes().to_vec(), 1);
		assert_ok!(
			DaoModule::vote(
				Origin::signed(10),
				1,
				hash.clone(),
				true
			)
		);

		prepare_twin_farm_and_node(11, "f2".as_bytes().to_vec(), 2);
		assert_ok!(
			DaoModule::vote(
				Origin::signed(11),
				2,
				hash.clone(),
				true
			)
		);
	});
}

#[test]
fn close_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let proposal = make_proposal("some_remark".as_bytes().to_vec());
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let proposal_weight = proposal.get_dispatch_info().weight;
		let hash = BlakeTwo256::hash_of(&proposal);

		assert_ok!(DaoModule::propose(
			Origin::signed(1),
			3,
			Box::new(proposal.clone()),
			"some_description".as_bytes().to_vec(),
			"some_link".as_bytes().to_vec(),
			proposal_len
		));

		// Farmer 1 votes yes
		prepare_twin_farm_and_node(10, "f1".as_bytes().to_vec(), 1);
		assert_ok!(
			DaoModule::vote(
				Origin::signed(10),
				1,
				hash.clone(),
				true
			)
		);

		// Farmer 2 votes yes
		prepare_twin_farm_and_node(11, "f2".as_bytes().to_vec(), 2);
		assert_ok!(
			DaoModule::vote(
				Origin::signed(11),
				2,
				hash.clone(),
				true
			)
		);

		System::set_block_number(4);
		assert_noop!(
			DaoModule::close(Origin::signed(4), hash.clone(), 0, proposal_weight, proposal_len),
			Error::<Test>::TooEarly
		);

		System::set_block_number(5);
		assert_ok!(DaoModule::close(Origin::signed(4), hash.clone(), 0, proposal_weight, proposal_len));

		let e = System::events();
		assert_eq!(e[0], record(MockEvent::pallet_dao(DaoEvent::Proposed {
			account: 1,
			proposal_index: 0,
			proposal_hash: hash,
			threshold: 3
		})));

		assert_eq!(e[4], record(MockEvent::pallet_dao(DaoEvent::Voted {
			account: 10,
			proposal_hash: hash,
			voted: true,
			yes: 1,
			no: 0
		})));

		assert_eq!(e[8], record(MockEvent::pallet_dao(DaoEvent::Voted {
			account: 11,
			proposal_hash: hash,
			voted: true,
			yes: 2,
			no: 0
		})));

		assert_eq!(e[9], record(MockEvent::pallet_dao(DaoEvent::Closed {
			proposal_hash: hash,
			yes: 2,
			no: 0
		})));

		// Needed 3 yes votes in order to pass
		assert_eq!(e[10], record(MockEvent::pallet_dao(DaoEvent::Disapproved {
			proposal_hash: hash,
		})));
	});
}


#[test]
fn motion_approval_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let proposal = Call::TfgridModule(pallet_tfgrid::Call::set_farm_certification(
			1,
			pallet_tfgrid_types::CertificationType::Certified
		));
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let proposal_weight = proposal.get_dispatch_info().weight;
		let hash = BlakeTwo256::hash_of(&proposal);

		assert_ok!(DaoModule::propose(
			Origin::signed(1),
			2,
			Box::new(proposal.clone()),
			"some_description".as_bytes().to_vec(),
			"some_link".as_bytes().to_vec(),
			proposal_len
		));

		// Farmer 1 votes yes
		prepare_twin_farm_and_node(10, "f1".as_bytes().to_vec(), 1);
		assert_ok!(
			DaoModule::vote(
				Origin::signed(10),
				1,
				hash.clone(),
				true
			)
		);

		// Farmer 2 votes yes
		prepare_twin_farm_and_node(11, "f2".as_bytes().to_vec(), 2);
		assert_ok!(
			DaoModule::vote(
				Origin::signed(11),
				2,
				hash.clone(),
				true
			)
		);

		// Check farm certification type before we close
		let f1 = TfgridModule::farms(1);
		assert_eq!(f1.certification_type, pallet_tfgrid_types::CertificationType::Diy);

		// System::set_block_number(5);
		assert_ok!(DaoModule::close(Origin::signed(4), hash.clone(), 0, proposal_weight, proposal_len));

		let e = System::events();
		assert_eq!(e[0], record(MockEvent::pallet_dao(DaoEvent::Proposed {
			account: 1,
			proposal_index: 0,
			proposal_hash: hash,
			threshold: 2
		})));

		assert_eq!(e[4], record(MockEvent::pallet_dao(DaoEvent::Voted {
			account: 10,
			proposal_hash: hash,
			voted: true,
			yes: 1,
			no: 0
		})));

		assert_eq!(e[8], record(MockEvent::pallet_dao(DaoEvent::Voted {
			account: 11,
			proposal_hash: hash,
			voted: true,
			yes: 2,
			no: 0
		})));

		assert_eq!(e[9], record(MockEvent::pallet_dao(DaoEvent::Closed {
			proposal_hash: hash,
			yes: 2,
			no: 0
		})));

		assert_eq!(e[10], record(MockEvent::pallet_dao(DaoEvent::Approved {
			proposal_hash: hash,
		})));

		assert_eq!(e[11], record(MockEvent::pallet_dao(DaoEvent::Executed {
			proposal_hash: hash,
			result: Ok(())
		})));

		// Certification type of farm should be set to certified.
		let f1 = TfgridModule::farms(1);
		assert_eq!(f1.certification_type, pallet_tfgrid_types::CertificationType::Certified);
	});
}

fn record(event: Event) -> EventRecord<Event, H256> {
	EventRecord { phase: Phase::Initialization, event, topics: vec![] }
}

fn make_proposal(value: Vec<u8>) -> Call {
	Call::System(frame_system::Call::remark(value))
}

pub fn prepare_twin_farm_and_node(account_id: u64, farm_name: Vec<u8>, farm_id: u32) {
    prepare_twin(account_id);
    prepare_farm(account_id, farm_name);
	prepare_node(account_id, farm_id);
}

pub fn prepare_twin(account_id: u64) {
    let document = "some_link".as_bytes().to_vec();
    let hash = "some_hash".as_bytes().to_vec();

    assert_ok!(TfgridModule::user_accept_tc(
        Origin::signed(account_id),
        document.clone(),
        hash.clone(),
    ));
    let ip = "10.2.3.3";
    TfgridModule::create_twin(Origin::signed(account_id), ip.as_bytes().to_vec()).unwrap();
}

const GIGABYTE: u64 = 1024 * 1024 * 1024;
fn prepare_node(account_id: u64, farm_id: u32) {
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
        Origin::signed(account_id),
        farm_id,
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

pub fn prepare_farm(account_id: u64, farm_name: Vec<u8>) {
    let mut pub_ips = Vec::new();
    pub_ips.push(pallet_tfgrid_types::PublicIP {
        ip: "1.1.1.0".as_bytes().to_vec(),
        gateway: "1.1.1.1".as_bytes().to_vec(),
        contract_id: 0,
    });

    TfgridModule::create_farm(
        Origin::signed(account_id),
        farm_name,
        pub_ips.clone(),
    )
    .unwrap();
}