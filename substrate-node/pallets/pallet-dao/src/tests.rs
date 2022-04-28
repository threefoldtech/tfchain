use crate::{mock::*, Error};
use frame_support::{
	assert_noop, assert_ok,
	weights::GetDispatchInfo,
	pallet_prelude::*,
};
use sp_runtime::{
	traits::{BlakeTwo256, Hash},
};
use pallet_tfgrid::types as pallet_tfgrid_types;

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