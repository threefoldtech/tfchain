#![cfg(test)]
use super::*;
use crate::name_contract::NameContractName;
use crate::{self as pallet_smart_contract};
use codec::{alloc::sync::Arc, Decode};
use frame_support::{
    construct_runtime,
    dispatch::{DispatchResultWithPostInfo, PostDispatchInfo},
    parameter_types,
    traits::{ConstU32, GenesisBuild},
    BoundedVec,
};
use frame_system::EnsureRoot;
use pallet_tfgrid::{
    farm::FarmName,
    interface::{InterfaceIp, InterfaceMac, InterfaceName},
    node::{CityName, CountryName},
    node::{Location, SerialNumber},
    terms_cond::TermsAndConditions,
    twin::TwinIp,
    CityNameInput, CountryNameInput, DocumentHashInput, DocumentLinkInput, Gw4Input, Ip4Input,
    LatitudeInput, LongitudeInput, TwinIpInput,
};
use parking_lot::RwLock;
use sp_core::{
    crypto::key_types::DUMMY,
    crypto::Ss58Codec,
    offchain::{
        testing::{self},
        OffchainDbExt, OffchainWorkerExt, TransactionPoolExt,
    },
    sr25519, Pair, Public, H256,
};
use sp_keystore::{testing::KeyStore, KeystoreExt, SyncCryptoStore};
use sp_runtime::{
    impl_opaque_keys,
    offchain::TransactionPool,
    testing::{Header, TestXt, UintAuthorityId},
    traits::{
        BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, OpaqueKeys, Verify,
    },
    AccountId32, MultiSignature,
};
use sp_std::convert::{TryFrom, TryInto};
use sp_std::marker::PhantomData;
use std::{cell::RefCell, panic, thread};
use tfchain_support::{
    constants::time::{MINUTES, SECS_PER_HOUR},
    traits::{ChangeNode, MintingHook, PublicIpModifier},
};

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
// set environment variable RUST_LOG=debug to see all logs when running the tests and call
// env_logger::init() at the beginning of the test
use env_logger;

pub type Signature = MultiSignature;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Moment = u64;

pub type Extrinsic = TestXt<RuntimeCall, ()>;
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

construct_runtime!(
    pub enum TestRuntime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        TfgridModule: pallet_tfgrid::{Pallet, Call, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        SmartContractModule: pallet_smart_contract::{Pallet, Call, Storage, Event<T>},
        TFTPriceModule: pallet_tft_price::{Pallet, Call, Storage, Event<T>},
        Authorship: pallet_authorship::{Pallet, Call, Storage, Inherent},
        ValidatorSet: substrate_validator_set::{Pallet, Call, Storage, Event<T>, Config<T>},
        Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub StakingPoolAccount: AccountId = get_staking_pool_account();
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
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
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
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<TestRuntime>;
}

pub(crate) type Serial = pallet_tfgrid::pallet::SerialNumberOf<TestRuntime>;
pub(crate) type Loc = pallet_tfgrid::pallet::LocationOf<TestRuntime>;
pub(crate) type Interface = pallet_tfgrid::pallet::InterfaceOf<TestRuntime>;

pub(crate) type TfgridNode = pallet_tfgrid::pallet::TfgridNode<TestRuntime>;

pub struct NodeChanged;
impl ChangeNode<Loc, Interface, Serial> for NodeChanged {
    fn node_changed(_old_node: Option<&TfgridNode>, _new_node: &TfgridNode) {}
    fn node_deleted(node: &TfgridNode) {
        SmartContractModule::node_deleted(node);
    }
}

pub struct PublicIpModifierType;
impl PublicIpModifier for PublicIpModifierType {
    fn ip_removed(ip: &PublicIP) {
        SmartContractModule::ip_removed(ip);
    }
}

pub struct MintingHookType;
impl MintingHook<AccountId> for MintingHookType {
    fn report_nru(_node_id: u32, _nru: u64, _window: u64) {}
    fn report_uptime(_source: &AccountId, _uptime: u64) -> DispatchResultWithPostInfo {
        Ok(().into())
    }
    fn report_used_resources(
        _node_id: u32,
        _resources: tfchain_support::resources::Resources,
        _window: u64,
        _public_ips: u32,
    ) {
    }
}

parameter_types! {
    pub const MaxFarmNameLength: u32 = 40;
    pub const MaxInterfaceIpsLength: u32 = 5;
    pub const MaxInterfacesLength: u32 = 10;
    pub const MaxFarmPublicIps: u32 = 512;
}

pub(crate) type TestTermsAndConditions = TermsAndConditions<TestRuntime>;

pub(crate) type TestTwinIp = TwinIp<TestRuntime>;
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

impl pallet_tft_price::Config for TestRuntime {
    type RuntimeEvent = RuntimeEvent;
    type AuthorityId = pallet_tft_price::AuthId;
    type Call = RuntimeCall;
    type RestrictedOrigin = EnsureRoot<Self::AccountId>;
}

impl pallet_timestamp::Config for TestRuntime {
    type Moment = Moment;
    type OnTimestampSet = ();
    type MinimumPeriod = ();
    type WeightInfo = pallet_timestamp::weights::SubstrateWeight<TestRuntime>;
}

parameter_types! {
    pub const BillingFrequency: u64 = 10;
    pub const BillingReferencePeriod: u64 = SECS_PER_HOUR;
    pub const GracePeriod: u64 = 100;
    pub const DistributionFrequency: u16 = 24;
    pub const MaxNameContractNameLength: u32 = 64;
    pub const MaxNodeContractPublicIPs: u32 = 512;
    pub const MaxDeploymentDataLength: u32 = 512;
    pub const SecondsPerHour: u64 = 3600;
}

pub(crate) type TestNameContractName = NameContractName<TestRuntime>;

use weights;
impl pallet_smart_contract::Config for TestRuntime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type Burn = ();
    type StakingPoolAccount = StakingPoolAccount;
    type BillingFrequency = BillingFrequency;
    type BillingReferencePeriod = BillingReferencePeriod;
    type DistributionFrequency = DistributionFrequency;
    type GracePeriod = GracePeriod;
    type WeightInfo = weights::SubstrateWeight<TestRuntime>;
    type MaxNameContractNameLength = MaxNameContractNameLength;
    type NameContractName = TestNameContractName;
    type RestrictedOrigin = EnsureRoot<Self::AccountId>;
    type MaxDeploymentDataLength = MaxDeploymentDataLength;
    type MaxNodeContractPublicIps = MaxNodeContractPublicIPs;
    type AuthorityId = pallet_smart_contract::crypto::AuthId;
    type Call = RuntimeCall;
    type MintingHook = MintingHookType;
}

