#![cfg(test)]

use super::*;
use crate::{self as pallet_tft_price};
use frame_support::traits::GenesisBuild;
use frame_support::{construct_runtime, parameter_types, traits::ConstU32};
use frame_system::mocking;
use frame_system::EnsureRoot;
use parity_scale_codec::alloc::sync::Arc;
use sp_core::{
    crypto::key_types::DUMMY,
    offchain::{testing, OffchainDbExt, TransactionPoolExt},
    sr25519, H256,
};
use sp_io::TestExternalities;
use sp_keystore::{testing::MemoryKeystore, Keystore, KeystoreExt};
use sp_runtime::{
    impl_opaque_keys,
    testing::{Header, UintAuthorityId},
    traits::{
        BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, OpaqueKeys, Verify,
    },
    MultiSignature,
};
use sp_std::marker::PhantomData;
use std::cell::RefCell;
use tfchain_support::constants::time::MINUTES;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = mocking::MockBlock<TestRuntime>;
pub type Signature = MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

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

#[derive(Debug, Clone, codec::Encode, codec::Decode, PartialEq, Eq)]
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

// For testing the module, we construct a mock runtime.
construct_runtime!(
    pub enum TestRuntime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        TFTPriceModule: pallet_tft_price::{Pallet, Call, Storage, Config<T>, Event<T>},
        Authorship: pallet_authorship::{Pallet, Storage},
        ValidatorSet: substrate_validator_set::{Pallet, Call, Storage, Event<T>, Config<T>},
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>},
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
    type AccountId = AccountId;
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
    pub const UnsignedPriority: u64 = 100;
}

use weights;
impl Config for TestRuntime {
    type AuthorityId = pallet_tft_price::AuthId;
    type Call = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type RestrictedOrigin = EnsureRoot<Self::AccountId>;
    type WeightInfo = weights::SubstrateWeight<TestRuntime>;
}

parameter_types! {
    pub const UncleGenerations: u32 = 0;
}

impl pallet_authorship::Config for TestRuntime {
    type FindAuthor = ();
    type EventHandler = ();
}

parameter_types! {
    pub const Period: u32 = 60 * MINUTES;
    pub const Offset: u32 = 0;
}

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

use pallet_session::SessionHandler;
use sp_runtime::RuntimeAppPublic;
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

parameter_types! {
    pub const MinAuthorities: u32 = 2;
}

impl substrate_validator_set::Config for TestRuntime {
    type AddRemoveOrigin = EnsureRoot<Self::AccountId>;
    type RuntimeEvent = RuntimeEvent;
    type MinAuthorities = MinAuthorities;
    type WeightInfo = substrate_validator_set::weights::SubstrateWeight<TestRuntime>;
}

impl pallet_session::Config for TestRuntime {
    type RuntimeEvent = RuntimeEvent;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = substrate_validator_set::ValidatorOf<Self>;
    type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
    type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
    type SessionManager = ();
    type SessionHandler = TestSessionHandler;
    type Keys = MockSessionKeys;
    type WeightInfo = ();
}

impl frame_system::offchain::SigningTypes for TestRuntime {
    type Public = <Signature as Verify>::Signer;
    type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for TestRuntime
where
    RuntimeCall: From<C>,
{
    type OverarchingCall = RuntimeCall;
    type Extrinsic = UncheckedExtrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for TestRuntime
where
    RuntimeCall: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: RuntimeCall,
        _public: <Signature as Verify>::Signer,
        account: AccountId,
        _nonce: u64,
    ) -> Option<(
        RuntimeCall,
        <UncheckedExtrinsic as ExtrinsicT>::SignaturePayload,
    )> {
        Some((call, (account, (), ())))
    }
}

use sp_core::{Pair, Public};
type AccountPublic = <MultiSignature as Verify>::Signer;

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

pub fn bob() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("Bob")
}

pub struct ExternalityBuilder;

impl ExternalityBuilder {
    pub fn build() -> TestExternalities {
        const PHRASE: &str =
            "expire stage crawl shell boss any story swamp skull yellow bamboo copy";

        let (offchain, _) = testing::TestOffchainExt::new();
        let (pool, _) = testing::TestTransactionPoolExt::new();
        let keystore = MemoryKeystore::new();
        keystore
            .sr25519_generate_new(KEY_TYPE, Some(&format!("{}/hunter1", PHRASE)))
            .unwrap();

        let mut storage = frame_system::GenesisConfig::default()
            .build_storage::<TestRuntime>()
            .unwrap();

        let session_genesis = pallet_session::GenesisConfig::<TestRuntime> {
            keys: vec![(alice(), alice(), MockSessionKeys::from(UintAuthorityId(1)))],
        };
        session_genesis.assimilate_storage(&mut storage).unwrap();

        let genesis = pallet_tft_price::GenesisConfig::<TestRuntime> {
            min_tft_price: 10,
            max_tft_price: 1000,
            _data: PhantomData,
        };
        genesis.assimilate_storage(&mut storage).unwrap();

        let mut t = TestExternalities::from(storage);
        t.register_extension(OffchainDbExt::new(offchain));
        t.register_extension(TransactionPoolExt::new(pool));
        t.register_extension(KeystoreExt(Arc::new(keystore)));
        t.execute_with(|| System::set_block_number(1));
        t
    }
}
