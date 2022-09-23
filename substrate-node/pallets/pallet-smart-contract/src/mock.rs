#![cfg(test)]

use super::*;
use crate::name_contract::NameContractName;
use crate::{self as pallet_smart_contract};
use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU32, GenesisBuild},
};
use frame_system::EnsureRoot;
use pallet_tfgrid::{
    farm::FarmName,
    interface::{InterfaceIp, InterfaceMac, InterfaceName},
    pub_config::{Domain, GW4, GW6, IP4, IP6},
    pub_ip::{GatewayIP, PublicIP},
    twin::TwinIp,
};
use sp_core::{crypto::Ss58Codec, sr25519, Pair, Public, H256};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_runtime::MultiSignature;
use sp_runtime::{
    testing::{Header, TestXt},
    traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentityLookup},
    AccountId32,
};
use sp_std::convert::{TryFrom, TryInto};
use tfchain_support::{traits::ChangeNode, types::Node};

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

pub(crate) type PubConfig = pallet_tfgrid::pallet::PubConfigOf<TestRuntime>;
pub(crate) type Interface = pallet_tfgrid::pallet::InterfaceOf<TestRuntime>;

pub struct NodeChanged;
impl ChangeNode<PubConfig, Interface> for NodeChanged {
    fn node_changed(
        _old_node: Option<&Node<PubConfig, Interface>>,
        _new_node: &Node<PubConfig, Interface>,
    ) {
    }

    fn node_deleted(node: &Node<PubConfig, Interface>) {
        SmartContractModule::node_deleted(node);
    }
}

parameter_types! {
    pub const MaxFarmNameLength: u32 = 40;
    pub const MaxInterfaceIpsLength: u32 = 5;
    pub const MaxInterfacesLength: u32 = 10;
    pub const MaxFarmPublicIps: u32 = 512;
}

pub(crate) type TestTwinIp = TwinIp<TestRuntime>;
pub(crate) type TestFarmName = FarmName<TestRuntime>;
pub(crate) type TestPublicIP = PublicIP<TestRuntime>;
pub(crate) type TestGatewayIP = GatewayIP<TestRuntime>;

pub(crate) type TestIP4 = IP4<TestRuntime>;
pub(crate) type TestGW4 = GW4<TestRuntime>;
pub(crate) type TestIP6 = IP6<TestRuntime>;
pub(crate) type TestGW6 = GW6<TestRuntime>;
pub(crate) type TestDomain = Domain<TestRuntime>;

pub(crate) type TestInterfaceName = InterfaceName<TestRuntime>;
pub(crate) type TestInterfaceMac = InterfaceMac<TestRuntime>;
pub(crate) type TestInterfaceIp = InterfaceIp<TestRuntime>;

impl pallet_tfgrid::Config for TestRuntime {
    type Event = Event;
    type RestrictedOrigin = EnsureRoot<Self::AccountId>;
    type WeightInfo = pallet_tfgrid::weights::SubstrateWeight<TestRuntime>;
    type NodeChanged = NodeChanged;
    type TwinIp = TestTwinIp;
    type FarmName = TestFarmName;
    type MaxFarmNameLength = MaxFarmNameLength;
    type MaxFarmPublicIps = MaxFarmPublicIps;
    type PublicIP = TestPublicIP;
    type GatewayIP = TestGatewayIP;
    type IP4 = TestIP4;
    type GW4 = TestGW4;
    type IP6 = TestIP6;
    type GW6 = TestGW6;
    type Domain = TestDomain;
    type MaxInterfacesLength = MaxInterfacesLength;
    type InterfaceName = TestInterfaceName;
    type InterfaceMac = TestInterfaceMac;
    type InterfaceIP = TestInterfaceIp;
    type MaxInterfaceIpsLength = MaxInterfaceIpsLength;
}

impl pallet_tft_price::Config for TestRuntime {
    type Event = Event;
    type AuthorityId = pallet_tft_price::AuthId;
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
    pub const MaxNameContractNameLength: u32 = 64;
    pub const MaxNodeContractPublicIPs: u32 = 1;
    pub const MaxDeploymentDataLength: u32 = 512;
}

pub(crate) type TestNameContractName = NameContractName<TestRuntime>;

use weights;
impl pallet_smart_contract::Config for TestRuntime {
    type Event = Event;
    type Currency = Balances;
    type Burn = ();
    type StakingPoolAccount = StakingPoolAccount;
    type BillingFrequency = BillingFrequency;
    type DistributionFrequency = DistributionFrequency;
    type GracePeriod = GracePeriod;
    type WeightInfo = weights::SubstrateWeight<TestRuntime>;
    type NodeChanged = NodeChanged;
    type MaxNameContractNameLength = MaxNameContractNameLength;
    type NameContractName = TestNameContractName;
    type RestrictedOrigin = EnsureRoot<Self::AccountId>;
    type MaxDeploymentDataLength = MaxDeploymentDataLength;
    type MaxNodeContractPublicIps = MaxNodeContractPublicIPs;
}

type AccountPublic = <MultiSignature as Verify>::Signer;

pub(crate) fn get_name_contract_name(contract_name_input: &[u8]) -> TestNameContractName {
    NameContractName::try_from(contract_name_input.to_vec()).expect("Invalid farm input.")
}

pub(crate) fn get_twin_ip(twin_ip_input: &[u8]) -> TestTwinIp {
    TwinIp::try_from(twin_ip_input.to_vec()).expect("Invalid twin ip input.")
}

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

pub fn dave() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("Dave")
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
        min_tft_price: 10,
        max_tft_price: 1000,
    };
    genesis.assimilate_storage(&mut t).unwrap();

    t.into()
}
