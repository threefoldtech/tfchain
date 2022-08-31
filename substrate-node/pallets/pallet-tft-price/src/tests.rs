use crate::{self as pallet_tft_price, *};
use codec::alloc::sync::Arc;
use frame_support::error::BadOrigin;
use frame_support::traits::GenesisBuild;
use frame_support::{assert_noop, assert_ok, construct_runtime, parameter_types, traits::ConstU32};
use frame_system::{limits, mocking};
use frame_system::{EnsureRoot, RawOrigin};
use sp_core::{
    offchain::{
        testing::{self},
        OffchainDbExt, TransactionPoolExt,
    },
    sr25519::{self},
    H256,
};
use sp_io::TestExternalities;
use sp_keystore::{testing::KeyStore, KeystoreExt, SyncCryptoStore};
use sp_runtime::{
    testing::{Header, TestXt},
    traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify},
};

type Extrinsic = TestXt<Call, ()>;
type UncheckedExtrinsic = mocking::MockUncheckedExtrinsic<TestRuntime>;
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
    pub BlockWeights: limits::BlockWeights = limits::BlockWeights::simple_max(1024);
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
    type Call = Call;
    type Event = Event;
    type RestrictedOrigin = EnsureRoot<Self::AccountId>;
}

impl frame_system::offchain::SigningTypes for TestRuntime {
    type Public = <Signature as Verify>::Signer;
    type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for TestRuntime
where
    Call: From<C>,
{
    type OverarchingCall = Call;
    type Extrinsic = Extrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for TestRuntime
where
    Call: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: Call,
        _public: <Signature as Verify>::Signer,
        _account: AccountId,
        nonce: u64,
    ) -> Option<(Call, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
        Some((call, (nonce, ())))
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

#[test]
fn test_set_allowed_origin_works() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        assert_ok!(TFTPriceModule::set_allowed_origin(
            RawOrigin::Root.into(),
            bob(),
        ));

        assert_eq!(TFTPriceModule::allowed_origin(), Some(bob()));
    })
}

#[test]
fn test_set_allowed_origin_by_wrong_origin_fails() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        assert_noop!(
            TFTPriceModule::set_allowed_origin(Origin::signed(bob()), bob()),
            BadOrigin,
        );
    })
}

#[test]
fn test_set_prices_works() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        let acct = allowed_account();
        for i in 1..1441 {
            let target_block = i * 100; // we set the price every 100 blocks
            run_to_block(target_block);
            match TFTPriceModule::set_prices(Origin::signed(acct.clone()), 500, target_block) {
                Ok(_) => (),
                Err(_) => panic!("Couldn't set tft_price"),
            }
        }
        let queue = TFTPriceModule::queue_transient();
        let items = queue.get_all_values();
        assert_eq!(items.len(), 1440);
    })
}

#[test]
fn test_set_price_works() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        let acct = allowed_account();
        assert_ok!(TFTPriceModule::set_prices(Origin::signed(acct), 500, 1));
    })
}

#[test]
fn test_set_price_wrong_origin_fails() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        assert_noop!(
            TFTPriceModule::set_prices(Origin::signed(bob()), 500, 1),
            Error::<TestRuntime>::AccountUnauthorizedToSetPrice
        );
    })
}

#[test]
fn test_parse_lowest_price_from_valid_request_works() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        let request_str = json_stellar_price_request_valid();
        let price = TFTPriceModule::parse_lowest_price_from_request(request_str).unwrap();
        assert_eq!(price, 33);
    })
}

#[test]
fn test_parse_lowest_price_from_empty_request_fails() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        let request_str = json_stellar_price_request_empty();
        assert_eq!(
            TFTPriceModule::parse_lowest_price_from_request(request_str),
            None
        );
    })
}

#[test]
fn test_parse_lowest_price_from_incomplete_request_fails() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        let request_str = json_stellar_price_request_incomplete();
        assert_eq!(
            TFTPriceModule::parse_lowest_price_from_request(request_str),
            None
        );
    })
}

#[test]
fn test_set_min_tft_price_works() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        assert_ok!(TFTPriceModule::set_min_tft_price(
            RawOrigin::Root.into(),
            20
        ));

        assert_eq!(TFTPriceModule::min_tft_price(), 20);
    })
}

#[test]
fn test_set_min_tft_price_wrong_origin_fails() {
    let mut t = ExternalityBuilder::build();
    t.execute_with(|| {
        assert_noop!(
            TFTPriceModule::set_min_tft_price(Origin::signed(bob()), 20),
            BadOrigin,
        );
    })
}

fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::set_block_number(System::block_number() + 1);
    }
}

fn json_stellar_price_request_empty() -> &'static str {
    r#"
    {
    }"#
}

fn json_stellar_price_request_incomplete() -> &'static str {
    r#"
    {
        "_embedded": {
          "records": [
          ]
        }
    }"#
}

fn json_stellar_price_request_valid() -> &'static str {
    r#"
    {
        "_embedded": {
          "records": [
            {
              "source_asset_type": "credit_alphanum4",
              "source_asset_code": "USDC",
              "source_asset_issuer": "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN",
              "source_amount": "0.0328655",
              "destination_asset_type": "credit_alphanum4",
              "destination_asset_code": "TFT",
              "destination_asset_issuer": "GBOVQKJYHXRR3DX6NOX2RRYFRCUMSADGDESTDNBDS6CDVLGVESRTAC47",
              "destination_amount": "1.0000000",
              "path": [
                {
                  "asset_type": "native"
                },
                {
                  "asset_type": "credit_alphanum4",
                  "asset_code": "yXLM",
                  "asset_issuer": "GARDNV3Q7YGT4AKSDF25LT32YSCCW4EV22Y2TV3I2PU2MMXJTEDL5T55"
                }
              ]
            },
            {
              "source_asset_type": "credit_alphanum4",
              "source_asset_code": "USDC",
              "source_asset_issuer": "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN",
              "source_amount": "0.0340713",
              "destination_asset_type": "credit_alphanum4",
              "destination_asset_code": "TFT",
              "destination_asset_issuer": "GBOVQKJYHXRR3DX6NOX2RRYFRCUMSADGDESTDNBDS6CDVLGVESRTAC47",
              "destination_amount": "1.0000000",
              "path": [
                {
                  "asset_type": "native"
                }
              ]
            },
            {
              "source_asset_type": "credit_alphanum4",
              "source_asset_code": "USDC",
              "source_asset_issuer": "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN",
              "source_amount": "0.0351812",
              "destination_asset_type": "credit_alphanum4",
              "destination_asset_code": "TFT",
              "destination_asset_issuer": "GBOVQKJYHXRR3DX6NOX2RRYFRCUMSADGDESTDNBDS6CDVLGVESRTAC47",
              "destination_amount": "1.0000000",
              "path": []
            }
          ]
        }
    }"#
}
