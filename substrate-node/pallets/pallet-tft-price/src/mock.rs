#![cfg(test)]

use super::*;
use crate::{self as pallet_tft_price};
use codec::alloc::sync::Arc;
use frame_support::traits::GenesisBuild;
use frame_support::{construct_runtime, parameter_types, traits::ConstU32};
use frame_system::mocking;
use frame_system::EnsureRoot;
use sp_core::{
    offchain::{testing, OffchainDbExt, TransactionPoolExt},
    sr25519, H256,
};
use sp_io::TestExternalities;
use sp_keystore::{testing::KeyStore, KeystoreExt, SyncCryptoStore};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = mocking::MockBlock<TestRuntime>;
use sp_runtime::MultiSignature;
pub type Signature = MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

// For testing the module, we construct a mock runtime.
construct_runtime!(
    pub enum TestRuntime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        TFTPriceModule: pallet_tft_price::{Pallet, Call, Storage, Config<T>, Event<T>},
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

impl Config for TestRuntime {
    type AuthorityId = pallet_tft_price::AuthId;
    type Call = RuntimeCall;
    type RuntimeEvent = RuntimeEvent;
    type RestrictedOrigin = EnsureRoot<Self::AccountId>;
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

pub fn allowed_account() -> AccountId {
    get_account_id_from_seed_string::<sr25519::Public>(
        "expire stage crawl shell boss any story swamp skull yellow bamboo copy",
    )
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
        let keystore = KeyStore::new();
        keystore
            .sr25519_generate_new(KEY_TYPE, Some(&format!("{}/hunter1", PHRASE)))
            .unwrap();

        let mut storage = frame_system::GenesisConfig::default()
            .build_storage::<TestRuntime>()
            .unwrap();

        let genesis = pallet_tft_price::GenesisConfig::<TestRuntime> {
            allowed_origin: Some(allowed_account()),
            min_tft_price: 10,
            max_tft_price: 1000,
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
