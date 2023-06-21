#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::{DispatchResultWithPostInfo, Pays},
    ensure,
    pallet_prelude::DispatchResult,
    traits::{
        Currency, EnsureOrigin, ExistenceRequirement, ExistenceRequirement::KeepAlive, Get,
        LockableCurrency, OnUnbalanced, WithdrawReasons,
    },
    transactional, BoundedVec,
};
use frame_system::{
    self as system, ensure_signed,
    offchain::{AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer},
};
pub use pallet::*;
use pallet_authorship;
use pallet_tfgrid;
use pallet_tfgrid::pallet::{InterfaceOf, LocationOf, SerialNumberOf, TfgridNode};
use pallet_tfgrid::types as pallet_tfgrid_types;
use pallet_timestamp as timestamp;
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
    traits::{CheckedAdd, CheckedSub, Convert, SaturatedConversion, Zero},
    Perbill,
};
use sp_std::prelude::*;
use substrate_fixed::types::U64F64;
use system::offchain::SignMessage;
use tfchain_support::{
    traits::{ChangeNode, PublicIpModifier},
    types::PublicIP,
};

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"aura");

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod test_utils;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

pub mod crypto {
    use crate::KEY_TYPE;
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify,
        MultiSignature, MultiSigner,
    };
    use sp_std::convert::TryFrom;

    app_crypto!(sr25519, KEY_TYPE);

    pub struct AuthId;

    // implemented for ocw-runtime
    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for AuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }

    // implemented for mock runtime in test
    impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
        for AuthId
    {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
}

