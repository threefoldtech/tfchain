use crate::{self as tfgridModule, Config};
use frame_support::{construct_runtime, parameter_types, traits::ConstU32};
use frame_system::EnsureRoot;
use sp_io::TestExternalities;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

use sp_core::{ed25519, sr25519, Pair, Public, H256};

use sp_std::prelude::*;

use crate::farm::FarmName;
use crate::interface::{InterfaceIp, InterfaceMac, InterfaceName};
use crate::node::Location;
use crate::pub_config::{Domain, GW4, GW6, IP4, IP6};
use crate::pub_ip::{GatewayIP, PublicIP};
use crate::twin::TwinIp;
use crate::weights;
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

pub(crate) type Loc = crate::LocationOf<TestRuntime>;
pub(crate) type PubConfig = crate::PubConfigOf<TestRuntime>;
pub(crate) type Interface = crate::InterfaceOf<TestRuntime>;

pub(crate) type TfgridNode = crate::TfgridNode<TestRuntime>;

pub struct NodeChanged;
impl tfchain_support::traits::ChangeNode<Loc, PubConfig, Interface> for NodeChanged {
    fn node_changed(_old_node: Option<&TfgridNode>, _new_node: &TfgridNode) {}
    fn node_deleted(_node: &TfgridNode) {}
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

pub(crate) type TestLocation = Location<TestRuntime>;

impl Config for TestRuntime {
    type Event = Event;
    type RestrictedOrigin = EnsureRoot<Self::AccountId>;
    type WeightInfo = weights::SubstrateWeight<TestRuntime>;
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
    type InterfaceName = TestInterfaceName;
    type InterfaceMac = TestInterfaceMac;
    type InterfaceIP = TestInterfaceIp;
    type MaxInterfacesLength = MaxInterfacesLength;
    type MaxInterfaceIpsLength = MaxInterfaceIpsLength;
    type Location = TestLocation;
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

pub(crate) fn get_twin_ip(twin_ip_input: &[u8]) -> TestTwinIp {
    TwinIp::try_from(twin_ip_input.to_vec()).expect("Invalid twin ip input.")
}

pub(crate) fn get_farm_name(farm_name_input: &[u8]) -> TestFarmName {
    FarmName::try_from(farm_name_input.to_vec()).expect("Invalid farm input.")
}

pub(crate) fn get_pub_config_ip4(ip4: &[u8]) -> TestIP4 {
    IP4::try_from(ip4.to_vec()).expect("Invalid ip4 input")
}

pub(crate) fn get_pub_config_gw4(gw4: &[u8]) -> TestGW4 {
    GW4::try_from(gw4.to_vec()).expect("Invalid gw4 input")
}

pub(crate) fn get_pub_config_ip6(ip6: &[u8]) -> TestIP6 {
    IP6::try_from(ip6.to_vec()).expect("Invalid ip6 input")
}

pub(crate) fn get_pub_config_gw6(gw6: &[u8]) -> TestGW6 {
    GW6::try_from(gw6.to_vec()).expect("Invalid gw6 input")
}

pub(crate) fn get_public_ip_ip(ip: &[u8]) -> TestPublicIP {
    PublicIP::try_from(ip.to_vec()).expect("Invalid public ip input")
}

pub(crate) fn get_public_ip_gateway(gw: &[u8]) -> TestGatewayIP {
    GatewayIP::try_from(gw.to_vec()).expect("Invalid gateway ip input")
}

pub(crate) fn get_interface_name(name: &[u8]) -> TestInterfaceName {
    InterfaceName::try_from(name.to_vec()).expect("Invalid interface name input")
}

pub(crate) fn get_interface_mac(mac: &[u8]) -> TestInterfaceMac {
    InterfaceMac::try_from(mac.to_vec()).expect("Invalid interface mac input")
}

pub(crate) fn get_interface_ip(ip: &[u8]) -> TestInterfaceIp {
    InterfaceIp::try_from(ip.to_vec()).expect("Invalid interface ip input")
}

pub(crate) fn get_location(
    city: &[u8],
    country: &[u8],
    latitude: &[u8],
    longitude: &[u8],
) -> TestLocation {
    Location::try_from((
        city.to_vec(),
        country.to_vec(),
        latitude.to_vec(),
        longitude.to_vec(),
    ))
    .expect("Invalid location input")
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
