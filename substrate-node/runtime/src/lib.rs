#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use pallet_grandpa::fg_primitives;
use pallet_grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, Encode, OpaqueMetadata};
use sp_runtime::traits::{
    AccountIdLookup, BlakeTwo256, Block as BlockT, IdentifyAccount, NumberFor, OpaqueKeys,
    SaturatedConversion, Verify,
};
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, MultiSignature,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use tfchain_support::{traits::ChangeNode, types::Node};

// A few exports that help ease life for downstream crates.
pub use frame_support::{
    construct_runtime, debug, parameter_types,
    traits::{FindAuthor, KeyOwnerProofSystem, Randomness},
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
        IdentityFee, Weight,
    },
    StorageValue,
};
use frame_system::{EnsureOneOf, EnsureRoot};
pub use pallet_collective;
pub use pallet_membership;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};

use sp_core::u32_trait::{_3, _5};

use pallet_transaction_payment::CurrencyAdapter;

pub mod impls;

/// Import the template pallet.
pub use pallet_tfgrid;

pub use pallet_smart_contract;

pub use pallet_tft_bridge;

pub use pallet_tft_price;

pub use pallet_session;

pub use pallet_burning;

pub use pallet_kvstore;

pub use pallet_runtime_upgrade;

pub use pallet_validator;

pub use pallet_dao;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    use super::*;

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;

    impl_opaque_keys! {
        pub struct SessionKeys {
            pub aura: Aura,
            pub grandpa: Grandpa,
        }
    }
}

impl_opaque_keys! {
    pub struct SessionKeys {
        pub grandpa: Grandpa,
        pub aura: Aura,
    }
}

pub fn session_keys(aura: AuraId, grandpa: GrandpaId) -> SessionKeys {
    SessionKeys { aura, grandpa }
}

/// Constant values used within the runtime.
pub mod constants;
use constants::fee::*;

pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("substrate-threefold"),
    impl_name: create_runtime_str!("substrate-threefold"),
    authoring_version: 1,
    spec_version: 61,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
};

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 6000;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
    pub const Version: RuntimeVersion = VERSION;
    pub const BlockHashCount: BlockNumber = 2400;
    /// We allow for 2 seconds of compute with a 6 second average block time.
    pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
        ::with_sensible_defaults(2 * WEIGHT_PER_SECOND, NORMAL_DISPATCH_RATIO);
    pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
        ::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub const SS58Prefix: u8 = 42;
    pub const MaximumBlockWeight: Weight = 2 * WEIGHT_PER_SECOND;

    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * MaximumBlockWeight::get();
    pub const MaxScheduledPerBlock: u32 = 50;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
    /// The basic call filter to use in dispatchable.
    type BaseCallFilter = ();
    /// Block & extrinsics weights: base values and limits.
    type BlockWeights = BlockWeights;
    /// The maximum length of a block (in bytes).
    type BlockLength = BlockLength;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = AccountIdLookup<AccountId, ()>;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Index;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type Event = Event;
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    /// Version of the runtime.
    type Version = Version;
    /// Converts a module to the index of the module in `construct_runtime!`.
    ///
    /// This type is being generated by `construct_runtime!`.
    type PalletInfo = PalletInfo;
    /// What to do if a new account is created.
    type OnNewAccount = ();
    /// What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// Weight information for the extrinsics of this pallet.
    type SystemWeightInfo = ();
    /// This is used as an identifier of the chain. 42 is the generic substrate prefix.
    type SS58Prefix = SS58Prefix;
}

impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
}

impl pallet_grandpa::Config for Runtime {
    type Event = Event;
    type Call = Call;

    type KeyOwnerProofSystem = ();

    type KeyOwnerProof =
        <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

    type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        GrandpaId,
    )>>::IdentificationTuple;

    type HandleEquivocation = ();

    type WeightInfo = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = Aura;
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 500;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = MaxLocks;
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const TransactionByteFee: Balance = 1;
}

