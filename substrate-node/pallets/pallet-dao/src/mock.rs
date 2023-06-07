use crate::mock::sp_api_hidden_includes_construct_runtime::hidden_include::traits::GenesisBuild;
use crate::{self as pallet_dao};
use frame_support::{construct_runtime, parameter_types, traits::ConstU32, BoundedVec};
use frame_system::EnsureRoot;
use pallet_collective;
use pallet_tfgrid::node::{CityName, CountryName};
use pallet_tfgrid::{
    farm::FarmName,
    interface::{InterfaceIp, InterfaceMac, InterfaceName},
    node::{Location, SerialNumber},
    terms_cond::TermsAndConditions,
    CityNameInput, CountryNameInput, DocumentHashInput, DocumentLinkInput, Gw4Input, Ip4Input,
    LatitudeInput, LongitudeInput, PkInput, RelayInput,
};
use pallet_timestamp;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use sp_std::convert::{TryFrom, TryInto};
use tfchain_support::traits::{ChangeNode, PublicIpModifier};
use tfchain_support::types::PublicIP;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
    pub enum TestRuntime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        DaoModule: pallet_dao::{Pallet, Call, Storage, Event<T>},
        TfgridModule: pallet_tfgrid::{Pallet, Call, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        Council: pallet_collective::<Instance1>::{Pallet, Call, Origin<T>, Event<T>, Config<T>},
        Membership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for TestRuntime {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type RuntimeOrigin = RuntimeOrigin;
    type Index = u64;
    type RuntimeCall = RuntimeCall;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
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

pub(crate) type Serial = pallet_tfgrid::pallet::SerialNumberOf<TestRuntime>;
pub(crate) type Loc = pallet_tfgrid::pallet::LocationOf<TestRuntime>;
pub(crate) type Interface = pallet_tfgrid::pallet::InterfaceOf<TestRuntime>;

pub(crate) type TfgridNode = pallet_tfgrid::pallet::TfgridNode<TestRuntime>;

pub struct NodeChanged;
impl ChangeNode<Loc, Interface, Serial> for NodeChanged {
    fn node_changed(old_node: Option<&TfgridNode>, new_node: &TfgridNode) {
        DaoModule::node_changed(old_node, new_node)
    }
    fn node_deleted(node: &TfgridNode) {
        DaoModule::node_deleted(node);
    }
}

pub struct PublicIpModifierType;
impl PublicIpModifier for PublicIpModifierType {
    fn ip_removed(_ip: &PublicIP) {}
}

use super::weights;
impl pallet_dao::Config for TestRuntime {
    type RuntimeEvent = RuntimeEvent;
    type CouncilOrigin = EnsureRoot<Self::AccountId>;
    type Proposal = RuntimeCall;
    type MotionDuration = DaoMotionDuration;
    type MinVetos = MinVetos;
    type Tfgrid = TfgridModule;
    type NodeChanged = NodeChanged;
    type WeightInfo = weights::SubstrateWeight<TestRuntime>;
}

parameter_types! {
    pub const MaxFarmNameLength: u32 = 40;
    pub const MaxInterfaceIpsLength: u32 = 5;
    pub const MaxInterfacesLength: u32 = 10;
    pub const MaxFarmPublicIps: u32 = 512;
    pub const TimestampHintDrift: u64 = 60;
}

pub(crate) type TestTermsAndConditions = TermsAndConditions<TestRuntime>;

pub(crate) type TestFarmName = FarmName<TestRuntime>;

pub(crate) type TestInterfaceName = InterfaceName<TestRuntime>;
pub(crate) type TestInterfaceMac = InterfaceMac<TestRuntime>;
pub(crate) type TestInterfaceIp = InterfaceIp<TestRuntime>;

pub(crate) type TestCountryName = CountryName<TestRuntime>;
pub(crate) type TestCityName = CityName<TestRuntime>;
pub(crate) type TestLocation = Location<TestRuntime>;
pub(crate) type TestSerialNumber = SerialNumber<TestRuntime>;

impl pallet_tfgrid::Config for TestRuntime {
    type RuntimeEvent = RuntimeEvent;
    type RestrictedOrigin = EnsureRoot<Self::AccountId>;
    type WeightInfo = pallet_tfgrid::weights::SubstrateWeight<TestRuntime>;
    type NodeChanged = NodeChanged;
    type PublicIpModifier = PublicIpModifierType;
    type TermsAndConditions = TestTermsAndConditions;
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
    type TimestampHintDrift = TimestampHintDrift;
}

impl pallet_timestamp::Config for TestRuntime {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ();
    type WeightInfo = pallet_timestamp::weights::SubstrateWeight<TestRuntime>;
}

parameter_types! {
    pub const CouncilMotionDuration: BlockNumber = 4;
    pub const CouncilMaxProposals: u32 = 100;
    pub const CouncilMaxMembers: u32 = 100;
}

pub type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for TestRuntime {
    type RuntimeOrigin = RuntimeOrigin;
    type Proposal = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type MotionDuration = CouncilMotionDuration;
    type MaxProposals = CouncilMaxProposals;
    type MaxMembers = CouncilMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type SetMembersOrigin = EnsureRoot<Self::AccountId>;
    type WeightInfo = ();
    type MaxProposalWeight = ();
}

impl pallet_membership::Config<pallet_membership::Instance1> for TestRuntime {
    type RuntimeEvent = RuntimeEvent;
    type AddOrigin = EnsureRoot<Self::AccountId>;
    type RemoveOrigin = EnsureRoot<Self::AccountId>;
    type SwapOrigin = EnsureRoot<Self::AccountId>;
    type ResetOrigin = EnsureRoot<Self::AccountId>;
    type PrimeOrigin = EnsureRoot<Self::AccountId>;
    type MembershipInitialized = Council;
    type MembershipChanged = ();
    type MaxMembers = CouncilMaxMembers;
    type WeightInfo = pallet_membership::weights::SubstrateWeight<TestRuntime>;
}

pub(crate) fn get_document_link_input(document_link_input: &[u8]) -> DocumentLinkInput {
    BoundedVec::try_from(document_link_input.to_vec()).expect("Invalid document link input.")
}

pub(crate) fn get_document_hash_input(document_hash_input: &[u8]) -> DocumentHashInput {
    BoundedVec::try_from(document_hash_input.to_vec()).expect("Invalid document hash input.")
}

pub(crate) fn get_relay_input(relay_input: &[u8]) -> RelayInput {
    Some(BoundedVec::try_from(relay_input.to_vec()).expect("Invalid relay input."))
}

pub(crate) fn get_public_key_input(pk_input: &[u8]) -> PkInput {
    Some(BoundedVec::try_from(pk_input.to_vec()).expect("Invalid public key input."))
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
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<TestRuntime>()
        .unwrap();

    let genesis = pallet_collective::GenesisConfig::<TestRuntime, CouncilCollective>::default();
    genesis.assimilate_storage(&mut t).unwrap();

    let genesis = pallet_membership::GenesisConfig::<TestRuntime, pallet_membership::Instance1> {
        members: vec![1, 2, 3].try_into().unwrap(),
        phantom: Default::default(),
    };
    genesis.assimilate_storage(&mut t).unwrap();

    let genesis = pallet_tfgrid::GenesisConfig::<TestRuntime> {
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
