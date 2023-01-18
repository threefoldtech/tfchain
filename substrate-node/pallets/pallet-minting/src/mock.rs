use crate::mock::sp_api_hidden_includes_construct_runtime::hidden_include::traits::GenesisBuild;
use crate::{self as pallet_minting};
use env_logger;
use frame_support::{
    construct_runtime, dispatch::DispatchResultWithPostInfo, parameter_types, traits::ConstU32,
    BoundedVec,
};
use frame_system::EnsureRoot;
use pallet_tfgrid::node::{CityName, CountryName};
use pallet_tfgrid::{
    farm::FarmName,
    interface::{InterfaceIp, InterfaceMac, InterfaceName},
    node::{Location, SerialNumber},
    terms_cond::TermsAndConditions,
    twin::TwinIp,
    DocumentHashInput, DocumentLinkInput, TwinIpInput,
};
use pallet_tfgrid::{
    CityNameInput, CountryNameInput, Gw4Input, Ip4Input, LatitudeInput, LongitudeInput,
};
use pallet_timestamp;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use sp_std::convert::{TryFrom, TryInto};
use tfchain_support::traits::{ChangeNode, MintingHook, PublicIpModifier};
use tfchain_support::types::PublicIP;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        MintingModule: pallet_minting::{Pallet, Call, Storage, Event<T>},
        TfgridModule: pallet_tfgrid::{Pallet, Call, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

type AccountId = u64;

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type RuntimeOrigin = RuntimeOrigin;
    type Index = u64;
    type RuntimeCall = RuntimeCall;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
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

pub type BlockNumber = u32;
parameter_types! {
    pub const DaoMotionDuration: BlockNumber = 4;
    pub const MinVetos: u32 = 2;
}

pub(crate) type Serial = pallet_tfgrid::pallet::SerialNumberOf<Test>;
pub(crate) type Loc = pallet_tfgrid::pallet::LocationOf<Test>;
pub(crate) type Interface = pallet_tfgrid::pallet::InterfaceOf<Test>;

pub(crate) type TfgridNode = pallet_tfgrid::pallet::TfgridNode<Test>;

pub struct NodeChanged;
impl ChangeNode<Loc, Interface, Serial> for NodeChanged {
    fn node_changed(_old_node: Option<&TfgridNode>, _new_node: &TfgridNode) {}
    fn node_deleted(_node: &TfgridNode) {}
}

pub struct PublicIpModifierType;
impl PublicIpModifier for PublicIpModifierType {
    fn ip_removed(_ip: &PublicIP) {}
}

pub struct MintingHookType;
impl MintingHook<AccountId> for MintingHookType {
    fn report_nru(node_id: u32, nru: u64, window: u64) {
        MintingModule::report_nru(node_id, nru, window)
    }
    fn report_uptime(source: &AccountId, uptime: u64) -> DispatchResultWithPostInfo {
        MintingModule::process_uptime_report(source, uptime)
    }
    fn report_used_resources(
        node_id: u32,
        resources: tfchain_support::resources::Resources,
        window: u64,
        ipu: u32,
    ) {
        MintingModule::report_used_resources(node_id, resources, window, ipu)
    }
}

parameter_types! {
    pub const AllowedUptimeDrift: u64 = 60;
    pub const PeriodTreshold: u64 = 10000000;
    pub const HeartbeatInterval: u64 = 7200;
}

impl pallet_minting::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type AllowedUptimeDrift = AllowedUptimeDrift;
    type PeriodTreshold = PeriodTreshold;
    type HeartbeatInterval = HeartbeatInterval;
}

parameter_types! {
    pub const MaxFarmNameLength: u32 = 40;
    pub const MaxInterfaceIpsLength: u32 = 5;
    pub const MaxInterfacesLength: u32 = 10;
    pub const MaxFarmPublicIps: u32 = 512;
}

pub(crate) type TestTermsAndConditions = TermsAndConditions<Test>;

pub(crate) type TestTwinIp = TwinIp<Test>;
pub(crate) type TestFarmName = FarmName<Test>;

pub(crate) type TestInterfaceName = InterfaceName<Test>;
pub(crate) type TestInterfaceMac = InterfaceMac<Test>;
pub(crate) type TestInterfaceIp = InterfaceIp<Test>;