impl pallet_transaction_payment::Config for Runtime {
    type OnChargeTransaction = CurrencyAdapter<Balances, impls::DealWithFees<Runtime>>;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = WeightToFee;
    type FeeMultiplierUpdate = ();
}

impl pallet_sudo::Config for Runtime {
    type Event = Event;
    type Call = Call;
}

pub struct NodeChanged;
impl ChangeNode for NodeChanged {
    fn node_changed(old_node: Option<&Node>, new_node: &Node) {
        Dao::node_changed(old_node, new_node)
    }

    fn node_deleted(node: &Node) {
        SmartContractModule::node_deleted(node);
        Dao::node_deleted(node);
    }
}

impl pallet_tfgrid::Config for Runtime {
    type Event = Event;
    type RestrictedOrigin = EnsureRootOrCouncilApproval;
    type WeightInfo = pallet_tfgrid::weights::SubstrateWeight<Runtime>;
    type NodeChanged = NodeChanged;
}

parameter_types! {
    pub StakingPoolAccount: AccountId = get_staking_pool_account();
    pub BillingFrequency: u64 = 600;
    pub GracePeriod: u64 = (6 * HOURS).into();
    pub DistributionFrequency: u16 = 24;
}

pub fn get_staking_pool_account() -> AccountId {
    // decoded public key from staking pool account 5CNposRewardAccount11111111111111111111111111FSU
    AccountId::from([
        13, 209, 209, 166, 229, 163, 90, 168, 199, 245, 229, 126, 30, 221, 12, 63, 189, 106, 191,
        46, 170, 142, 244, 37, 72, 152, 110, 84, 162, 86, 32, 0,
    ])
}

impl pallet_smart_contract::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type StakingPoolAccount = StakingPoolAccount;
    type BillingFrequency = BillingFrequency;
    type DistributionFrequency = DistributionFrequency;
    type GracePeriod = GracePeriod;
    type WeightInfo = pallet_smart_contract::weights::SubstrateWeight<Runtime>;
    // type Tfgrid = TfgridModule;
    type NodeChanged = NodeChanged;
}

impl pallet_tft_bridge::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type Burn = ();
    type RestrictedOrigin = EnsureRootOrCouncilApproval;
}

impl pallet_burning::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type Burn = ();
}

impl pallet_kvstore::Config for Runtime {
    type Event = Event;
}

pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;

impl pallet_tft_price::Config for Runtime {
    type AuthorityId = pallet_tft_price::crypto::AuthId;
    type Call = Call;
    type Event = Event;
    type RestrictedOrigin = EnsureRootOrCouncilApproval;
}

impl pallet_validator::Config for Runtime {
    type Event = Event;
    type CouncilOrigin = EnsureRootOrCouncilApproval;
    type Currency = Balances;
}

impl validatorset::Config for Runtime {
    type Event = Event;
    type AddRemoveOrigin = EnsureRootOrCouncilApproval;
}

parameter_types! {
    pub const DaoMotionDuration: BlockNumber = 7 * DAYS;
    pub const MinVetos: u32 = 3;
}

impl pallet_dao::Config for Runtime {
    type Event = Event;
    type CouncilOrigin = EnsureRootOrCouncilApproval;
    type Proposal = Call;
    type MotionDuration = DaoMotionDuration;
    type Tfgrid = TfgridModule;
    type NodeChanged = NodeChanged;
    type WeightInfo = pallet_dao::weights::SubstrateWeight<Runtime>;
    type MinVetos = MinVetos;
}

/// Special `FullIdentificationOf` implementation that is returning for every input `Some(Default::default())`.
pub struct FullIdentificationOf;
impl sp_runtime::traits::Convert<AccountId, Option<()>> for FullIdentificationOf {
    fn convert(_: AccountId) -> Option<()> {
        Some(Default::default())
    }
}

impl pallet_session::historical::Config for Runtime {
    type FullIdentification = ();
    type FullIdentificationOf = FullIdentificationOf;
}

/// Special `ValidatorIdOf` implementation that is just returning the input as result.
pub struct ValidatorIdOf;
impl sp_runtime::traits::Convert<AccountId, Option<AccountId>> for ValidatorIdOf {
    fn convert(a: AccountId) -> Option<AccountId> {
        Some(a)
    }
}

