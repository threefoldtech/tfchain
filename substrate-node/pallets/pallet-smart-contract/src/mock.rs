#![cfg(test)]

use super::*;
use crate::{self as pallet_smart_contract};
use frame_support::{construct_runtime, parameter_types, traits::ConstU32};
use frame_system::EnsureRoot;
use sp_core::{crypto::Ss58Codec, sr25519, Pair, Public, H256};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_runtime::MultiSignature;
use sp_runtime::{
    testing::{Header, TestXt},
    traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentityLookup},
    AccountId32,
};
use sp_std::prelude::*;
use tfchain_support::{traits::ChangeNode, types::Node};
use sp_std::convert::TryFrom;

pub type Signature = MultiSignature;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Moment = u64;

type Extrinsic = TestXt<Call, ()>;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

construct_runtime!(
    pub enum TestRuntime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        TfgridModule: pallet_tfgrid::{Pallet, Call, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        SmartContractModule: pallet_smart_contract::{Pallet, Call, Storage, Event<T>},
        TFTPriceModule: pallet_tft_price::{Pallet, Call, Storage, Event<T>}
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(1024);
    pub const ExistentialDeposit: u64 = 1;
    pub StakingPoolAccount: AccountId = get_staking_pool_account();
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

pub struct NodeChanged;
impl ChangeNode for NodeChanged {
    fn node_changed(_old_node: Option<&Node>, _new_node: &Node) {}

    fn node_deleted(node: &Node) {
        SmartContractModule::node_deleted(node);
    }
}

impl pallet_tfgrid::Config for TestRuntime {
    type Event = Event;
    type RestrictedOrigin = EnsureRoot<Self::AccountId>;
    type WeightInfo = pallet_tfgrid::weights::SubstrateWeight<TestRuntime>;
    type NodeChanged = NodeChanged;
}

impl pallet_tft_price::Config for TestRuntime {
    type Event = Event;
    type AuthorityId = pallet_tft_price::crypto::AuthId;
    type Call = Call;
    type RestrictedOrigin = EnsureRoot<Self::AccountId>;
}

impl pallet_timestamp::Config for TestRuntime {
    type Moment = Moment;
    type OnTimestampSet = ();
    type MinimumPeriod = ();
    type WeightInfo = pallet_timestamp::weights::SubstrateWeight<TestRuntime>;
}

parameter_types! {
    pub const BillingFrequency: u64 = 10;
    pub const GracePeriod: u64 = 100;
    pub const DistributionFrequency: u16 = 24;
}

use weights;
impl pallet_smart_contract::Config for TestRuntime {
    type Event = Event;
    type Currency = Balances;
    type StakingPoolAccount = StakingPoolAccount;
    type BillingFrequency = BillingFrequency;
    type DistributionFrequency = DistributionFrequency;
    type GracePeriod = GracePeriod;
    type WeightInfo = weights::SubstrateWeight<TestRuntime>;
    type NodeChanged = NodeChanged;
}

type AccountPublic = <MultiSignature as Verify>::Signer;

/// Helper function to generate a crypto pair from seed
fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

impl frame_system::offchain::SigningTypes for TestRuntime {
    type Public = <Signature as Verify>::Signer;
    type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for TestRuntime
where
    Call: From<C>,
{
    type OverarchingCall = Call;
    type Extrinsic = Extrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for TestRuntime
where
    Call: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: Call,
        _public: <Signature as Verify>::Signer,
        _account: AccountId,
        nonce: u64,
    ) -> Option<(Call, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
        Some((call, (nonce, ())))
    }
}

/// Helper function to generate an account ID from seed
fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn alice() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("Alice")
}

pub fn bob() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("Bob")
}

pub fn ferdie() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("Ferdie")
}

pub fn eve() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("Eve")
}

pub fn charlie() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("Charlie")
}

pub fn get_staking_pool_account() -> AccountId {
    AccountId32::from_ss58check("5CNposRewardAccount11111111111111111111111111FSU").unwrap()
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<TestRuntime>()
        .unwrap();
    let genesis = pallet_balances::GenesisConfig::<TestRuntime> {
        balances: vec![
            (alice(), 1000000000000),
            (bob(), 2500000000),
            (charlie(), 150000),
        ],
    };
    genesis.assimilate_storage(&mut t).unwrap();

    let genesis = pallet_tft_price::GenesisConfig::<TestRuntime> {
        allowed_origin: Some(bob()),
    };
    genesis.assimilate_storage(&mut t).unwrap();

    t.into()
}