pub(crate) type TestCountryName = CountryName<Test>;
pub(crate) type TestCityName = CityName<Test>;
pub(crate) type TestLocation = Location<Test>;
pub(crate) type TestSerialNumber = SerialNumber<Test>;

impl pallet_tfgrid::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type RestrictedOrigin = EnsureRoot<Self::AccountId>;
    type WeightInfo = pallet_tfgrid::weights::SubstrateWeight<Test>;
    type NodeChanged = NodeChanged;
    type PublicIpModifier = PublicIpModifierType;
    type TermsAndConditions = TestTermsAndConditions;
    type TwinIp = TestTwinIp;
    type FarmName = TestFarmName;
    type MaxFarmNameLength = MaxFarmNameLength;
    type MaxFarmPublicIps = MaxFarmPublicIps;
    type MaxInterfacesLength = MaxInterfacesLength;
    type InterfaceName = TestInterfaceName;
    type InterfaceMac = TestInterfaceMac;
    type InterfaceIP = TestInterfaceIp;
    type MaxInterfaceIpsLength = MaxInterfaceIpsLength;
    type CountryName = TestCountryName;
    type CityName = TestCityName;
    type Location = TestLocation;
    type SerialNumber = TestSerialNumber;
    type MintingHook = MintingHookType;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ();
    type WeightInfo = pallet_timestamp::weights::SubstrateWeight<Test>;
}

parameter_types! {
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    /// The type for recording an account's balance.
    type Balance = u64;
    /// The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
}

pub(crate) fn get_document_link_input(document_link_input: &[u8]) -> DocumentLinkInput {
    BoundedVec::try_from(document_link_input.to_vec()).expect("Invalid document link input.")
}

pub(crate) fn get_document_hash_input(document_hash_input: &[u8]) -> DocumentHashInput {
    BoundedVec::try_from(document_hash_input.to_vec()).expect("Invalid document hash input.")
}

pub(crate) fn get_twin_ip_input(twin_ip_input: &[u8]) -> TwinIpInput {
    BoundedVec::try_from(twin_ip_input.to_vec()).expect("Invalid twin ip input.")
}

pub(crate) fn get_public_ip_ip_input(ip_input: &[u8]) -> Ip4Input {
    BoundedVec::try_from(ip_input.to_vec()).expect("Invalid public ip (ip) input.")
}

pub(crate) fn get_public_ip_gw_input(gw_input: &[u8]) -> Gw4Input {
    BoundedVec::try_from(gw_input.to_vec()).expect("Invalid public ip (gw) input.")
}

pub(crate) fn get_city_name_input(city_input: &[u8]) -> CityNameInput {
    BoundedVec::try_from(city_input.to_vec()).expect("Invalid city name input.")
}

pub(crate) fn get_country_name_input(country_input: &[u8]) -> CountryNameInput {
    BoundedVec::try_from(country_input.to_vec()).expect("Invalid country name input.")
}

pub(crate) fn get_latitude_input(latitude_input: &[u8]) -> LatitudeInput {
    BoundedVec::try_from(latitude_input.to_vec()).expect("Invalid latitude input.")
}

pub(crate) fn get_longitude_input(longitude_input: &[u8]) -> LongitudeInput {
    BoundedVec::try_from(longitude_input.to_vec()).expect("Invalid longitude input.")
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let _ = env_logger::try_init();

    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    let genesis = pallet_tfgrid::GenesisConfig::<Test> {
        su_price_value: 300000,
        su_price_unit: 4,
        nu_price_value: 2000,
        nu_price_unit: 4,
        cu_price_value: 600000,
        cu_price_unit: 4,
        ipu_price_value: 100000,
        ipu_price_unit: 4,
        unique_name_price_value: 20000,
        domain_name_price_value: 40000,
        foundation_account: Some(101),
        sales_account: Some(100),
        farming_policy_diy_cu: 160000000,
        farming_policy_diy_su: 100000000,
        farming_policy_diy_nu: 2000000,
        farming_policy_diy_ipu: 800000,
        farming_policy_diy_minimal_uptime: 95,
        farming_policy_certified_cu: 200000000,
        farming_policy_certified_su: 120000000,
        farming_policy_certified_nu: 3000000,
        farming_policy_certified_ipu: 1000000,
        farming_policy_certified_minimal_uptime: 95,
        discount_for_dedication_nodes: 50,
        connection_price: 80,
    };
    genesis.assimilate_storage(&mut t).unwrap();

    t.into()
}