pub mod billing;
pub mod cost;
pub mod grid_contract;
pub mod migrations;
pub mod name_contract;
pub mod service_contract;
pub mod solution_provider;
pub mod types;
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use super::types::*;
    use super::weights::WeightInfo;
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_support::traits::Hooks;
    use frame_support::traits::{Currency, Get, LockIdentifier, LockableCurrency, OnUnbalanced};
    use frame_system::pallet_prelude::*;
    use parity_scale_codec::FullCodec;
    use sp_core::H256;
    use sp_std::{
        convert::{TryFrom, TryInto},
        fmt::Debug,
        vec::Vec,
    };
    use tfchain_support::traits::{ChangeNode, PublicIpModifier};

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;
    pub type NegativeImbalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::NegativeImbalance;

    pub const GRID_LOCK_ID: LockIdentifier = *b"gridlock";

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // Version constant that referenced the struct version
    pub const CONTRACT_VERSION: u32 = 4;

    pub type BillingReferencePeriod<T> = <T as Config>::BillingReferencePeriod;
    pub type MaxNodeContractPublicIPs<T> = <T as Config>::MaxNodeContractPublicIps;
    pub type MaxDeploymentDataLength<T> = <T as Config>::MaxDeploymentDataLength;
    pub type DeploymentDataInput<T> = BoundedVec<u8, MaxDeploymentDataLength<T>>;
    pub type DeploymentHash = H256;
    pub type NameContractNameOf<T> = <T as Config>::NameContractName;

    pub type MigrationStage = u8;

    #[pallet::storage]
    #[pallet::getter(fn contracts)]
    pub type Contracts<T: Config> = StorageMap<_, Blake2_128Concat, u64, Contract<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn contract_billing_information_by_id)]
    pub type ContractBillingInformationByID<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, ContractBillingInformation, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn node_contract_resources)]
    pub type NodeContractResources<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, ContractResources, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn node_contract_by_hash)]
    pub type ContractIDByNodeIDAndHash<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, u32, Blake2_128Concat, HexHash, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn active_node_contracts)]
    // A list of Contract ID's for a given node.
    // In this list, all the active contracts are kept for a node.
    pub type ActiveNodeContracts<T: Config> =
        StorageMap<_, Blake2_128Concat, u32, Vec<u64>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn contract_to_bill_at_block)]
    pub type ContractsToBillAt<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, Vec<u64>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn contract_number_of_cylces_billed)]
    pub type ContractLock<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, types::ContractLock<BalanceOf<T>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn contract_id_by_name_registration)]
    pub type ContractIDByNameRegistration<T: Config> =
        StorageMap<_, Blake2_128Concat, T::NameContractName, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn active_rent_contracts)]
    // A mapping between a Node ID and Contract ID
    // If there is an active Rent Contract for a node, the value will be the contract ID
    pub type ActiveRentContractForNode<T: Config> =
        StorageMap<_, Blake2_128Concat, u32, u64, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn contract_id)]
    pub type ContractID<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn solution_providers)]
    pub type SolutionProviders<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, types::SolutionProvider<T::AccountId>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn solution_provider_id)]
    pub type SolutionProviderID<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pallet_version)]
    pub type PalletVersion<T> = StorageValue<_, types::StorageVersion, ValueQuery>;

    #[pallet::type_value]
    pub fn DefaultBillingFrequency<T: Config>() -> u64 {
        T::BillingFrequency::get()
    }

    #[pallet::storage]
    #[pallet::getter(fn billing_frequency)]
    pub type BillingFrequency<T> = StorageValue<_, u64, ValueQuery, DefaultBillingFrequency<T>>;

    #[pallet::storage]
    #[pallet::getter(fn service_contracts)]
    pub type ServiceContracts<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, ServiceContract, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn service_contract_id)]
    pub type ServiceContractID<T> = StorageValue<_, u64, ValueQuery>;

    /// The current migration's stage, if any.
    #[pallet::storage]
    #[pallet::getter(fn current_migration_stage)]
    pub(super) type CurrentMigrationStage<T: Config> = StorageValue<_, MigrationStage, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn dedicated_nodes_extra_fee)]
    pub type DedicatedNodesExtraFee<T> = StorageMap<_, Blake2_128Concat, u32, u64, ValueQuery>;

    #[pallet::config]
    pub trait Config:
        CreateSignedTransaction<Call<Self>>
        + frame_system::Config
        + pallet_timestamp::Config
        + pallet_balances::Config
        + pallet_tfgrid::Config
        + pallet_tft_price::Config
        + pallet_authorship::Config
        + pallet_session::Config
    {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Currency: LockableCurrency<Self::AccountId>;
        /// Handler for the unbalanced decrement when slashing (burning collateral)
        type Burn: OnUnbalanced<NegativeImbalanceOf<Self>>;
        type StakingPoolAccount: Get<Self::AccountId>;
        type BillingFrequency: Get<u64>;
        type BillingReferencePeriod: Get<u64>;
        type DistributionFrequency: Get<u16>;
        type GracePeriod: Get<u64>;
        type WeightInfo: weights::WeightInfo;
        type NodeChanged: ChangeNode<LocationOf<Self>, InterfaceOf<Self>, SerialNumberOf<Self>>;
        type PublicIpModifier: PublicIpModifier;
        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
        type Call: From<Call<Self>>;

        #[pallet::constant]
        type MaxNameContractNameLength: Get<u32>;

        #[pallet::constant]
        type MaxDeploymentDataLength: Get<u32>;

        #[pallet::constant]
        type MaxNodeContractPublicIps: Get<u32>;

        /// The type of a name contract name.
        type NameContractName: FullCodec
            + Debug
            + PartialEq
            + Eq
            + Clone
            + TypeInfo
            + TryFrom<Vec<u8>, Error = Error<Self>>
            + Into<Vec<u8>>
            + MaxEncodedLen;

        type RestrictedOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A contract got created
        ContractCreated(types::Contract<T>),
        /// A contract was updated
        ContractUpdated(types::Contract<T>),
        /// A Node contract is canceled
        NodeContractCanceled {
            contract_id: u64,
            node_id: u32,
            twin_id: u32,
        },
        /// A Name contract is canceled
        NameContractCanceled {
            contract_id: u64,
        },
        /// IP got reserved by a Node contract
        IPsReserved {
            contract_id: u64,
            public_ips: BoundedVec<PublicIP, MaxNodeContractPublicIPs<T>>,
        },
        /// IP got freed by a Node contract
        IPsFreed {
            contract_id: u64,
            // public ip as a string
            public_ips: BoundedVec<PublicIP, MaxNodeContractPublicIPs<T>>,
        },
        /// Deprecated event
        ContractDeployed(u64, T::AccountId),
        /// Deprecated event
        ConsumptionReportReceived(types::Consumption),
        ContractBilled(types::ContractBill),
        /// A certain amount of tokens got burned by a contract
        TokensBurned {
            contract_id: u64,
            amount: BalanceOf<T>,
        },
        /// Contract resources got updated
        UpdatedUsedResources(types::ContractResources),
        /// Network resources report received for contract
        NruConsumptionReportReceived(types::NruConsumption),
        /// a Rent contract is canceled
        RentContractCanceled {
            contract_id: u64,
        },
        /// A Contract grace period is triggered
        ContractGracePeriodStarted {
            contract_id: u64,
            node_id: u32,
            twin_id: u32,
            block_number: u64,
        },
        /// A Contract grace period was ended
        ContractGracePeriodEnded {
            contract_id: u64,
            node_id: u32,
            twin_id: u32,
        },
        SolutionProviderCreated(types::SolutionProvider<T::AccountId>),
        SolutionProviderApproved(u64, bool),
        /// A Service contract is created
        ServiceContractCreated(types::ServiceContract),
        /// A Service contract metadata is set
        ServiceContractMetadataSet(types::ServiceContract),
        /// A Service contract fees are set
        ServiceContractFeesSet(types::ServiceContract),
        /// A Service contract is approved
        ServiceContractApproved(types::ServiceContract),
        /// A Service contract is canceled
        ServiceContractCanceled {
            service_contract_id: u64,
            cause: types::Cause,
        },
        /// A Service contract is billed
        ServiceContractBilled {
            service_contract: types::ServiceContract,
            bill: types::ServiceContractBill,
            amount: BalanceOf<T>,
        },
        BillingFrequencyChanged(u64),
        NodeExtraFeeSet {
            node_id: u32,
            extra_fee: u64,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        TwinNotExists,
        NodeNotExists,
        FarmNotExists,
        FarmHasNotEnoughPublicIPs,
        FarmHasNotEnoughPublicIPsFree,
        FailedToReserveIP,
        FailedToFreeIPs,
        ContractNotExists,
        TwinNotAuthorizedToUpdateContract,
        TwinNotAuthorizedToCancelContract,
        NodeNotAuthorizedToDeployContract,
        NodeNotAuthorizedToComputeReport,
        PricingPolicyNotExists,
        ContractIsNotUnique,
        ContractWrongBillingLoopIndex,
        NameExists,
        NameNotValid,
        InvalidContractType,
        TFTPriceValueError,
        NotEnoughResourcesOnNode,
        NodeNotAuthorizedToReportResources,
        MethodIsDeprecated,
        NodeHasActiveContracts,
        NodeHasRentContract,
        FarmIsNotDedicated,
        NodeNotAvailableToDeploy,
        CannotUpdateContractInGraceState,
        NumOverflow,
        OffchainSignedTxCannotSign,
        OffchainSignedTxAlreadySent,
        OffchainSignedTxNoLocalAccountAvailable,
        NameContractNameTooShort,
        NameContractNameTooLong,
        InvalidProviderConfiguration,
        NoSuchSolutionProvider,
        SolutionProviderNotApproved,
        TwinNotAuthorized,
        ServiceContractNotExists,
        ServiceContractCreationNotAllowed,
        ServiceContractModificationNotAllowed,
        ServiceContractApprovalNotAllowed,
        ServiceContractRejectionNotAllowed,
        ServiceContractBillingNotApprovedByBoth,
        ServiceContractBillingVariableAmountTooHigh,
        ServiceContractBillMetadataTooLong,
        ServiceContractMetadataTooLong,
        ServiceContractNotEnoughFundsToPayBill,
        CanOnlyIncreaseFrequency,
        IsNotAnAuthority,
        WrongAuthority,
        UnauthorizedToChangeSolutionProviderId,
        UnauthorizedToSetExtraFee,
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig {
        pub billing_frequency: u64,
    }

    // The default value for the genesis config type.
    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self {
                billing_frequency: 600,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            BillingFrequency::<T>::put(self.billing_frequency);
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::create_node_contract())]
        pub fn create_node_contract(
            origin: OriginFor<T>,
            node_id: u32,
            deployment_hash: HexHash,
            deployment_data: DeploymentDataInput<T>,
            public_ips: u32,
            solution_provider_id: Option<u64>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_create_node_contract(
                account_id,
                node_id,
                deployment_hash,
                deployment_data,
                public_ips,
                solution_provider_id,
            )
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::update_node_contract())]
        pub fn update_node_contract(
            origin: OriginFor<T>,
            contract_id: u64,
            deployment_hash: HexHash,
            deployment_data: DeploymentDataInput<T>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_update_node_contract(account_id, contract_id, deployment_hash, deployment_data)
        }

        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::cancel_contract())]
        pub fn cancel_contract(
            origin: OriginFor<T>,
            contract_id: u64,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_cancel_contract(account_id, contract_id, types::Cause::CanceledByUser)
        }

        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::create_name_contract())]
        pub fn create_name_contract(
            origin: OriginFor<T>,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_create_name_contract(account_id, name)
        }

        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::add_nru_reports())]
        pub fn add_nru_reports(
            origin: OriginFor<T>,
            reports: Vec<types::NruConsumption>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_compute_reports(account_id, reports)
        }

        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::report_contract_resources())]
        pub fn report_contract_resources(
            origin: OriginFor<T>,
            contract_resources: Vec<types::ContractResources>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_report_contract_resources(account_id, contract_resources)
        }

        #[pallet::call_index(7)]
        #[pallet::weight(<T as Config>::WeightInfo::create_rent_contract())]
        pub fn create_rent_contract(
            origin: OriginFor<T>,
            node_id: u32,
            solution_provider_id: Option<u64>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_create_rent_contract(account_id, node_id, solution_provider_id)
        }

        #[pallet::call_index(8)]
        #[pallet::weight(<T as Config>::WeightInfo::create_solution_provider())]
        pub fn create_solution_provider(
            origin: OriginFor<T>,
            description: Vec<u8>,
            link: Vec<u8>,
            providers: Vec<types::Provider<T::AccountId>>,
        ) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;
            Self::_create_solution_provider(description, link, providers)
        }

        #[pallet::call_index(9)]
        #[pallet::weight(<T as Config>::WeightInfo::approve_solution_provider())]
        pub fn approve_solution_provider(
            origin: OriginFor<T>,
            solution_provider_id: u64,
            approve: bool,
        ) -> DispatchResultWithPostInfo {
            <T as Config>::RestrictedOrigin::ensure_origin(origin)?;
            Self::_approve_solution_provider(solution_provider_id, approve)
        }

        #[pallet::call_index(10)]
        #[pallet::weight(<T as Config>::WeightInfo::bill_contract_for_block())]
        pub fn bill_contract_for_block(
            origin: OriginFor<T>,
            contract_id: u64,
        ) -> DispatchResultWithPostInfo {
            let _account_id = ensure_signed(origin)?;
            Self::bill_contract(contract_id)
        }

        #[pallet::call_index(11)]
        #[pallet::weight(<T as Config>::WeightInfo::service_contract_create())]
        pub fn service_contract_create(
            origin: OriginFor<T>,
            service_account: T::AccountId,
            consumer_account: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let caller_account = ensure_signed(origin)?;
            Self::_service_contract_create(caller_account, service_account, consumer_account)
        }

        #[pallet::call_index(12)]
        #[pallet::weight(<T as Config>::WeightInfo::service_contract_set_metadata())]
        pub fn service_contract_set_metadata(
            origin: OriginFor<T>,
            service_contract_id: u64,
            metadata: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_service_contract_set_metadata(account_id, service_contract_id, metadata)
        }

        #[pallet::call_index(13)]
        #[pallet::weight(<T as Config>::WeightInfo::service_contract_set_fees())]
        pub fn service_contract_set_fees(
            origin: OriginFor<T>,
            service_contract_id: u64,
            base_fee: u64,
            variable_fee: u64,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_service_contract_set_fees(
                account_id,
                service_contract_id,
                base_fee,
                variable_fee,
            )
        }

        #[pallet::call_index(14)]
        #[pallet::weight(<T as Config>::WeightInfo::service_contract_approve())]
        pub fn service_contract_approve(
            origin: OriginFor<T>,
            service_contract_id: u64,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_service_contract_approve(account_id, service_contract_id)
        }

        #[pallet::call_index(15)]
        #[pallet::weight(<T as Config>::WeightInfo::service_contract_reject())]
        pub fn service_contract_reject(
            origin: OriginFor<T>,
            service_contract_id: u64,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_service_contract_reject(account_id, service_contract_id)
        }

        #[pallet::call_index(16)]
        #[pallet::weight(<T as Config>::WeightInfo::service_contract_cancel())]
        pub fn service_contract_cancel(
            origin: OriginFor<T>,
            service_contract_id: u64,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_service_contract_cancel(
                account_id,
                service_contract_id,
                types::Cause::CanceledByUser,
            )
        }

        #[pallet::call_index(17)]
        #[pallet::weight(<T as Config>::WeightInfo::service_contract_bill())]
        pub fn service_contract_bill(
            origin: OriginFor<T>,
            service_contract_id: u64,
            variable_amount: u64,
            metadata: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_service_contract_bill(account_id, service_contract_id, variable_amount, metadata)
        }

        #[pallet::call_index(18)]
        #[pallet::weight(<T as Config>::WeightInfo::change_billing_frequency())]
        pub fn change_billing_frequency(
            origin: OriginFor<T>,
            frequency: u64,
        ) -> DispatchResultWithPostInfo {
            <T as Config>::RestrictedOrigin::ensure_origin(origin)?;
            Self::_change_billing_frequency(frequency)
        }

        #[pallet::call_index(19)]
        #[pallet::weight(<T as Config>::WeightInfo::attach_solution_provider_id())]
        pub fn attach_solution_provider_id(
            origin: OriginFor<T>,
            contract_id: u64,
            solution_provider_id: u64,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_attach_solution_provider_id(account_id, contract_id, solution_provider_id)
        }

        #[pallet::call_index(20)]
        #[pallet::weight(<T as Config>::WeightInfo::set_dedicated_node_extra_fee())]
        pub fn set_dedicated_node_extra_fee(
            origin: OriginFor<T>,
            node_id: u32,
            extra_fee: u64,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_set_dedicated_node_extra_fee(account_id, node_id, extra_fee)
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            let mut weight_used = Weight::zero();
            if let Some(migration_stage) = CurrentMigrationStage::<T>::get() {
                let (w, new_migration_stage) =
                    migrations::v9::clean_pallet_smart_contract::<T>(migration_stage);
                CurrentMigrationStage::<T>::set(new_migration_stage);
                weight_used.saturating_accrue(w);
            }
            weight_used
        }

        fn offchain_worker(block_number: T::BlockNumber) {
            // Let offchain worker check if there are contracts on the map at current index
            let current_index = Self::get_current_billing_loop_index();

            let contract_ids = ContractsToBillAt::<T>::get(current_index);
            if contract_ids.is_empty() {
                log::info!(
                    "No contracts to bill at block {:?}, index: {:?}",
                    block_number,
                    current_index
                );
                return;
            }

            log::info!(
                "{:?} contracts to bill at block {:?}",
                contract_ids,
                block_number
            );

            for contract_id in contract_ids {
                if let Some(c) = Contracts::<T>::get(contract_id) {
                    if let types::ContractData::NodeContract(node_contract) = c.contract_type {
                        // Is there IP consumption to bill?
                        let bill_ip = node_contract.public_ips > 0;

                        // Is there CU/SU consumption to bill?
                        // No need for preliminary call to contains_key() because default resource value is empty
                        let bill_cu_su =
                            !NodeContractResources::<T>::get(contract_id).used.is_empty();

                        // Is there NU consumption to bill?
                        // No need for preliminary call to contains_key() because default amount_unbilled is 0
                        let bill_nu = ContractBillingInformationByID::<T>::get(contract_id)
                            .amount_unbilled
                            > 0;

                        // Don't bill if no IP/CU/SU/NU to be billed
                        if !bill_ip && !bill_cu_su && !bill_nu {
                            continue;
                        }
                    }
                }
                let _res = Self::bill_contract_using_signed_transaction(contract_id);
            }
        }
    }
}

