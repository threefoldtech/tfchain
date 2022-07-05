use crate::{self as tfgridModule, Config};
use frame_support::{construct_runtime, parameter_types, traits::ConstU32};
use frame_system::EnsureRoot;
use sp_io::TestExternalities;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use tfchain_support::types::Node;

use sp_core::{ed25519, sr25519, Pair, Public, H256};

use sp_std::prelude::*;

use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_runtime::MultiSignature;

use hex;

pub type Signature = MultiSignature;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Moment = u64;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

construct_runtime!(
    pub enum TestRuntime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        TfgridModule: tfgridModule::{Pallet, Call, Storage, Config<T>, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(1024);
    pub const ExistentialDeposit: u64 = 1;
}

impl frame_system::Config for TestRuntime {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type Origin = Origin;
    type Index = u64;
    type Call = Call;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

pub struct NodeChanged;
impl tfchain_support::traits::ChangeNode for NodeChanged {
    fn node_changed(_old_node: Option<&Node>, _new_node: &Node) {}

    fn node_deleted(_node: &tfchain_support::types::Node) {}
}

use crate::weights;
impl Config for TestRuntime {
    type Event = Event;
    type RestrictedOrigin = EnsureRoot<Self::AccountId>;
    type WeightInfo = weights::SubstrateWeight<TestRuntime>;
    type NodeChanged = NodeChanged;
}

parameter_types! {
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for TestRuntime {
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    /// The type for recording an account's balance.
    type Balance = u64;
    /// The ubiquitous event type.
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<TestRuntime>;
}

impl pallet_timestamp::Config for TestRuntime {
    type Moment = Moment;
    type OnTimestampSet = ();
    type MinimumPeriod = ();
    type WeightInfo = pallet_timestamp::weights::SubstrateWeight<TestRuntime>;
}

pub struct ExternalityBuilder;

impl ExternalityBuilder {
    pub fn build() -> TestExternalities {
        let storage = frame_system::GenesisConfig::default()
            .build_storage::<TestRuntime>()
            .unwrap();
        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}
type AccountPublic = <MultiSignature as Verify>::Signer;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<TestRuntime>()
        .unwrap();
    let genesis = pallet_balances::GenesisConfig::<TestRuntime> {
        balances: vec![(alice(), 1000000000000), (bob(), 190000)],
    };
    genesis.assimilate_storage(&mut t).unwrap();
    t.into()
}

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
fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

fn get_account_id_from_seed_string<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed_string::<TPublic>(seed)).into_account()
}

pub fn alice() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("Alice")
}

pub fn test_ed25519() -> AccountId {
    get_account_id_from_seed_string::<ed25519::Public>(
        "industry dismiss casual gym gap music pave gasp sick owner dumb cost",
    )
}

pub fn test_sr25519() -> AccountId {
    get_account_id_from_seed_string::<sr25519::Public>(
        "industry dismiss casual gym gap music pave gasp sick owner dumb cost",
    )
}

pub fn bob() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("Bob")
}

pub fn sign_create_entity(name: Vec<u8>, country: Vec<u8>, city: Vec<u8>) -> Vec<u8> {
    let seed =
        hex::decode("59336423ee7af732b2d4a76e440651e33e5ba51540e5633535b9030492c2a6f6").unwrap();
    let pair = ed25519::Pair::from_seed_slice(&seed).unwrap();

    let mut message = vec![];
    message.extend_from_slice(&name);
    message.extend_from_slice(&country);
    message.extend_from_slice(&city);

    let signature = pair.sign(&message);

    // hex encode signature
    hex::encode(signature.0.to_vec()).into()
}

pub fn sign_add_entity_to_twin(entity_id: u32, twin_id: u32) -> Vec<u8> {
    let seed =
        hex::decode("59336423ee7af732b2d4a76e440651e33e5ba51540e5633535b9030492c2a6f6").unwrap();
    let pair = ed25519::Pair::from_seed_slice(&seed).unwrap();

    let mut message = vec![];
    message.extend_from_slice(&entity_id.to_be_bytes());
    message.extend_from_slice(&twin_id.to_be_bytes());

    let signature = pair.sign(&message);

    // hex encode signature
    hex::encode(signature.0.to_vec()).into()
}

pub fn sign_create_entity_sr(name: Vec<u8>, country: Vec<u8>, city: Vec<u8>) -> Vec<u8> {
    let seed =
        hex::decode("59336423ee7af732b2d4a76e440651e33e5ba51540e5633535b9030492c2a6f6").unwrap();
    let pair = sr25519::Pair::from_seed_slice(&seed).unwrap();

    let mut message = vec![];
    message.extend_from_slice(&name);
    message.extend_from_slice(&country);
    message.extend_from_slice(&city);

    let signature = pair.sign(&message);

    // hex encode signature
    hex::encode(signature.0.to_vec()).into()
}
