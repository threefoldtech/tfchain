use crate::{
    self as tfgridModule,
    farm::FarmName,
    interface::{InterfaceIp, InterfaceMac, InterfaceName},
    node::{CityName, CountryName, Location, SerialNumber},
    terms_cond::TermsAndConditions,
    weights, CityNameInput, Config, CountryNameInput, DocumentHashInput, DocumentLinkInput,
    DomainInput, FarmNameInput, Gw4Input, Gw6Input, InterfaceIpInput, InterfaceMacInput,
    InterfaceNameInput, Ip4Input, Ip6Input, LatitudeInput, LongitudeInput, PkInput, RelayInput,
};
use env_logger;
use frame_support::{construct_runtime, parameter_types, traits::ConstU32, BoundedVec};
use frame_system::EnsureRoot;
use sp_core::{ed25519, sr25519, Pair, Public, H256};
use sp_io::TestExternalities;
use sp_runtime::{
    traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
    BuildStorage, MultiSignature,
};
use sp_std::prelude::*;

use hex;
use tfchain_support::{
    traits::{ChangeNode, NodeActiveContracts, PublicIpModifier},
    types::PublicIP,
};

pub type Signature = MultiSignature;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Moment = u64;

type Block = frame_system::mocking::MockBlock<TestRuntime>;