use crate::types::HexHash;
use sp_std::convert::{TryFrom, TryInto};

// Internal functions of the pallet
impl<T: Config> Pallet<T> {
    // Calculates the total cost of a report.
    // Takes in a report for NRU (network resource units)
    // Updates the contract's billing information in storage
    pub fn _calculate_report_cost(
        report: &types::NruConsumption,
        pricing_policy: &pallet_tfgrid_types::PricingPolicy<T::AccountId>,
    ) {
        let mut contract_billing_info =
            ContractBillingInformationByID::<T>::get(report.contract_id);
        if report.timestamp < contract_billing_info.last_updated {
            return;
        }

        // seconds elapsed is the report.window
        let seconds_elapsed = report.window;
        log::debug!("seconds elapsed: {:?}", seconds_elapsed);

        // calculate NRU used and the cost
        let used_nru = U64F64::from_num(report.nru) / pricing_policy.nu.factor_base_1000();
        let nu_cost = used_nru
            * (U64F64::from_num(pricing_policy.nu.value)
                / U64F64::from_num(T::BillingReferencePeriod::get()))
            * U64F64::from_num(seconds_elapsed);
        log::debug!("nu cost: {:?}", nu_cost);

        // save total
        let total = nu_cost.round().to_num::<u64>();
        log::debug!("total cost: {:?}", total);

        // update contract billing info
        contract_billing_info.amount_unbilled += total;
        contract_billing_info.last_updated = report.timestamp;
        ContractBillingInformationByID::<T>::insert(report.contract_id, &contract_billing_info);
    }

