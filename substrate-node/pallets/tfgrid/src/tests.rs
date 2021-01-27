use crate::{Error, Module, Trait};
use frame_support::{assert_noop, assert_ok, impl_outer_origin, parameter_types};
use frame_system as system;
use sp_io::TestExternalities;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

use sp_core::{H256, Pair, Public, ed25519, sr25519};

use sp_std::prelude::*;

use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_runtime::{
	MultiSignature,
};

pub type Signature = MultiSignature;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

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
	type AccountId = AccountId;
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

impl Trait for TestRuntime {
	type Event = ();
}


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

type AccountPublic = <MultiSignature as Verify>::Signer;


// industry dismiss casual gym gap music pave gasp sick owner dumb cost

/// Helper function to generate a crypto pair from seed
fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

fn get_from_seed_string<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

fn get_account_id_from_seed_string<TPublic: Public>(seed: &str) -> AccountId where
AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed_string::<TPublic>(seed)).into_account()
}

fn alice() -> AccountId {
	get_account_id_from_seed::<sr25519::Public>("Alice")
}

// fn alice_ed25519() -> AccountId {
// 	get_account_id_from_seed::<ed25519::Public>("Alice")
// }

fn test_ed25519() -> AccountId {
	get_account_id_from_seed_string::<ed25519::Public>("industry dismiss casual gym gap music pave gasp sick owner dumb cost")
}

fn bob() -> AccountId {
	get_account_id_from_seed::<sr25519::Public>("Bob")
}

#[test]
fn test_create_entity_works() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(alice()), name.as_bytes().to_vec(), 0,0));
	});
}

#[test]
fn test_update_entity_works() {
	ExternalityBuilder::build().execute_with(|| {
		let mut name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(alice()), name.as_bytes().to_vec(), 0,0));

		// Change name to barfoo
		name = "barfoo";

		assert_ok!(TemplateModule::update_entity(Origin::signed(alice()), name.as_bytes().to_vec(), 0,0));
	});
}

#[test]
fn test_update_entity_fails_if_signed_by_someone_else() {
	ExternalityBuilder::build().execute_with(|| {
		let mut name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(alice()), name.as_bytes().to_vec(), 0,0));

		// Change name to barfoo
		name = "barfoo";

		assert_noop!(
			TemplateModule::update_entity(Origin::signed(bob()), name.as_bytes().to_vec(), 0,0),
			Error::<TestRuntime>::EntityNotExists
		);
	});
}

#[test]
fn test_create_entity_double_fails() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(alice()), name.as_bytes().to_vec(), 0,0));

		assert_noop!(
			TemplateModule::create_entity(Origin::signed(alice()), name.as_bytes().to_vec(), 0,0),
			Error::<TestRuntime>::EntityWithNameExists
		);
	});
}

#[test]
fn test_create_entity_double_fails_with_same_pubkey() {
	ExternalityBuilder::build().execute_with(|| {
		let mut name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(alice()), name.as_bytes().to_vec(), 0,0));

		name = "barfoo";

		assert_noop!(
			TemplateModule::create_entity(Origin::signed(alice()), name.as_bytes().to_vec(), 0,0),
			Error::<TestRuntime>::EntityWithPubkeyExists
		);
	});
}

#[test]
fn test_delete_entity_works() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";
		assert_ok!(TemplateModule::create_entity(Origin::signed(alice()), name.as_bytes().to_vec(), 0,0));

		assert_ok!(TemplateModule::delete_entity(Origin::signed(alice())));
	});
}

#[test]
fn test_delete_entity_fails_if_signed_by_someone_else() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";
		assert_ok!(TemplateModule::create_entity(Origin::signed(alice()), name.as_bytes().to_vec(), 0,0));

		assert_noop!(
			TemplateModule::delete_entity(Origin::signed(bob())),
			Error::<TestRuntime>::EntityNotExists
		);
	});
}

#[test]
fn test_create_twin_works() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";
		assert_ok!(TemplateModule::create_entity(Origin::signed(alice()), name.as_bytes().to_vec(), 0,0));

		let peer_id = "some_peer_id";
		assert_ok!(TemplateModule::create_twin(Origin::signed(alice()), peer_id.as_bytes().to_vec()));
	});
}

#[test]
fn test_update_twin_works() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";
		assert_ok!(TemplateModule::create_entity(Origin::signed(alice()), name.as_bytes().to_vec(), 0,0));

		let mut peer_id = "some_peer_id";
		assert_ok!(TemplateModule::create_twin(Origin::signed(alice()), peer_id.as_bytes().to_vec()));

		let twin_id = 0;
		peer_id = "some_other_peer_id";
		assert_ok!(TemplateModule::update_twin(Origin::signed(alice()), twin_id, peer_id.as_bytes().to_vec()));
	});
}

