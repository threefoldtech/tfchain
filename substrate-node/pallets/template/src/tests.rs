use crate::{Error, Module, Trait};
use frame_support::{assert_noop, assert_ok, impl_outer_origin, parameter_types};
use frame_system as system;
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};
use sp_std::prelude::*;

impl_outer_origin! {
	pub enum Origin for TestRuntime {}
}
// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TestRuntime;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: u32 = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
	pub const ExistentialDeposit: u64 = 1;
}
impl system::Trait for TestRuntime {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Index = u64;
	type Call = ();
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type PalletInfo = ();
	type AccountData = balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

impl balances::Trait for TestRuntime {
	type Balance = u64;
	type MaxLocks = ();
	type Event = ();
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

impl Trait for TestRuntime {
	type Event = ();
	type Currency = Balances;
}

type Balances = balances::Module<TestRuntime>;
type System = system::Module<TestRuntime>;

struct ExternalityBuilder;

impl ExternalityBuilder {
	pub fn build() -> TestExternalities {
		let storage = system::GenesisConfig::default()
			.build_storage::<TestRuntime>()
			.unwrap();
		TestExternalities::from(storage)
	}
}

pub type TemplateModule = Module<TestRuntime>;

#[test]
fn test_create_entity_works() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(1), name.as_bytes().to_vec(), 0,0));
	});
}

#[test]
fn test_update_entity_works() {
	ExternalityBuilder::build().execute_with(|| {
		let mut name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(1), name.as_bytes().to_vec(), 0,0));

		// Change name to barfoo
		name = "barfoo";

		assert_ok!(TemplateModule::update_entity(Origin::signed(1), name.as_bytes().to_vec(), 0,0));
	});
}

#[test]
fn test_update_entity_fails_if_signed_by_someone_else() {
	ExternalityBuilder::build().execute_with(|| {
		let mut name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(1), name.as_bytes().to_vec(), 0,0));

		// Change name to barfoo
		name = "barfoo";

		assert_noop!(
			TemplateModule::update_entity(Origin::signed(2), name.as_bytes().to_vec(), 0,0),
			Error::<TestRuntime>::EntityNotExists
		);
	});
}

#[test]
fn test_create_entity_double_fails() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(1), name.as_bytes().to_vec(), 0,0));

		assert_noop!(
			TemplateModule::create_entity(Origin::signed(1), name.as_bytes().to_vec(), 0,0),
			Error::<TestRuntime>::EntityWithNameExists
		);
	});
}

#[test]
fn test_create_entity_double_fails_with_same_pubkey() {
	ExternalityBuilder::build().execute_with(|| {
		let mut name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(1), name.as_bytes().to_vec(), 0,0));

		name = "barfoo";

		assert_noop!(
			TemplateModule::create_entity(Origin::signed(1), name.as_bytes().to_vec(), 0,0),
			Error::<TestRuntime>::EntityWithPubkeyExists
		);
	});
}

#[test]
fn test_delete_entity_works() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";
		assert_ok!(TemplateModule::create_entity(Origin::signed(1), name.as_bytes().to_vec(), 0,0));

		assert_ok!(TemplateModule::delete_entity(Origin::signed(1)));
	});
}

#[test]
fn test_delete_entity_fails_if_signed_by_someone_else() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";
		assert_ok!(TemplateModule::create_entity(Origin::signed(1), name.as_bytes().to_vec(), 0,0));

		assert_noop!(
			TemplateModule::delete_entity(Origin::signed(2)),
			Error::<TestRuntime>::EntityNotExists
		);
	});
}

#[test]
fn test_create_twin_works() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(1), name.as_bytes().to_vec(), 0,0));


		assert_ok!(TemplateModule::create_twin(Origin::signed(1)));
	});
}

#[test]
fn test_delete_twin_works() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(1), name.as_bytes().to_vec(), 0,0));


		assert_ok!(TemplateModule::create_twin(Origin::signed(1)));

		assert_ok!(TemplateModule::delete_twin(Origin::signed(1)));
	});
}