    pub fn _set_dedicated_node_extra_fee(
        account_id: T::AccountId,
        node_id: u32,
        extra_fee: u64,
    ) -> DispatchResultWithPostInfo {
        // Make sure only the farmer that owns this node can set the extra fee
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(Error::<T>::TwinNotExists)?;
        let node = pallet_tfgrid::Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;
        let farm = pallet_tfgrid::Farms::<T>::get(node.farm_id).ok_or(Error::<T>::FarmNotExists)?;
        ensure!(
            twin_id == farm.twin_id,
            Error::<T>::UnauthorizedToSetExtraFee
        );

        // Make sure there is no active node or rent contract on this node
        ensure!(
            ActiveRentContractForNode::<T>::get(node_id).is_none()
                && ActiveNodeContracts::<T>::get(&node_id).is_empty(),
            Error::<T>::NodeHasActiveContracts
        );

        // Set fee in mUSD
        DedicatedNodesExtraFee::<T>::insert(node_id, extra_fee);
        Self::deposit_event(Event::NodeExtraFeeSet { node_id, extra_fee });

        Ok(().into())
    }
}

impl<T: Config> ChangeNode<LocationOf<T>, InterfaceOf<T>, SerialNumberOf<T>> for Pallet<T> {
    fn node_changed(_node: Option<&TfgridNode<T>>, _new_node: &TfgridNode<T>) {}