#[test]
fn test_update_twin_fails_if_signed_by_someone_else() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";
		assert_ok!(TemplateModule::create_entity(Origin::signed(alice()), name.as_bytes().to_vec(), 0,0));

		let mut peer_id = "some_peer_id";
		assert_ok!(TemplateModule::create_twin(Origin::signed(alice()), peer_id.as_bytes().to_vec()));

		let twin_id = 0;
		peer_id = "some_other_peer_id";
		assert_noop!(
			TemplateModule::update_twin(Origin::signed(bob()), twin_id, peer_id.as_bytes().to_vec()),
			Error::<TestRuntime>::UnauthorizedToUpdateTwin
		);
	});
}


#[test]
fn test_delete_twin_works() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(alice()), name.as_bytes().to_vec(), 0,0));


		let peer_id = "some_peer_id";
		assert_ok!(TemplateModule::create_twin(Origin::signed(alice()), peer_id.as_bytes().to_vec()));

		let twin_id = 0;
		assert_ok!(TemplateModule::delete_twin(Origin::signed(alice()), twin_id));
	});
}

#[test]
fn test_add_entity_to_twin() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		// Someone first creates an entity
		assert_ok!(TemplateModule::create_entity(Origin::signed(test_ed25519()), name.as_bytes().to_vec(), 0,0));

		let peer_id = "some_peer_id";
		// Bob creates an anonymous twin
		assert_ok!(TemplateModule::create_twin(Origin::signed(bob()), peer_id.as_bytes().to_vec()));

		// Signature of the entityid (0) and twinid (0) signed with test_ed25519 account
		let signature = "0cbebadf1ca1a60e6d9df4ffd9bd971ae91f1336a496154e25774b0037e1cdfe4ee518ccdce9d9006fedba8d76921dccbfe1692f7f4480e034d27749a814e206";
		
		let twin_id = 0;
		let entity_id = 0;
		
		// Bob adds someone as entity to his twin
		assert_ok!(TemplateModule::add_twin_entity(Origin::signed(bob()), twin_id, entity_id, signature.as_bytes().to_vec()));
	});
}

#[test]
fn test_add_entity_to_twin_fails_with_invalid_signature() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(test_ed25519()), name.as_bytes().to_vec(), 0,0));

		let peer_id = "some_peer_id";
		assert_ok!(TemplateModule::create_twin(Origin::signed(bob()), peer_id.as_bytes().to_vec()));

		// Add Alice as entity to bob's twin

		// Signature of the entityid (0) and twinid (0) signed with test_ed25519 account
		let signature = "12fa1dfb735dc528a8d38bc0003c90521ea313ff82e4d0b2c683283e0fbc05001af5fd106ccf938356b9679790c6e7c4c4235c3ce2d88c787a1768ddcb401d08";
		
		let twin_id = 0;
		let entity_id = 0;
		
		assert_noop!(
			TemplateModule::add_twin_entity(Origin::signed(bob()), twin_id, entity_id, signature.as_bytes().to_vec()),
			Error::<TestRuntime>::EntitySignatureDoesNotMatch
		);
	});
}

#[test]
fn test_add_entity_to_twin_fails_if_entity_is_added_twice() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(test_ed25519()), name.as_bytes().to_vec(), 0,0));

		let peer_id = "some_peer_id";
		assert_ok!(TemplateModule::create_twin(Origin::signed(bob()), peer_id.as_bytes().to_vec()));

		// Add Alice as entity to bob's twin

		// Signature of the entityid (0) and twinid (0) signed with test_ed25519 account
		let signature = "0cbebadf1ca1a60e6d9df4ffd9bd971ae91f1336a496154e25774b0037e1cdfe4ee518ccdce9d9006fedba8d76921dccbfe1692f7f4480e034d27749a814e206";
		
		let twin_id = 0;
		let entity_id = 0;
		
		assert_ok!(TemplateModule::add_twin_entity(Origin::signed(bob()), twin_id, entity_id, signature.as_bytes().to_vec()));

		
		assert_noop!(
			TemplateModule::add_twin_entity(Origin::signed(bob()), twin_id, entity_id, signature.as_bytes().to_vec()),
			Error::<TestRuntime>::EntityWithSignatureAlreadyExists
		);
	});
}

#[test]
fn test_create_twin_double_works() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(alice()), name.as_bytes().to_vec(), 0,0));

		let peer_id = "some_peer_id";
		// First time creating twin succeeds
		assert_ok!(TemplateModule::create_twin(Origin::signed(alice()), peer_id.as_bytes().to_vec()));

		let peer_id = "some_peer_id";
		// Second time creating twin succeeds
		assert_ok!(TemplateModule::create_twin(Origin::signed(alice()), peer_id.as_bytes().to_vec()));
	});
}

