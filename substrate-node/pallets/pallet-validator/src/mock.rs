use super::*;
use crate as pallet_validator;
use core::cell::RefCell;
use frame_support::{bounded_vec, construct_runtime, parameter_types, traits::ConstU32};
use frame_system::EnsureRoot;
use pallet_session::{SessionHandler, ShouldEndSession};
use sp_core::{crypto::key_types::DUMMY, H256};
use sp_runtime::{
    impl_opaque_keys,
    testing::{Header, UintAuthorityId},
    traits::{BlakeTwo256, IdentityLookup, OpaqueKeys},
    RuntimeAppPublic,
};
use sp_std::convert::{TryFrom, TryInto};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

construct_runtime!(
    pub enum TestRuntime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>},
        ValidatorModule: pallet_validator::{Pallet, Call, Storage, Event<T>},
        Council: pallet_collective::<Instance1>::{Pallet, Call, Origin<T>, Event<T>, Config<T>},
        Membership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>},
        ValidatorSet: substrate_validator_set::{Event<T>},
    }
);

impl pallet_validator::Config for TestRuntime {
    type RuntimeEvent = RuntimeEvent;
    type CouncilOrigin = EnsureRoot<Self::AccountId>;
    type Currency = ();
}

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

parameter_types! {
    pub const MinAuthorities: u32 = 0;
}

impl substrate_validator_set::Config for TestRuntime {
    type RuntimeEvent = RuntimeEvent;
    type AddRemoveOrigin = EnsureRoot<Self::AccountId>;
    type MinAuthorities = MinAuthorities;
}

thread_local! {
    pub static AUTHORITIES: RefCell<Vec<UintAuthorityId>> =
        RefCell::new(vec![UintAuthorityId(1), UintAuthorityId(2), UintAuthorityId(3)]);
    pub static FORCE_SESSION_END: RefCell<bool> = RefCell::new(false);
    pub static SESSION_LENGTH: RefCell<u64> = RefCell::new(2);
    pub static SESSION_CHANGED: RefCell<bool> = RefCell::new(false);
    pub static DISABLED: RefCell<bool> = RefCell::new(false);
    pub static BEFORE_SESSION_END_CALLED: RefCell<bool> = RefCell::new(false);
}

pub struct TestShouldEndSession;
impl ShouldEndSession<u64> for TestShouldEndSession {
    fn should_end_session(now: u64) -> bool {
        let l = SESSION_LENGTH.with(|l| *l.borrow());
        now % l == 0
            || FORCE_SESSION_END.with(|l| {
                let r = *l.borrow();
                *l.borrow_mut() = false;
                r
            })
    }
}

pub struct TestSessionHandler;
impl SessionHandler<u64> for TestSessionHandler {
    const KEY_TYPE_IDS: &'static [sp_runtime::KeyTypeId] = &[UintAuthorityId::ID];
    fn on_genesis_session<T: OpaqueKeys>(_validators: &[(u64, T)]) {}
    fn on_new_session<T: OpaqueKeys>(
        changed: bool,
        validators: &[(u64, T)],
        _queued_validators: &[(u64, T)],
    ) {
        SESSION_CHANGED.with(|l| *l.borrow_mut() = changed);
        AUTHORITIES.with(|l| {
            *l.borrow_mut() = validators
                .iter()
                .map(|(_, id)| id.get::<UintAuthorityId>(DUMMY).unwrap_or_default())
                .collect()
        });
    }
    fn on_disabled(_validator_index: u32) {
        DISABLED.with(|l| *l.borrow_mut() = true)
    }
    fn on_before_session_ending() {
        BEFORE_SESSION_END_CALLED.with(|b| *b.borrow_mut() = true);
    }
}

impl_opaque_keys! {
    pub struct MockSessionKeys {
        pub dummy: UintAuthorityId,
    }
}

impl pallet_session::Config for TestRuntime {
    type ValidatorId = Self::AccountId;
    type ValidatorIdOf = substrate_validator_set::ValidatorOf<Self>;
    type ShouldEndSession = TestShouldEndSession;
    type NextSessionRotation = ();
    type SessionManager = ValidatorSet;
    type SessionHandler = TestSessionHandler;
    type Keys = MockSessionKeys;
    type WeightInfo = ();
    type RuntimeEvent = RuntimeEvent;
}

pub type BlockNumber = u32;
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

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<TestRuntime>()
        .unwrap();

    let genesis = pallet_collective::GenesisConfig::<TestRuntime, CouncilCollective>::default();
    genesis.assimilate_storage(&mut t).unwrap();

    let genesis = pallet_membership::GenesisConfig::<TestRuntime, pallet_membership::Instance1> {
        members: bounded_vec![1, 2, 3], // [1, 2, 3] accounts are council members
        phantom: Default::default(),
    };
    genesis.assimilate_storage(&mut t).unwrap();

    // Enable receiving events
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
