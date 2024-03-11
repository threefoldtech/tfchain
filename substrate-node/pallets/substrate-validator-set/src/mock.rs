//! Mock helpers for Validator Set pallet.

#![cfg(test)]

use crate as validator_set;
use frame_support::{parameter_types, traits::ConstU32};
use frame_system::EnsureRoot;
use pallet_session::*;
use parity_scale_codec::{Decode, Encode};
use sp_core::{crypto::key_types::DUMMY, H256};
use sp_runtime::{
    impl_opaque_keys,
    testing::UintAuthorityId,
    traits::{BlakeTwo256, IdentityLookup, OpaqueKeys},
    BuildStorage, KeyTypeId, RuntimeAppPublic,
};
use sp_std::convert::{TryFrom, TryInto};
use std::cell::RefCell;

impl_opaque_keys! {
    pub struct MockSessionKeys {
        pub dummy: UintAuthorityId,
    }
}

impl From<UintAuthorityId> for MockSessionKeys {
    fn from(dummy: UintAuthorityId) -> Self {
        Self { dummy }
    }
}

pub const KEY_ID_A: KeyTypeId = KeyTypeId([4; 4]);
pub const KEY_ID_B: KeyTypeId = KeyTypeId([9; 4]);

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub struct PreUpgradeMockSessionKeys {
    pub a: [u8; 32],
    pub b: [u8; 64],
}

impl OpaqueKeys for PreUpgradeMockSessionKeys {
    type KeyTypeIdProviders = ();

    fn key_ids() -> &'static [KeyTypeId] {
        &[KEY_ID_A, KEY_ID_B]
    }

    fn get_raw(&self, i: KeyTypeId) -> &[u8] {
        match i {
            i if i == KEY_ID_A => &self.a[..],
            i if i == KEY_ID_B => &self.b[..],
            _ => &[],
        }
    }
}

type Block = frame_system::mocking::MockBlock<TestRuntime>;

frame_support::construct_runtime!(
    pub enum TestRuntime
    {
        System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
        ValidatorSet: validator_set::{Pallet, Call, Storage, Event<T>, Config<T>},
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>},
    }
);

thread_local! {
    pub static VALIDATORS: RefCell<Vec<u64>> = RefCell::new(vec![1, 2, 3]);
    pub static NEXT_VALIDATORS: RefCell<Vec<u64>> = RefCell::new(vec![1, 2, 3]);
    pub static AUTHORITIES: RefCell<Vec<UintAuthorityId>> =
        RefCell::new(vec![UintAuthorityId(1), UintAuthorityId(2), UintAuthorityId(3)]);
    pub static FORCE_SESSION_END: RefCell<bool> = RefCell::new(false);
    pub static SESSION_LENGTH: RefCell<u64> = RefCell::new(2);
    pub static SESSION_CHANGED: RefCell<bool> = RefCell::new(false);
    pub static DISABLED: RefCell<bool> = RefCell::new(false);
    pub static BEFORE_SESSION_END_CALLED: RefCell<bool> = RefCell::new(false);
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

pub fn authorities() -> Vec<UintAuthorityId> {
    AUTHORITIES.with(|l| l.borrow().to_vec())
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<TestRuntime>::default()
        .build_storage()
        .unwrap();
    let keys: Vec<_> = NEXT_VALIDATORS.with(|l| {
        l.borrow()
            .iter()
            .cloned()
            .map(|i| (i, i, UintAuthorityId(i).into()))
            .collect()
    });
    sp_state_machine::BasicExternalities::execute_with_storage(&mut t, || {
        for (ref k, ..) in &keys {
            frame_system::Pallet::<TestRuntime>::inc_providers(k);
        }
        frame_system::Pallet::<TestRuntime>::inc_providers(&4);
        frame_system::Pallet::<TestRuntime>::inc_providers(&69);
    });
    validator_set::GenesisConfig::<TestRuntime> {
        initial_validators: keys.iter().map(|x| x.0).collect::<Vec<_>>(),
    }
    .assimilate_storage(&mut t)
    .unwrap();
    pallet_session::GenesisConfig::<TestRuntime> { keys: keys.clone() }
        .assimilate_storage(&mut t)
        .unwrap();
    sp_io::TestExternalities::new(t)
}

parameter_types! {
    pub const MinimumPeriod: u64 = 5;
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for TestRuntime {
    type BaseCallFilter = frame_support::traits::Everything;
    type Block = Block;
    type BlockWeights = ();
    type BlockLength = ();
    type AccountId = u64;
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
    type AccountData = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub const MinAuthorities: u32 = 2;
}

use validator_set::weights;
impl validator_set::Config for TestRuntime {
    type AddRemoveOrigin = EnsureRoot<Self::AccountId>;
    type RuntimeEvent = RuntimeEvent;
    type MinAuthorities = MinAuthorities;
    type WeightInfo = weights::SubstrateWeight<TestRuntime>;
}

impl pallet_session::Config for TestRuntime {
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = validator_set::ValidatorOf<Self>;
    type ShouldEndSession = TestShouldEndSession;
    type NextSessionRotation = ();
    type SessionManager = ValidatorSet;
    type SessionHandler = TestSessionHandler;
    type Keys = MockSessionKeys;
    type WeightInfo = ();
    type RuntimeEvent = RuntimeEvent;
}