#[test]
fn test_create_twin_double_fails() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(1), name.as_bytes().to_vec(), 0,0));

		// First time creating twin succeeds
		assert_ok!(TemplateModule::create_twin(Origin::signed(1)));

		// Creating it a second time with the same pubkey would fail
		assert_noop!(
			TemplateModule::create_twin(Origin::signed(1)),
			Error::<TestRuntime>::TwinExists
		);
	});
}

#[test]
fn test_create_twin_with_unknown_entityid_fails() {
	ExternalityBuilder::build().execute_with(|| {
		assert_noop!(
			TemplateModule::create_twin(Origin::signed(1)),
			Error::<TestRuntime>::EntityNotExists
		);
	});
}

#[test]
fn test_create_farm_works() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(1), name.as_bytes().to_vec(), 0,0));

		assert_ok!(TemplateModule::create_twin(Origin::signed(1)));

		let twin_id = 0;

		let farm_name = "test_farm";

		assert_ok!(TemplateModule::create_farm(
			Origin::signed(1), 
			farm_name.as_bytes().to_vec(),
			twin_id,
			0,
			0,
			super::types::CertificationType::None,
			0,
			0
		));
	});
}

#[test]
fn test_create_farm_with_invalid_entity_id_fails() {
	ExternalityBuilder::build().execute_with(|| {		
		let farm_name = "test_farm";
		
		let twin_id = 0;
		let entity_id = 654;

		// Create farm with invalid entity-id
		assert_noop!(
			TemplateModule::create_farm(
				Origin::signed(1), 
				farm_name.as_bytes().to_vec(),
				entity_id,
				twin_id,
				0,
				super::types::CertificationType::None,
				0,
				0
			),
			Error::<TestRuntime>::EntityNotExists
		);
	});
}

#[test]
fn test_create_farm_with_invalid_twin_id_fails() {
	ExternalityBuilder::build().execute_with(|| {		
		let farm_name = "test_farm";

		let name = "foobar";
		assert_ok!(TemplateModule::create_entity(Origin::signed(1), name.as_bytes().to_vec(), 0,0));
		
		let entity_id = 0;
		let twin_id = 5342433;

		// Create farm with invalid twin-id
		assert_noop!(
			TemplateModule::create_farm(
				Origin::signed(1), 
				farm_name.as_bytes().to_vec(),
				entity_id,
				twin_id,
				0,
				super::types::CertificationType::None,
				0,
				0
			),
			Error::<TestRuntime>::TwinNotExists
		);
	});
}

#[test]
fn test_create_farm_with_same_name_fails() {
	ExternalityBuilder::build().execute_with(|| {		
		let name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(1), name.as_bytes().to_vec(), 0,0));

		assert_ok!(TemplateModule::create_twin(Origin::signed(1)));

		let twin_id = 0;

		let farm_name = "test_farm";

		assert_ok!(TemplateModule::create_farm(
			Origin::signed(1), 
			farm_name.as_bytes().to_vec(),
			twin_id,
			0,
			0,
			super::types::CertificationType::None,
			0,
			0
		));

		assert_noop!(
			TemplateModule::create_farm(
				Origin::signed(1), 
				farm_name.as_bytes().to_vec(),
				0,
				twin_id,
				0,
				super::types::CertificationType::None,
				0,
				0
			),
			Error::<TestRuntime>::FarmExists
		);
	});
}

#[test]
fn create_node_works() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(1), name.as_bytes().to_vec(), 0,0));

		assert_ok!(TemplateModule::create_twin(Origin::signed(1)));

		let twin_id = 0;

		let farm_name = "test_farm";

		assert_ok!(TemplateModule::create_farm(
			Origin::signed(1), 
			farm_name.as_bytes().to_vec(),
			twin_id,
			0,
			0,
			super::types::CertificationType::None,
			0,
			0
		));

		// random location
		let location = super::types::Location{
			longitude: "12.233213231".as_bytes().to_vec(),
			latitude: "32.323112123".as_bytes().to_vec()
		};

		let resource = super::types::Resources {
			hru: 1,
			sru: 1,
			cru: 1,
			mru: 1,
		};

		let farm_id = 0;
		assert_ok!(TemplateModule::create_node(
			Origin::signed(1), 
			farm_id,
			twin_id,
			resource,
			location,
			0,
			0
		));
	});
}