#[test]
fn test_create_farm_works() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(alice()), name.as_bytes().to_vec(), 0,0));

		let peer_id = "some_peer_id";
		assert_ok!(TemplateModule::create_twin(Origin::signed(alice()), peer_id.as_bytes().to_vec()));

		let twin_id = 0;

		let name = "test_farm";

		let farm = super::types::Farm{
			id: 0,
			name: name.as_bytes().to_vec(),
			twin_id,
			country_id: 0,
			city_id: 0,
			certification_type: super::types::CertificationType::None,
			entity_id: 0,
			pricing_policy_id: 0,
		};

		assert_ok!(TemplateModule::create_farm(Origin::signed(alice()), farm));
	});
}

#[test]
fn test_create_farm_with_invalid_entity_id_fails() {
	ExternalityBuilder::build().execute_with(|| {		
		let farm_name = "test_farm";
		
		let twin_id = 0;
		let entity_id = 654;

		let farm = super::types::Farm{
			id: 0,
			name: farm_name.as_bytes().to_vec(),
			twin_id,
			entity_id,
			country_id: 0,
			city_id: 0,
			certification_type: super::types::CertificationType::None,
			pricing_policy_id: 0,
		};

		// Create farm with invalid entity-id
		assert_noop!(
			TemplateModule::create_farm(Origin::signed(alice()), farm),
			Error::<TestRuntime>::EntityNotExists
		);
	});
}

#[test]
fn test_create_farm_with_invalid_twin_id_fails() {
	ExternalityBuilder::build().execute_with(|| {		
		let farm_name = "test_farm";

		let name = "foobar";
		assert_ok!(TemplateModule::create_entity(Origin::signed(alice()), name.as_bytes().to_vec(), 0,0));
		
		let entity_id = 0;
		let twin_id = 5342433;

		let farm = super::types::Farm{
			id: 0,
			name: farm_name.as_bytes().to_vec(),
			twin_id,
			entity_id,
			country_id: 0,
			city_id: 0,
			certification_type: super::types::CertificationType::None,
			pricing_policy_id: 0,
		};

		// Create farm with invalid twin-id
		assert_noop!(
			TemplateModule::create_farm(Origin::signed(alice()), farm),
			Error::<TestRuntime>::TwinNotExists
		);
	});
}

#[test]
fn test_create_farm_with_same_name_fails() {
	ExternalityBuilder::build().execute_with(|| {		
		let name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(alice()), name.as_bytes().to_vec(), 0,0));

		let peer_id = "some_peer_id";
		assert_ok!(TemplateModule::create_twin(Origin::signed(alice()), peer_id.as_bytes().to_vec()));

		let twin_id = 0;

		let farm_name = "test_farm";

		let farm = super::types::Farm{
			id: 0,
			name: farm_name.as_bytes().to_vec(),
			twin_id,
			entity_id: 0,
			country_id: 0,
			city_id: 0,
			certification_type: super::types::CertificationType::None,
			pricing_policy_id: 0,
		};

		assert_ok!(TemplateModule::create_farm(Origin::signed(alice()), farm.clone()));

		assert_noop!(
			TemplateModule::create_farm(Origin::signed(alice()), farm),
			Error::<TestRuntime>::FarmExists
		);
	});
}

#[test]
fn create_node_works() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		assert_ok!(TemplateModule::create_entity(Origin::signed(alice()), name.as_bytes().to_vec(), 0,0));

		let peer_id = "some_peer_id";
		assert_ok!(TemplateModule::create_twin(Origin::signed(alice()), peer_id.as_bytes().to_vec()));

		let twin_id = 0;

		let farm_name = "test_farm";

		let farm = super::types::Farm{
			id: 0,
			name: farm_name.as_bytes().to_vec(),
			twin_id,
			entity_id: 0,
			country_id: 0,
			city_id: 0,
			certification_type: super::types::CertificationType::None,
			pricing_policy_id: 0,
		};

		assert_ok!(TemplateModule::create_farm(Origin::signed(alice()), farm));

		// random location
		let location = super::types::Location{
			longitude: "12.233213231".as_bytes().to_vec(),
			latitude: "32.323112123".as_bytes().to_vec()
		};

		let resources = super::types::Resources {
			hru: 1,
			sru: 1,
			cru: 1,
			mru: 1,
		};

		let node = super::types::Node {
			id: 0,
			farm_id: 0,
			twin_id,
			resources,
			location,
			city_id: 0,
			country_id: 0
		};

		assert_ok!(TemplateModule::create_node(Origin::signed(alice()), node));
	});
}