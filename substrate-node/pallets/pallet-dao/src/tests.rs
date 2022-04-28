use crate::{mock::*, Error};
use frame_support::{
	assert_noop, assert_ok,
	weights::GetDispatchInfo,
	pallet_prelude::*,
};
use sp_runtime::{
	traits::{BlakeTwo256, Hash},
};

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

fn make_proposal(value: Vec<u8>) -> Call {
	Call::System(frame_system::Call::remark(value))
}