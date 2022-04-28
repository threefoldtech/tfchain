use crate as pallet_dao;
use frame_support::{construct_runtime, parameter_types};
use frame_system as system;
use frame_system::{EnsureRoot};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use pallet_tfgrid;
use pallet_timestamp;
use pallet_collective;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		DaoModule: pallet_dao::{Module, Call, Storage, Event<T>},
		TfgridModule: pallet_tfgrid::{Module, Call, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
		Council: pallet_collective::<Instance1>::{Module, Call, Origin<T>, Event<T>, Config<T>},
		Membership: pallet_membership::<Instance1>::{Module, Call, Storage, Event<T>},
	}
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(1024);
    pub const ExistentialDeposit: u64 = 1;
}

impl system::Config for Test {
    type BaseCallFilter = ();
	type BlockWeights = BlockWeights;
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

pub type BlockNumber = u32;
parameter_types! {
	pub const DaoMotionDuration: BlockNumber = 4;
	pub const DaoMaxProposals: u32 = 100;
}

impl pallet_dao::Config for Test {
	type Event = Event;
	type CouncilOrigin = EnsureRoot<Self::AccountId>;
	type Proposal = Call;
	type MotionDuration = DaoMotionDuration;
	type MaxProposals = DaoMaxProposals;
}

impl pallet_tfgrid::Config for Test {
    type Event = Event;
    type RestrictedOrigin = EnsureRoot<Self::AccountId>;
    type WeightInfo = pallet_tfgrid::weights::SubstrateWeight<Test>;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ();
    type WeightInfo = pallet_timestamp::weights::SubstrateWeight<Test>;
}

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 4;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
}

pub type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Test {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();
}

impl pallet_membership::Config<pallet_membership::Instance1> for Test {
	type Event = Event;
	type AddOrigin = EnsureRoot<Self::AccountId>;
	type RemoveOrigin = EnsureRoot<Self::AccountId>;
	type SwapOrigin = EnsureRoot<Self::AccountId>;
	type ResetOrigin = EnsureRoot<Self::AccountId>;
	type PrimeOrigin = EnsureRoot<Self::AccountId>;
	type MembershipInitialized = Council;
	type MembershipChanged = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    let genesis = pallet_collective::GenesisConfig::<Test, CouncilCollective>::default();
	genesis.assimilate_storage(&mut t).unwrap();

	let genesis = pallet_membership::GenesisConfig::<Test, pallet_membership::Instance1> {
		members: vec![1, 2, 3],
		phantom: Default::default(),
	};
    genesis.assimilate_storage(&mut t).unwrap();

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
		foundation_account: 101,
		sales_account: 100,
		farming_policy_diy_cu: 160000000,
		farming_policy_diy_su: 100000000,
		farming_policy_diy_nu: 2000000,
		farming_policy_diy_ipu: 800000,
		farming_policy_certified_cu: 200000000,
		farming_policy_certified_su: 120000000,
		farming_policy_certified_nu: 3000000,
		farming_policy_certified_ipu: 1000000,
		discount_for_dedication_nodes: 50,
	};
    genesis.assimilate_storage(&mut t).unwrap();

    t.into()
}