    fn node_deleted(node: &TfgridNode<T>) {
        // Clean up all active contracts
        let active_node_contracts = ActiveNodeContracts::<T>::get(node.id);
        for node_contract_id in active_node_contracts {
            if let Some(mut contract) = Contracts::<T>::get(node_contract_id) {
                // Bill contract
                let _ = Self::update_contract_state(
                    &mut contract,
                    &types::ContractState::Deleted(types::Cause::CanceledByUser),
                );
                let _ = Self::bill_contract(node_contract_id);
            }
        }

        // First clean up rent contract if it exists
        if let Some(rc_id) = ActiveRentContractForNode::<T>::get(node.id) {
            if let Some(mut contract) = Contracts::<T>::get(rc_id) {
                // Bill contract
                let _ = Self::update_contract_state(
                    &mut contract,
                    &types::ContractState::Deleted(types::Cause::CanceledByUser),
                );
                let _ = Self::bill_contract(contract.contract_id);
            }
        }
    }
}

impl<T: Config> Pallet<T> {
    // Validates if the given signer is the next block author based on the validators in session
    // This can be used if an extrinsic should be refunded by the author in the same block
    // It also requires that the keytype inserted for the offchain workers is the validator key
    fn is_next_block_author(
        signer: &Signer<T, <T as Config>::AuthorityId>,
    ) -> Result<(), Error<T>> {
        let author = <pallet_authorship::Pallet<T>>::author();
        let validators = <pallet_session::Pallet<T>>::validators();

        // Sign some arbitrary data in order to get the AccountId, maybe there is another way to do this?
        let signed_message = signer.sign_message(&[0]);
        if let Some(signed_message_data) = signed_message {
            if let Some(block_author) = author {
                let validator =
                    <T as pallet_session::Config>::ValidatorIdOf::convert(block_author.clone())
                        .ok_or(Error::<T>::IsNotAnAuthority)?;

                let validator_count = validators.len();
                let author_index = (validators.iter().position(|a| a == &validator).unwrap_or(0)
                    + 1)
                    % validator_count;

                let signer_validator_account =
                    <T as pallet_session::Config>::ValidatorIdOf::convert(
                        signed_message_data.0.id.clone(),
                    )
                    .ok_or(Error::<T>::IsNotAnAuthority)?;

                if signer_validator_account != validators[author_index] {
                    return Err(Error::<T>::WrongAuthority);
                }
            }
        }

        Ok(().into())
    }
}