impl pallet_session::Config for Runtime {
    type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type ShouldEndSession = ValidatorSet;
    type SessionManager = ValidatorSet;
    type Event = Event;
    type Keys = opaque::SessionKeys;
    type NextSessionRotation = ValidatorSet;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = validatorset::ValidatorOf<Self>;
    type DisabledValidatorsThreshold = ();
    type WeightInfo = ();
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
    Call: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: Call,
        public: <Signature as sp_runtime::traits::Verify>::Signer,
        account: AccountId,
        index: Index,
    ) -> Option<(
        Call,
        <UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload,
    )> {
        let period = BlockHashCount::get() as u64;
        let current_block = System::block_number()
            .saturated_into::<u64>()
            .saturating_sub(1);
        let tip = 0;
        let extra: SignedExtra = (
            frame_system::CheckSpecVersion::<Runtime>::new(),
            frame_system::CheckTxVersion::<Runtime>::new(),
            frame_system::CheckGenesis::<Runtime>::new(),
            frame_system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
            frame_system::CheckNonce::<Runtime>::from(index),
            frame_system::CheckWeight::<Runtime>::new(),
            pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
        );

        #[cfg_attr(not(feature = "std"), allow(unused_variables))]
        let raw_payload = SignedPayload::new(call, extra)
            .map_err(|e| {
                debug::native::info!("SignedPayload error: {:?}", e);
            })
            .ok()?;

        let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;

        let address = account;
        let (call, extra, _) = raw_payload.deconstruct();
        Some((
            call,
            (sp_runtime::MultiAddress::Id(address), signature, extra),
        ))
    }
}