parameter_types! {
    pub const UncleGenerations: u32 = 0;
}

impl pallet_authorship::Config for TestRuntime {
    type FindAuthor = ();
    type UncleGenerations = UncleGenerations;
    type FilterUncle = ();
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

type AccountPublic = <MultiSignature as Verify>::Signer;

pub(crate) fn get_name_contract_name(contract_name_input: &[u8]) -> TestNameContractName {
    NameContractName::try_from(contract_name_input.to_vec()).expect("Invalid farm input.")
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

pub(crate) fn get_public_ip_ip_input(public_ip_ip_input: &[u8]) -> Ip4Input {
    BoundedVec::try_from(public_ip_ip_input.to_vec()).expect("Invalid public ip (ip) input.")
}

pub(crate) fn get_public_ip_gw_input(public_ip_gw_input: &[u8]) -> Gw4Input {
    BoundedVec::try_from(public_ip_gw_input.to_vec()).expect("Invalid public ip (gw) input.")
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

/// Helper function to generate a crypto pair from seed
fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

impl frame_system::offchain::SigningTypes for TestRuntime {
    type Public = <Signature as Verify>::Signer;
    type Signature = Signature;
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for TestRuntime
where
    RuntimeCall: From<LocalCall>,
{
    type OverarchingCall = RuntimeCall;
    type Extrinsic = Extrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for TestRuntime
where
    RuntimeCall: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: RuntimeCall,
        _public: <Signature as Verify>::Signer,
        _account: AccountId,
        nonce: u64,
    ) -> Option<(RuntimeCall, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
        Some((call, (nonce, ())))
    }
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

pub fn ferdie() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("Ferdie")
}

pub fn eve() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("Eve")
}

pub fn charlie() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("Charlie")
}

pub fn dave() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("Dave")
}

pub fn get_staking_pool_account() -> AccountId {
    AccountId32::from_ss58check("5CNposRewardAccount11111111111111111111111111FSU").unwrap()
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    // for showing logs in tests
    let _ = env_logger::try_init();

    let mut storage = frame_system::GenesisConfig::default()
        .build_storage::<TestRuntime>()
        .unwrap();
    let genesis = pallet_balances::GenesisConfig::<TestRuntime> {
        balances: vec![
            (alice(), 1000000000000),
            (bob(), 2500000000),
            (charlie(), 150000),
        ],
    };
    genesis.assimilate_storage(&mut storage).unwrap();

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

    let t = sp_io::TestExternalities::from(storage);

    t
}
pub type TransactionCall = pallet_smart_contract::Call<TestRuntime>;
pub type ExtrinsicResult = Result<PostDispatchInfo, DispatchErrorWithPostInfo>;

#[derive(Default)]
pub struct PoolState {
    /// A vector of calls that we expect should be executed
    pub expected_calls: Vec<(TransactionCall, ExtrinsicResult, u64)>,
    pub calls_to_execute: Vec<(TransactionCall, ExtrinsicResult, u64)>,
    pub i: usize,
}