construct_runtime!(
    pub enum TestRuntime
    {
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
        TfgridModule: tfgridModule::{Pallet, Call, Storage, Config<T>, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        Council: pallet_collective::<Instance1>::{Pallet, Call, Origin<T>, Event<T>, Config<T>},
        Membership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const ExistentialDeposit: u64 = 1;
}

impl frame_system::Config for TestRuntime {
    type BaseCallFilter = frame_support::traits::Everything;
    type Block = Block;
    type BlockWeights = ();
    type BlockLength = ();
    type AccountId = AccountId;
    type RuntimeCall = RuntimeCall;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type AccountData = pallet_balances::AccountData<u64>;
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

pub(crate) type Serial = crate::SerialNumberOf<TestRuntime>;
pub(crate) type Loc = crate::LocationOf<TestRuntime>;
pub(crate) type Interface = crate::InterfaceOf<TestRuntime>;

pub(crate) type TfgridNode = crate::TfgridNode<TestRuntime>;

pub struct NodeChanged;
impl ChangeNode<Loc, Interface, Serial> for NodeChanged {
    fn node_changed(_old_node: Option<&TfgridNode>, _new_node: &TfgridNode) {}
    fn node_deleted(_node: &TfgridNode) {}
}

pub struct PublicIpModifierType;
impl PublicIpModifier for PublicIpModifierType {
    fn ip_removed(_ip: &PublicIP) {}
}

pub struct NodeActiveContractsType;
impl NodeActiveContracts for NodeActiveContractsType {
    fn node_has_no_active_contracts(_node_id: u32) -> bool {
        true
    }
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

impl Config for TestRuntime {
    type RuntimeEvent = RuntimeEvent;
    type RestrictedOrigin = EnsureRoot<Self::AccountId>;
    type WeightInfo = weights::SubstrateWeight<TestRuntime>;
    type NodeChanged = NodeChanged;
    type PublicIpModifier = PublicIpModifierType;
    type NodeActiveContracts = NodeActiveContractsType;
    type TermsAndConditions = TestTermsAndConditions;
    type FarmName = TestFarmName;
    type MaxFarmNameLength = MaxFarmNameLength;
    type MaxFarmPublicIps = MaxFarmPublicIps;
    type InterfaceName = TestInterfaceName;
    type InterfaceMac = TestInterfaceMac;
    type InterfaceIP = TestInterfaceIp;
    type MaxInterfacesLength = MaxInterfacesLength;
    type MaxInterfaceIpsLength = MaxInterfaceIpsLength;
    type CountryName = TestCountryName;
    type CityName = TestCityName;
    type Location = TestLocation;
    type SerialNumber = TestSerialNumber;
    type TimestampHintDrift = TimestampHintDrift;
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
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<TestRuntime>;
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type RuntimeHoldReason = ();
    type MaxHolds = ();
}

impl pallet_timestamp::Config for TestRuntime {
    type Moment = Moment;
    type OnTimestampSet = ();
    type MinimumPeriod = ();
    type WeightInfo = pallet_timestamp::weights::SubstrateWeight<TestRuntime>;
}

parameter_types! {
    pub const CouncilMotionDuration: u32 = 4;
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

pub struct ExternalityBuilder;

impl ExternalityBuilder {
    pub fn build() -> TestExternalities {
        let _ = env_logger::try_init();

        let storage = frame_system::GenesisConfig::<TestRuntime>::default()
            .build_storage()
            .unwrap();
        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}
type AccountPublic = <MultiSignature as Verify>::Signer;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let _ = env_logger::try_init();

    let mut t = frame_system::GenesisConfig::<TestRuntime>::default()
        .build_storage()
        .unwrap();

    let genesis = pallet_balances::GenesisConfig::<TestRuntime> {
        balances: vec![(alice(), 1000000000000), (bob(), 190000)],
    };
    genesis.assimilate_storage(&mut t).unwrap();

    let genesis = pallet_membership::GenesisConfig::<TestRuntime, pallet_membership::Instance1> {
        members: vec![alice()].try_into().unwrap(),
        phantom: Default::default(),
    };
    genesis.assimilate_storage(&mut t).unwrap();

    t.into()
}

pub(crate) fn get_relay_input(relay_input: &[u8]) -> RelayInput {
    Some(BoundedVec::try_from(relay_input.to_vec()).expect("Invalid relay input."))
}

pub(crate) fn get_public_key_input(pk_input: &[u8]) -> PkInput {
    Some(BoundedVec::try_from(pk_input.to_vec()).expect("Invalid document hash input."))
}

pub(crate) fn get_document_link_input(document_link_input: &[u8]) -> DocumentLinkInput {
    BoundedVec::try_from(document_link_input.to_vec()).expect("Invalid document link input.")
}

pub(crate) fn get_document_hash_input(document_hash_input: &[u8]) -> DocumentHashInput {
    BoundedVec::try_from(document_hash_input.to_vec()).expect("Invalid document hash input.")
}

pub(crate) fn get_farm_name_input(farm_name_input: &[u8]) -> FarmNameInput<TestRuntime> {
    BoundedVec::try_from(farm_name_input.to_vec()).expect("Invalid farm name input.")
}

pub(crate) fn get_public_ip_ip_input(ip_input: &[u8]) -> Ip4Input {
    BoundedVec::try_from(ip_input.to_vec()).expect("Invalid public ip (ip) input.")
}

pub(crate) fn get_public_ip_gw_input(gw_input: &[u8]) -> Gw4Input {
    BoundedVec::try_from(gw_input.to_vec()).expect("Invalid public ip (gw) input.")
}

pub(crate) fn get_pub_config_ip4_input(ip4_input: &[u8]) -> Ip4Input {
    BoundedVec::try_from(ip4_input.to_vec()).expect("Invalid ip4 input.")
}

pub(crate) fn get_pub_config_gw4_input(gw4_input: &[u8]) -> Gw4Input {
    BoundedVec::try_from(gw4_input.to_vec()).expect("Invalid gw4 input.")
}

pub(crate) fn get_pub_config_ip6_input(ip6_input: &[u8]) -> Ip6Input {
    BoundedVec::try_from(ip6_input.to_vec()).expect("Invalid ip6 input.")
}

pub(crate) fn get_pub_config_gw6_input(gw6_input: &[u8]) -> Gw6Input {
    BoundedVec::try_from(gw6_input.to_vec()).expect("Invalid gw6 input.")
}

pub(crate) fn get_pub_config_domain_input(domain_input: &[u8]) -> DomainInput {
    BoundedVec::try_from(domain_input.to_vec()).expect("Invalid domain input.")
}

pub(crate) fn get_interface_name_input(if_name_input: &[u8]) -> InterfaceNameInput {
    BoundedVec::try_from(if_name_input.to_vec()).expect("Invalid interface name input")
}

pub(crate) fn get_interface_mac_input(if_mac_input: &[u8]) -> InterfaceMacInput {
    BoundedVec::try_from(if_mac_input.to_vec()).expect("Invalid interface mac input")
}

pub(crate) fn get_interface_ip_input(if_ip_input: &[u8]) -> InterfaceIpInput {
    BoundedVec::try_from(if_ip_input.to_vec()).expect("Invalid interface ip input")
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
