#![cfg(test)]
use std::{panic, thread};

use super::*;
use crate::name_contract::NameContractName;
use crate::{self as pallet_smart_contract, types::BlockNumber};
use codec::{alloc::sync::Arc, Decode};
use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU32, GenesisBuild},
    weights::PostDispatchInfo,
    BoundedVec,
};
use frame_system::EnsureRoot;
use pallet_tfgrid::node::{CityName, CountryName};
use pallet_tfgrid::{
    farm::FarmName,
    interface::{InterfaceIp, InterfaceMac, InterfaceName},
    node::{Location, SerialNumber},
    pub_config::{Domain, GW4, GW6, IP4, IP6},
    pub_ip::{GatewayIP, PublicIP},
    terms_cond::TermsAndConditions,
    twin::TwinIp,
    DocumentHashInput, DocumentLinkInput, PublicIpGatewayInput, PublicIpIpInput, TwinIpInput,
};
use pallet_tfgrid::{CityNameInput, CountryNameInput, LatitudeInput, LongitudeInput};
use parking_lot::RwLock;
use sp_core::{
    crypto::Ss58Codec,
    offchain::{
        testing::{self},
        OffchainDbExt, OffchainWorkerExt, TransactionPoolExt,
    },
    sr25519, Pair, Public, H256,
};
use sp_keystore::{testing::KeyStore, KeystoreExt, SyncCryptoStore};
use sp_runtime::{offchain::TransactionPool, MultiSignature};
use sp_keystore::{testing::KeyStore, KeystoreExt, SyncCryptoStore};
use sp_runtime::{
    testing::{Header, TestXt},
    traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentityLookup},
    AccountId32,
};
use sp_std::convert::{TryFrom, TryInto};
use tfchain_support::traits::ChangeNode;

// set environment variable RUST_LOG=debug to see all logs when running the tests and call
// env_logger::init() at the beginning of the test
use env_logger;

pub type Signature = MultiSignature;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Moment = u64;

pub type Extrinsic = TestXt<Call, ()>;
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
        TFTPriceModule: pallet_tft_price::{Pallet, Call, Storage, Event<T>}
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(1024);
    pub const ExistentialDeposit: u64 = 1;
    pub StakingPoolAccount: AccountId = get_staking_pool_account();
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
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<TestRuntime>;
}

pub(crate) type Serial = pallet_tfgrid::pallet::SerialNumberOf<TestRuntime>;
pub(crate) type Loc = pallet_tfgrid::pallet::LocationOf<TestRuntime>;
pub(crate) type PubConfig = pallet_tfgrid::pallet::PubConfigOf<TestRuntime>;
pub(crate) type Interface = pallet_tfgrid::pallet::InterfaceOf<TestRuntime>;

pub(crate) type TfgridNode = pallet_tfgrid::pallet::TfgridNode<TestRuntime>;