impl PoolState {
    pub fn should_call_bill_contract(
        &mut self,
        contract_id: u64,
        expected_result: ExtrinsicResult,
        block_number: u64,
    ) {
        self.expected_calls.push((
            crate::Call::bill_contract_for_block { contract_id },
            expected_result,
            block_number,
        ));
    }

    pub fn execute_calls_and_check_results(&mut self, block_number: u64) {
        if self.calls_to_execute.len() == 0 {
            return;
        }

        // execute the calls that were submitted to the pool and compare the result
        for call_to_execute in self.calls_to_execute.iter() {
            let result = match call_to_execute.0 {
                // matches bill_contract_for_block
                crate::Call::bill_contract_for_block { contract_id } => {
                    SmartContractModule::bill_contract_for_block(
                        RuntimeOrigin::signed(bob()),
                        contract_id,
                    )
                }
                // did not match anything => unkown call => this means you should add
                // a capture for that function here
                _ => panic!("Unknown call!"),
            };

            // the call should return what we expect it to return
            assert_eq!(
                call_to_execute.1, result,
                "The result of call to {:?} was not as expected!",
                call_to_execute.0
            );

            assert_eq!(block_number, call_to_execute.2);
        }

        self.calls_to_execute.clear();
    }
}

impl Drop for PoolState {
    fn drop(&mut self) {
        // do not panic if the thread is already panicking!
        if !thread::panicking() && self.i < self.expected_calls.len() {
            panic!("\nNot all expected calls have been executed! The following calls were still expected: {:?}\n", &self.expected_calls[self.i..]);
        }
    }
}

/// Implementation of mocked transaction pool used for testing
///
/// This transaction pool mocks submitting the transactions to the pool. It does
/// not execute the transactions. Instead it keeps them in list. It does compare
/// the submitted call to the expected call.
#[derive(Default)]
pub struct MockedTransactionPoolExt(Arc<RwLock<PoolState>>);

impl MockedTransactionPoolExt {
    /// Create new `TestTransactionPoolExt` and a reference to the internal state.
    pub fn new() -> (Self, Arc<RwLock<PoolState>>) {
        let ext = Self::default();
        let state = ext.0.clone();
        (ext, state)
    }
}

impl TransactionPool for MockedTransactionPoolExt {
    fn submit_transaction(&mut self, extrinsic: Vec<u8>) -> Result<(), ()> {
        if self.0.read().expected_calls.is_empty() {
            return Ok(());
        }

        let extrinsic_decoded: Extrinsic = match Decode::decode(&mut &*extrinsic) {
            Ok(xt) => xt,
            Err(e) => {
                log::error!("Unable to decode extrinsic: {:?}: {}", extrinsic, e);
                return Err(());
            }
        };

        if self.0.read().i < self.0.read().expected_calls.len() {
            let i = self.0.read().i.clone();

            log::debug!("Call {:?}: {:?}", i, extrinsic_decoded.call);
            // the extrinsic should match the expected call at position i
            if extrinsic_decoded.call
                != RuntimeCall::SmartContractModule(self.0.read().expected_calls[i].0.clone())
            {
                panic!(
                    "\nEXPECTED call: {:?}\nACTUAL call: {:?}\n",
                    self.0.read().expected_calls[i].0,
                    extrinsic_decoded.call
                );
            }

            // increment i for the next iteration
            let call_to_execute = self.0.read().expected_calls[i].clone();
            // we push the call to be executed later in the "test thread"
            self.0.write().calls_to_execute.push(call_to_execute);
            self.0.write().i = i + 1;

            // return the expected return value
            return self.0.read().expected_calls[i]
                .1
                .map_err(|_| ())
                .map(|_| ());
        }

        // we should not end here as it would mean we did not expect any more calls
        panic!(
            "\nDid not expect any more calls! Still have the call {:?} left.\n",
            extrinsic_decoded.call
        );
    }
}

pub fn new_test_ext_with_pool_state(
    iterations: u32,
) -> (sp_io::TestExternalities, Arc<RwLock<PoolState>>) {
    let mut ext = new_test_ext();
    let (offchain, offchain_state) = testing::TestOffchainExt::new();
    let (pool, pool_state) = MockedTransactionPoolExt::new();
    let keystore = KeyStore::new();
    keystore
        .sr25519_generate_new(KEY_TYPE, Some(&format!("//Alice")))
        .unwrap();

    let mut seed = [0_u8; 32];
    seed[0..4].copy_from_slice(&iterations.to_le_bytes());
    offchain_state.write().seed = seed;

    ext.register_extension(OffchainWorkerExt::new(offchain.clone()));
    ext.register_extension(OffchainDbExt::new(offchain));
    ext.register_extension(TransactionPoolExt::new(pool));
    ext.register_extension(KeystoreExt(Arc::new(keystore)));

    (ext, pool_state)
}