impl frame_system::offchain::SigningTypes for Runtime {
    type Public = <Signature as sp_runtime::traits::Verify>::Signer;
    type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
    Call: From<C>,
{
    type OverarchingCall = Call;
    type Extrinsic = UncheckedExtrinsic;
}

impl pallet_scheduler::Config for Runtime {
    type Event = Event;
    type Origin = Origin;
    type PalletsOrigin = OriginCaller;
    type Call = Call;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = frame_system::EnsureRoot<AccountId>;
    type MaxScheduledPerBlock = MaxScheduledPerBlock;
    type WeightInfo = ();
}

parameter_types! {
    pub const CouncilMotionDuration: BlockNumber = 7 * DAYS;
    pub const CouncilMaxProposals: u32 = 100;
    pub const CouncilMaxMembers: u32 = 100;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
    type MotionDuration = CouncilMotionDuration;
    type MaxProposals = CouncilMaxProposals;
    type MaxMembers = CouncilMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = ();
}

impl pallet_membership::Config<pallet_membership::Instance1> for Runtime {
    type Event = Event;
    type AddOrigin = EnsureRootOrCouncilApproval;
    type RemoveOrigin = EnsureRootOrCouncilApproval;
    type SwapOrigin = EnsureRootOrCouncilApproval;
    type ResetOrigin = EnsureRootOrCouncilApproval;
    type PrimeOrigin = EnsureRootOrCouncilApproval;
    type MembershipInitialized = Council;
    type MembershipChanged = MembershipChangedGroup;
}

use frame_support::traits::ChangeMembers;

pub struct MembershipChangedGroup;
impl ChangeMembers<AccountId> for MembershipChangedGroup {
    fn change_members_sorted(
        incoming: &[AccountId],
        outgoing: &[AccountId],
        sorted_new: &[AccountId],
    ) {
        Council::change_members_sorted(incoming, outgoing, sorted_new);
        // Validator::change_members_sorted(incoming, outgoing, sorted_new);
    }
}

type EnsureRootOrCouncilApproval = EnsureOneOf<
    AccountId,
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<_3, _5, AccountId, CouncilCollective>,
>;

impl pallet_runtime_upgrade::Config for Runtime {
    type Event = Event;
    type SetCodeOrigin = EnsureRootOrCouncilApproval;
}

pub struct AuraAccountAdapter;
use sp_runtime::ConsensusEngineId;

impl FindAuthor<AccountId> for AuraAccountAdapter {
    fn find_author<'a, I>(digests: I) -> Option<AccountId>
    where
        I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
    {
        if let Some(index) = pallet_aura::Module::<Runtime>::find_author(digests) {
            let validator = pallet_session::Module::<Runtime>::validators()[index as usize].clone();
            Some(validator)
        } else {
            None
        }
    }
}

parameter_types! {
    pub const UncleGenerations: u32 = 0;
}

impl pallet_authorship::Config for Runtime {
    type FindAuthor = AuraAccountAdapter;
    type UncleGenerations = UncleGenerations;
    type FilterUncle = ();
    type EventHandler = ();
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage},
        Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        Session: pallet_session::{Module, Call, Storage, Event, Config<T>},
        ValidatorSet: validatorset::{Module, Call, Storage, Event<T>, Config<T>},
        Aura: pallet_aura::{Module, Config<T>},
        Grandpa: pallet_grandpa::{Module, Call, Storage, Config, Event},
        TransactionPayment: pallet_transaction_payment::{Module, Storage},
        Sudo: pallet_sudo::{Module, Call, Config<T>, Storage, Event<T>},
        Authorship: pallet_authorship::{Module, Call, Storage, Inherent},
        TfgridModule: pallet_tfgrid::{Module, Call, Config<T>, Storage, Event<T>},
        SmartContractModule: pallet_smart_contract::{Module, Call, Storage, Event<T>},
        TFTBridgeModule: pallet_tft_bridge::{Module, Call, Config<T>, Storage, Event<T>},
        TFTPriceModule: pallet_tft_price::{Module, Call, Storage, Config<T>, Event<T>},
        Scheduler: pallet_scheduler::{Module, Call, Storage, Event<T>},
        BurningModule: pallet_burning::{Module, Call, Storage, Event<T>},
        TFKVStore: pallet_kvstore::{Module, Call, Storage, Event<T>},
        Council: pallet_collective::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},
        CouncilMembership: pallet_membership::<Instance1>::{Module, Call, Storage, Event<T>, Config<T>},
        RuntimeUpgrade: pallet_runtime_upgrade::{Module, Call, Event},
        Validator: pallet_validator::{Module, Call, Storage, Event<T>},
        Dao: pallet_dao::{Module, Call, Storage, Event<T>},
    }
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllModules,
>;

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            Runtime::metadata().into()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) ->
            Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }

        fn random_seed() -> <Block as BlockT>::Hash {
            RandomnessCollectiveFlip::random_seed()
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> u64 {
            Aura::slot_duration()
        }

        fn authorities() -> Vec<AuraId> {
            Aura::authorities()
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            opaque::SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            _equivocation_proof: fg_primitives::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            _key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            None
        }

        fn generate_key_ownership_proof(
            _set_id: fg_primitives::SetId,
            _authority_id: GrandpaId,
        ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
            // NOTE: this is the only implementation possible since we've
            // defined our key owner proof type as a bottom type (i.e. a type
            // with no values).
            None
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
        for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

            use frame_system_benchmarking::Module as SystemBench;
            impl frame_system_benchmarking::Config for Runtime {}

            let whitelist: Vec<TrackedStorageKey> = vec![
                // Block Number
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac")
                    .to_vec().into(),
                // Total Issuance
                hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80")
                    .to_vec().into(),
                // Execution Phase
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a")
                    .to_vec().into(),
                // Event Count
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850")
                    .to_vec().into(),
                // System Events
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7")
                    .to_vec().into(),
            ];

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);

            add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
            add_benchmark!(params, batches, pallet_balances, Balances);
            add_benchmark!(params, batches, pallet_timestamp, Timestamp);
            add_benchmark!(params, batches, pallet_tfgrid, TfgridModule);
            add_benchmark!(params, batches, pallet_smart_contract, SmartContractModule);
            add_benchmark!(params, batches, pallet_dao, Dao);

            if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
            Ok(batches)
        }
    }
}