pub struct NodeChanged;
impl ChangeNode<Loc, PubConfig, Interface, Serial> for NodeChanged {
    fn node_changed(_old_node: Option<&TfgridNode>, _new_node: &TfgridNode) {}
    fn node_deleted(node: &TfgridNode) {
        SmartContractModule::node_deleted(node);
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
pub(crate) type TestPublicIP = PublicIP<TestRuntime>;
pub(crate) type TestGatewayIP = GatewayIP<TestRuntime>;

pub(crate) type TestIP4 = IP4<TestRuntime>;
pub(crate) type TestGW4 = GW4<TestRuntime>;
pub(crate) type TestIP6 = IP6<TestRuntime>;
pub(crate) type TestGW6 = GW6<TestRuntime>;
pub(crate) type TestDomain = Domain<TestRuntime>;

pub(crate) type TestInterfaceName = InterfaceName<TestRuntime>;
pub(crate) type TestInterfaceMac = InterfaceMac<TestRuntime>;
pub(crate) type TestInterfaceIp = InterfaceIp<TestRuntime>;

pub(crate) type TestCountryName = CountryName<TestRuntime>;
pub(crate) type TestCityName = CityName<TestRuntime>;
pub(crate) type TestLocation = Location<TestRuntime>;
pub(crate) type TestSerialNumber = SerialNumber<TestRuntime>;

impl pallet_tfgrid::Config for TestRuntime {
    type Event = Event;
    type RestrictedOrigin = EnsureRoot<Self::AccountId>;
    type WeightInfo = pallet_tfgrid::weights::SubstrateWeight<TestRuntime>;
    type NodeChanged = NodeChanged;
    type TermsAndConditions = TestTermsAndConditions;
    type TwinIp = TestTwinIp;
    type FarmName = TestFarmName;
    type MaxFarmNameLength = MaxFarmNameLength;
    type MaxFarmPublicIps = MaxFarmPublicIps;
    type PublicIP = TestPublicIP;
    type GatewayIP = TestGatewayIP;
    type IP4 = TestIP4;
    type GW4 = TestGW4;
    type IP6 = TestIP6;
    type GW6 = TestGW6;
    type Domain = TestDomain;
    type MaxInterfacesLength = MaxInterfacesLength;
    type InterfaceName = TestInterfaceName;
    type InterfaceMac = TestInterfaceMac;
    type InterfaceIP = TestInterfaceIp;
    type MaxInterfaceIpsLength = MaxInterfaceIpsLength;
    type CountryName = TestCountryName;
    type CityName = TestCityName;
    type Location = TestLocation;
    type SerialNumber = TestSerialNumber;
}

impl pallet_tft_price::Config for TestRuntime {
    type Event = Event;
    type AuthorityId = pallet_tft_price::AuthId;
    type Call = Call;
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
    pub const GracePeriod: u64 = 100;
    pub const DistributionFrequency: u16 = 24;
    pub const MaxNameContractNameLength: u32 = 64;
    pub const MaxNodeContractPublicIPs: u32 = 512;
    pub const MaxDeploymentDataLength: u32 = 512;
}

pub(crate) type TestNameContractName = NameContractName<TestRuntime>;

use weights;
impl pallet_smart_contract::Config for TestRuntime {
    type Event = Event;
    type Currency = Balances;
    type Burn = ();
    type StakingPoolAccount = StakingPoolAccount;
    type BillingFrequency = BillingFrequency;
    type DistributionFrequency = DistributionFrequency;
    type GracePeriod = GracePeriod;
    type WeightInfo = weights::SubstrateWeight<TestRuntime>;
    type NodeChanged = NodeChanged;
    type MaxNameContractNameLength = MaxNameContractNameLength;
    type NameContractName = TestNameContractName;
    type RestrictedOrigin = EnsureRoot<Self::AccountId>;
    type MaxDeploymentDataLength = MaxDeploymentDataLength;
    type MaxNodeContractPublicIps = MaxNodeContractPublicIPs;
    type AuthorityId = pallet_smart_contract::crypto::AuthId;
    type Call = Call;
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

pub(crate) fn get_public_ip_ip_input(public_ip_ip_input: &[u8]) -> PublicIpIpInput {
    BoundedVec::try_from(public_ip_ip_input.to_vec()).expect("Invalid public ip (ip) input.")
}

pub(crate) fn get_public_ip_gw_input(public_ip_gw_input: &[u8]) -> PublicIpGatewayInput {
    BoundedVec::try_from(public_ip_gw_input.to_vec()).expect("Invalid public ip (gw) input.")
}

pub(crate) fn get_public_ip_ip(public_ip_ip_input: &[u8]) -> TestPublicIP {
    let input = get_public_ip_ip_input(public_ip_ip_input);
    TestPublicIP::try_from(input).expect("Invalid public ip (ip).")
}

pub(crate) fn get_public_ip_gw(public_ip_gw_input: &[u8]) -> TestGatewayIP {
    let input = get_public_ip_gw_input(public_ip_gw_input);
    TestGatewayIP::try_from(input).expect("Invalid public ip (gw).")
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

    let genesis = pallet_tft_price::GenesisConfig::<TestRuntime> {
        allowed_origin: Some(bob()),
        min_tft_price: 10,
        max_tft_price: 1000,
    };
    genesis.assimilate_storage(&mut storage).unwrap();

    let t = sp_io::TestExternalities::from(storage);

    t
}
pub type TransactionCall = pallet_smart_contract::Call<TestRuntime>;

#[derive(Default)]
pub struct PoolState {
    /// A vector of calls that we expect should be executed
    pub expected_calls: Vec<(TransactionCall, Result<(), ()>)>,
    pub calls_to_execute: Vec<(TransactionCall, Result<(), ()>)>,
    pub i: usize,
}

impl PoolState {
    pub fn should_call_bill_contract(
        &mut self,
        contract_id: u64,
        block_number: BlockNumber,
        expected_result: Result<(), ()>,
    ) {
        self.expected_calls.push((
            crate::Call::bill_contract_for_block {
                contract_id,
                block_number,
            },
            expected_result,
        ));
    }

    pub fn should_call(&mut self, expected_call: TransactionCall, expected_result: Result<(), ()>) {
        self.expected_calls.push((expected_call, expected_result));
    }

    pub fn execute_calls_and_check_results(&mut self) {
        if self.calls_to_execute.len() == 0 {
            return;
        }
    
        // execute the calls that were submitted to the pool and compare the result
        for call_to_execute in self.calls_to_execute.iter() {
            let result = match call_to_execute.0 {
                // matches bill_contract_for_block
                crate::Call::bill_contract_for_block {
                    contract_id,
                    block_number,
                } => SmartContractModule::bill_contract_for_block(
                    Origin::signed(bob()),
                    contract_id,
                    block_number,
                ),
                // did not match anything => unkown call => this means you should add
                // a capture for that function here
                _ => panic!("Unknown call!"),
            };
    
            let result = match result {
                Ok(_) => Ok(()),
                Err(_) => Err(()),
            };
    
            // the call should return what we expect it to return
            assert_eq!(
                call_to_execute.1, result,
                "The result of call to {:?} was not as expected!",
                call_to_execute.0
            );
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

        let extrinsic_decoded: Extrinsic = Decode::decode(&mut &*extrinsic).unwrap();
        if self.0.read().i < self.0.read().expected_calls.len() {
            let i = self.0.read().i.clone();

            log::debug!("Call {:?}: {:?}", i, extrinsic_decoded.call);
            // the extrinsic should match the expected call at position i
            if extrinsic_decoded.call
                != Call::SmartContractModule(self.0.read().expected_calls[i].0.clone())
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
            return self.0.read().expected_calls[i].1;
        }

        // we should not end here as it would mean we did not expect any more calls
        panic!(
            "\nDid not expect any more calls! Still have the call {:?} left.\n",
            extrinsic_decoded.call
        );
    }
}



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
                    SmartContractModule::bill_contract_for_block(Origin::signed(bob()), contract_id)
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

        let extrinsic_decoded: Extrinsic = Decode::decode(&mut &*extrinsic).unwrap();
        if self.0.read().i < self.0.read().expected_calls.len() {
            let i = self.0.read().i.clone();

            log::debug!("Call {:?}: {:?}", i, extrinsic_decoded.call);
            // the extrinsic should match the expected call at position i
            if extrinsic_decoded.call
                != Call::SmartContractModule(self.0.read().expected_calls[i].0.clone())
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
