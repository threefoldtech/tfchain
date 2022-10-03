use super::*;
use crate as pallet_validator;
use core::cell::RefCell;
use frame_support::{construct_runtime, parameter_types, traits::ConstU32};
use frame_system::EnsureRoot;
use pallet_session::{SessionHandler, ShouldEndSession};
use sp_core::{crypto::key_types::DUMMY, sr25519, Pair, Public, H256};
use sp_runtime::{
    impl_opaque_keys,
    testing::{Header, UintAuthorityId},
    traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, OpaqueKeys, Verify},
    MultiSignature, RuntimeAppPublic,
};
use sp_std::convert::{TryFrom, TryInto};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;
type AccountPublic = <MultiSignature as Verify>::Signer;
type AccountId = <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;

construct_runtime!(
    pub enum TestRuntime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>},
        ValidatorModule: pallet_validator::{Pallet, Call, Storage, Event<T>},
        Membership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>},
        ValidatorSet: substrate_validator_set::{Event<T>},
    }
);

impl pallet_validator::Config for TestRuntime {
    type Event = Event;
    type CouncilOrigin = EnsureRoot<Self::AccountId>;
    type Currency = Balances;
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
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
    pub const MinAuthorities: u32 = 2;
}

impl substrate_validator_set::Config for TestRuntime {
    type Event = Event;
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
impl SessionHandler<AccountId> for TestSessionHandler {
    const KEY_TYPE_IDS: &'static [sp_runtime::KeyTypeId] = &[UintAuthorityId::ID];
    fn on_genesis_session<T: OpaqueKeys>(_validators: &[(AccountId, T)]) {}
    fn on_new_session<T: OpaqueKeys>(
        changed: bool,
        validators: &[(AccountId, T)],
        _queued_validators: &[(AccountId, T)],
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
    type Event = Event;
}

impl pallet_membership::Config<pallet_membership::Instance1> for TestRuntime {
    type Event = Event;
    type AddOrigin = EnsureRoot<Self::AccountId>;
    type RemoveOrigin = EnsureRoot<Self::AccountId>;
    type SwapOrigin = EnsureRoot<Self::AccountId>;
    type ResetOrigin = EnsureRoot<Self::AccountId>;
    type PrimeOrigin = EnsureRoot<Self::AccountId>;
    type MembershipInitialized = (); // Council;
    type MembershipChanged = ();
    type MaxMembers = (); //CouncilMaxMembers;
    type WeightInfo = pallet_membership::weights::SubstrateWeight<TestRuntime>;
}

parameter_types! {
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
    pub const ExistentialDeposit: u64 = 1;
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

/// Helper function to generate a crypto pair from seed
fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
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

pub fn alice() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("Alice")
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<TestRuntime>()
        .unwrap();

    let genesis = pallet_balances::GenesisConfig::<TestRuntime> {
        balances: vec![(alice(), 1000000000000)],
    };
    genesis.assimilate_storage(&mut t).unwrap();

    t.into()
}
