#![cfg(test)]

use super::*;
use crate as pallet_smart_contract;
use frame_support::{construct_runtime, parameter_types};
use sp_core::{sr25519, Pair, Public, H256, crypto::Ss58Codec};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_runtime::MultiSignature;
use sp_runtime::{
    AccountId32,
    testing::{Header, TestXt},
    traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentityLookup},
};
use sp_std::prelude::*;
use frame_system::EnsureRoot;

pub type Signature = MultiSignature;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;
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
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        TfgridModule: pallet_tfgrid::{Module, Call, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
        SmartContractModule: pallet_smart_contract::{Module, Call, Event<T>},
        TFTPriceModule: pallet_tft_price::{Module, Call, Storage, Event<T>}
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
    type BaseCallFilter = ();
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
}

impl pallet_balances::Config for TestRuntime {
    type MaxLocks = ();
    type Balance = u64;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<TestRuntime>;
}

impl pallet_tfgrid::Config for TestRuntime {
    type Event = Event;
    type Currency = Balances;
    type RestrictedOrigin = EnsureRoot<Self::AccountId>;
}

impl pallet_tft_price::Config for TestRuntime {
    type Event = Event;
    type AuthorityId = pallet_tft_price::crypto::AuthId;
    type Call = Call;
}

impl pallet_timestamp::Config for TestRuntime {
    type Moment = Moment;
    type OnTimestampSet = ();
    type MinimumPeriod = ();
    type WeightInfo = pallet_timestamp::weights::SubstrateWeight<TestRuntime>;
}

parameter_types! {
    pub const BillingFrequency: u64 = 10;
}

impl Config for TestRuntime {
    type Event = Event;
    type Currency = Balances;
    type StakingPoolAccount = StakingPoolAccount;
    type BillingFrequency = BillingFrequency;
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
        balances: vec![(alice(), 1000000000000), (bob(), 2500000000), (charlie(), 100000)],
    };
    genesis.assimilate_storage(&mut t).unwrap();
    t.into()
}
