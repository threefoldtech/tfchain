#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo, Pays},
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

pub mod cost;
pub mod migrations;
pub mod name_contract;
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
    pub type DedicatedNodesExtraFee<T> = StorageMap<_, Blake2_128Concat, u32, u64, OptionQuery>;

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

            let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
                .ok_or(Error::<T>::TwinNotExists)?;

            Self::_service_contract_cancel(
                twin_id,
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
    pub fn _create_node_contract(
        account_id: T::AccountId,
        node_id: u32,
        deployment_hash: HexHash,
        deployment_data: DeploymentDataInput<T>,
        public_ips: u32,
        solution_provider_id: Option<u64>,
    ) -> DispatchResultWithPostInfo {
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(Error::<T>::TwinNotExists)?;

        let node = pallet_tfgrid::Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;

        let node_power = pallet_tfgrid::NodePower::<T>::get(node_id);
        ensure!(!node_power.is_down(), Error::<T>::NodeNotAvailableToDeploy);

        let farm = pallet_tfgrid::Farms::<T>::get(node.farm_id).ok_or(Error::<T>::FarmNotExists)?;

        let mut owns_rent_contract = false;
        if let Some(contract_id) = ActiveRentContractForNode::<T>::get(node_id) {
            let rent_contract =
                Contracts::<T>::get(contract_id).ok_or(Error::<T>::ContractNotExists)?;
            owns_rent_contract = rent_contract.twin_id == twin_id;
        }

        // A node is dedicated (can only be used under a rent contract)
        // if it has a dedicated node extra fee or if the farm is dedicated
        let node_is_dedicated =
            DedicatedNodesExtraFee::<T>::get(node_id).is_some() || farm.dedicated_farm;

        // If the user is not the owner of a supposed rent contract on the node and the node
        // is set to be used as dedicated then we don't allow the creation of a node contract.
        if !owns_rent_contract && node_is_dedicated {
            return Err(Error::<T>::NodeNotAvailableToDeploy.into());
        }

        // If the contract with hash and node id exists and it's in any other state then
        // contractState::Deleted then we don't allow the creation of it.
        // If it exists we allow the user to "restore" this contract
        if ContractIDByNodeIDAndHash::<T>::contains_key(node_id, &deployment_hash) {
            let contract_id = ContractIDByNodeIDAndHash::<T>::get(node_id, &deployment_hash);
            let contract = Contracts::<T>::get(contract_id).ok_or(Error::<T>::ContractNotExists)?;
            if !contract.is_state_delete() {
                return Err(Error::<T>::ContractIsNotUnique.into());
            }
        }

        let public_ips_list: BoundedVec<PublicIP, MaxNodeContractPublicIPs<T>> =
            vec![].try_into().unwrap();
        // Prepare NodeContract struct
        let node_contract = types::NodeContract {
            node_id,
            deployment_hash: deployment_hash.clone(),
            deployment_data,
            public_ips,
            public_ips_list,
        };

        // Create contract
        let contract = Self::_create_contract(
            twin_id,
            types::ContractData::NodeContract(node_contract.clone()),
            solution_provider_id,
        )?;

        let now = <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;
        let contract_billing_information = types::ContractBillingInformation {
            last_updated: now,
            amount_unbilled: 0,
            previous_nu_reported: 0,
        };
        ContractBillingInformationByID::<T>::insert(
            contract.contract_id,
            contract_billing_information,
        );

        // Insert contract id by (node_id, hash)
        ContractIDByNodeIDAndHash::<T>::insert(node_id, deployment_hash, contract.contract_id);

        // Insert contract into active contracts map
        let mut node_contracts = ActiveNodeContracts::<T>::get(&node_contract.node_id);
        node_contracts.push(contract.contract_id);
        ActiveNodeContracts::<T>::insert(&node_contract.node_id, &node_contracts);

        Self::deposit_event(Event::ContractCreated(contract));

        Ok(().into())
    }

    pub fn _create_rent_contract(
        account_id: T::AccountId,
        node_id: u32,
        solution_provider_id: Option<u64>,
    ) -> DispatchResultWithPostInfo {
        ensure!(
            !ActiveRentContractForNode::<T>::contains_key(node_id),
            Error::<T>::NodeHasRentContract
        );

        let node = pallet_tfgrid::Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;
        ensure!(
            pallet_tfgrid::Farms::<T>::contains_key(node.farm_id),
            Error::<T>::FarmNotExists
        );

        let node_power = pallet_tfgrid::NodePower::<T>::get(node_id);
        ensure!(!node_power.is_down(), Error::<T>::NodeNotAvailableToDeploy);

        let active_node_contracts = ActiveNodeContracts::<T>::get(node_id);
        let farm = pallet_tfgrid::Farms::<T>::get(node.farm_id).ok_or(Error::<T>::FarmNotExists)?;
        ensure!(
            farm.dedicated_farm || active_node_contracts.is_empty(),
            Error::<T>::NodeNotAvailableToDeploy
        );

        // Create contract
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(Error::<T>::TwinNotExists)?;
        let contract = Self::_create_contract(
            twin_id,
            types::ContractData::RentContract(types::RentContract { node_id }),
            solution_provider_id,
        )?;

        // Insert active rent contract for node
        ActiveRentContractForNode::<T>::insert(node_id, contract.contract_id);

        Self::deposit_event(Event::ContractCreated(contract));

        Ok(().into())
    }

    // Registers a DNS name for a Twin
    // Ensures uniqueness and also checks if it's a valid DNS name
    pub fn _create_name_contract(
        source: T::AccountId,
        name: Vec<u8>,
    ) -> DispatchResultWithPostInfo {
        ensure!(
            pallet_tfgrid::TwinIdByAccountID::<T>::contains_key(&source),
            Error::<T>::TwinNotExists
        );
        let twin_id =
            pallet_tfgrid::TwinIdByAccountID::<T>::get(&source).ok_or(Error::<T>::TwinNotExists)?;

        let valid_name =
            NameContractNameOf::<T>::try_from(name).map_err(DispatchErrorWithPostInfo::from)?;

        // Validate name uniqueness
        ensure!(
            !ContractIDByNameRegistration::<T>::contains_key(&valid_name),
            Error::<T>::NameExists
        );

        let name_contract = types::NameContract {
            name: valid_name.clone(),
        };

        let contract = Self::_create_contract(
            twin_id,
            types::ContractData::NameContract(name_contract),
            None,
        )?;

        ContractIDByNameRegistration::<T>::insert(valid_name, &contract.contract_id);

        Self::deposit_event(Event::ContractCreated(contract));

        Ok(().into())
    }

    fn _create_contract(
        twin_id: u32,
        mut contract_type: types::ContractData<T>,
        solution_provider_id: Option<u64>,
    ) -> Result<types::Contract<T>, DispatchErrorWithPostInfo> {
        // Get the Contract ID map and increment
        let mut id = ContractID::<T>::get();
        id = id + 1;

        if let types::ContractData::NodeContract(ref mut nc) = contract_type {
            Self::_reserve_ip(id, nc)?;
        };

        Self::validate_solution_provider(solution_provider_id)?;

        // Contract is inserted in billing loop ONLY once at contract creation
        Self::insert_contract_in_billing_loop(id);

        let contract = types::Contract {
            version: CONTRACT_VERSION,
            twin_id,
            contract_id: id,
            state: types::ContractState::Created,
            contract_type,
            solution_provider_id,
        };

        // insert into contracts map
        Contracts::<T>::insert(id, &contract);

        // Update Contract ID
        ContractID::<T>::put(id);

        let now = <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;
        let mut contract_lock = types::ContractLock::default();
        contract_lock.lock_updated = now;
        ContractLock::<T>::insert(id, contract_lock);

        Ok(contract)
    }

    pub fn _update_node_contract(
        account_id: T::AccountId,
        contract_id: u64,
        deployment_hash: HexHash,
        deployment_data: DeploymentDataInput<T>,
    ) -> DispatchResultWithPostInfo {
        let mut contract = Contracts::<T>::get(contract_id).ok_or(Error::<T>::ContractNotExists)?;
        let twin =
            pallet_tfgrid::Twins::<T>::get(contract.twin_id).ok_or(Error::<T>::TwinNotExists)?;
        ensure!(
            twin.account_id == account_id,
            Error::<T>::TwinNotAuthorizedToUpdateContract
        );

        // Don't allow updates for contracts that are in grace state
        let is_grace_state = matches!(contract.state, types::ContractState::GracePeriod(_));
        ensure!(
            !is_grace_state,
            Error::<T>::CannotUpdateContractInGraceState
        );

        let mut node_contract = Self::get_node_contract(&contract.clone())?;

        // remove and reinsert contract id by node id and hash because that hash can have changed
        ContractIDByNodeIDAndHash::<T>::remove(
            node_contract.node_id,
            node_contract.deployment_hash,
        );
        ContractIDByNodeIDAndHash::<T>::insert(
            node_contract.node_id,
            &deployment_hash,
            contract_id,
        );

        node_contract.deployment_hash = deployment_hash;
        node_contract.deployment_data = deployment_data;

        // override values
        contract.contract_type = types::ContractData::NodeContract(node_contract);

        let state = contract.state.clone();
        Self::_update_contract_state(&mut contract, &state)?;

        Self::deposit_event(Event::ContractUpdated(contract));

        Ok(().into())
    }

    pub fn _cancel_contract(
        account_id: T::AccountId,
        contract_id: u64,
        cause: types::Cause,
    ) -> DispatchResultWithPostInfo {
        let mut contract = Contracts::<T>::get(contract_id).ok_or(Error::<T>::ContractNotExists)?;
        let twin =
            pallet_tfgrid::Twins::<T>::get(contract.twin_id).ok_or(Error::<T>::TwinNotExists)?;
        ensure!(
            twin.account_id == account_id,
            Error::<T>::TwinNotAuthorizedToCancelContract
        );

        // If it's a rent contract and it still has active workloads, don't allow cancellation.
        if matches!(
            &contract.contract_type,
            types::ContractData::RentContract(_)
        ) {
            let rent_contract = Self::get_rent_contract(&contract)?;
            let active_node_contracts = ActiveNodeContracts::<T>::get(rent_contract.node_id);
            ensure!(
                active_node_contracts.len() == 0,
                Error::<T>::NodeHasActiveContracts
            );
        }

        Self::_update_contract_state(&mut contract, &types::ContractState::Deleted(cause))?;
        Self::bill_contract(contract.contract_id)?;

        Ok(().into())
    }

    pub fn _report_contract_resources(
        source: T::AccountId,
        contract_resources: Vec<types::ContractResources>,
    ) -> DispatchResultWithPostInfo {
        ensure!(
            pallet_tfgrid::TwinIdByAccountID::<T>::contains_key(&source),
            Error::<T>::TwinNotExists
        );
        let twin_id =
            pallet_tfgrid::TwinIdByAccountID::<T>::get(&source).ok_or(Error::<T>::TwinNotExists)?;
        ensure!(
            pallet_tfgrid::NodeIdByTwinID::<T>::contains_key(twin_id),
            Error::<T>::NodeNotExists
        );
        let node_id = pallet_tfgrid::NodeIdByTwinID::<T>::get(twin_id);

        for contract_resource in contract_resources {
            // we know contract exists, fetch it
            // if the node is trying to send garbage data we can throw an error here
            if let Some(contract) = Contracts::<T>::get(contract_resource.contract_id) {
                let node_contract = Self::get_node_contract(&contract)?;
                ensure!(
                    node_contract.node_id == node_id,
                    Error::<T>::NodeNotAuthorizedToComputeReport
                );

                // Do insert
                NodeContractResources::<T>::insert(contract.contract_id, &contract_resource);

                // deposit event
                Self::deposit_event(Event::UpdatedUsedResources(contract_resource));
            }
        }

        Ok(Pays::No.into())
    }

    pub fn _compute_reports(
        source: T::AccountId,
        reports: Vec<types::NruConsumption>,
    ) -> DispatchResultWithPostInfo {
        let twin_id =
            pallet_tfgrid::TwinIdByAccountID::<T>::get(&source).ok_or(Error::<T>::TwinNotExists)?;
        // fetch the node from the source account (signee)
        let node_id = pallet_tfgrid::NodeIdByTwinID::<T>::get(&twin_id);
        let node = pallet_tfgrid::Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;

        let farm = pallet_tfgrid::Farms::<T>::get(node.farm_id).ok_or(Error::<T>::FarmNotExists)?;

        let pricing_policy = pallet_tfgrid::PricingPolicies::<T>::get(farm.pricing_policy_id)
            .ok_or(Error::<T>::PricingPolicyNotExists)?;

        // validation
        for report in &reports {
            if !Contracts::<T>::contains_key(report.contract_id) {
                continue;
            }
            if !ContractBillingInformationByID::<T>::contains_key(report.contract_id) {
                continue;
            }

            // we know contract exists, fetch it
            // if the node is trying to send garbage data we can throw an error here
            let contract =
                Contracts::<T>::get(report.contract_id).ok_or(Error::<T>::ContractNotExists)?;
            let node_contract = Self::get_node_contract(&contract)?;
            ensure!(
                node_contract.node_id == node_id,
                Error::<T>::NodeNotAuthorizedToComputeReport
            );

            Self::_calculate_report_cost(&report, &pricing_policy);
            Self::deposit_event(Event::NruConsumptionReportReceived(report.clone()));
        }

        Ok(Pays::No.into())
    }

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

    fn bill_contract_using_signed_transaction(contract_id: u64) -> Result<(), Error<T>> {
        let signer = Signer::<T, <T as pallet::Config>::AuthorityId>::any_account();

        // Only allow the author of the next block to trigger the billing
        Self::is_next_block_author(&signer)?;

        if !signer.can_sign() {
            log::error!(
                "failed billing contract {:?} account cannot be used to sign transaction",
                contract_id,
            );
            return Err(<Error<T>>::OffchainSignedTxCannotSign);
        }

        let result =
            signer.send_signed_transaction(|_acct| Call::bill_contract_for_block { contract_id });

        if let Some((acc, res)) = result {
            // if res is an error this means sending the transaction failed
            // this means the transaction was already send before (probably by another node)
            // unfortunately the error is always empty (substrate just logs the error and
            // returns Err())
            if res.is_err() {
                log::error!(
                    "signed transaction failed for billing contract {:?} using account {:?}",
                    contract_id,
                    acc.id
                );
                return Err(<Error<T>>::OffchainSignedTxAlreadySent);
            }
            return Ok(());
        }
        log::error!("No local account available");
        return Err(<Error<T>>::OffchainSignedTxNoLocalAccountAvailable);
    }

    // Bills a contract (NodeContract, NameContract or RentContract)
    // Calculates how much TFT is due by the user and distributes the rewards
    fn bill_contract(contract_id: u64) -> DispatchResultWithPostInfo {
        let mut contract = Contracts::<T>::get(contract_id).ok_or(Error::<T>::ContractNotExists)?;

        let twin =
            pallet_tfgrid::Twins::<T>::get(contract.twin_id).ok_or(Error::<T>::TwinNotExists)?;
        let usable_balance = Self::get_usable_balance(&twin.account_id);
        let stash_balance = Self::get_stash_balance(twin.id);
        let total_balance = usable_balance
            .checked_add(&stash_balance)
            .unwrap_or(BalanceOf::<T>::zero());

        let now = <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;

        // Calculate amount of seconds elapsed based on the contract lock struct
        let mut contract_lock = ContractLock::<T>::get(contract.contract_id);
        let seconds_elapsed = now.checked_sub(contract_lock.lock_updated).unwrap_or(0);

        // Calculate total amount due
        let (regular_amount_due, discount_received) =
            contract.calculate_contract_cost_tft(total_balance, seconds_elapsed)?;
        let extra_amount_due = match &contract.contract_type {
            types::ContractData::RentContract(rc) => {
                contract.calculate_extra_fee_cost_tft(rc.node_id, seconds_elapsed)?
            }
            _ => BalanceOf::<T>::zero(),
        };
        let amount_due = regular_amount_due
            .checked_add(&extra_amount_due)
            .unwrap_or(BalanceOf::<T>::zero());

        // If there is nothing to be paid and the contract is not in state delete, return
        // Can be that the users cancels the contract in the same block that it's getting billed
        // where elapsed seconds would be 0, but we still have to distribute rewards
        if amount_due == BalanceOf::<T>::zero() && !contract.is_state_delete() {
            log::debug!("amount to be billed is 0, nothing to do");
            return Ok(().into());
        };

        // Calculate total amount locked
        let regular_lock_amount = contract_lock
            .amount_locked
            .checked_add(&regular_amount_due)
            .unwrap_or(BalanceOf::<T>::zero());
        let extra_lock_amount = contract_lock
            .extra_amount_locked
            .checked_add(&extra_amount_due)
            .unwrap_or(BalanceOf::<T>::zero());
        let lock_amount = regular_lock_amount
            .checked_add(&extra_lock_amount)
            .unwrap_or(BalanceOf::<T>::zero());

        // Handle grace
        let contract = Self::handle_grace(&mut contract, usable_balance, lock_amount)?;

        // Only update contract lock in state (Created, GracePeriod)
        if !matches!(contract.state, types::ContractState::Deleted(_)) {
            // increment cycles billed and update the internal lock struct
            contract_lock.lock_updated = now;
            contract_lock.cycles += 1;
            contract_lock.amount_locked = regular_lock_amount;
            contract_lock.extra_amount_locked = extra_lock_amount;
        }

        // If still in grace period, no need to continue doing locking and other stuff
        if matches!(contract.state, types::ContractState::GracePeriod(_)) {
            log::info!("contract {} is still in grace", contract.contract_id);
            ContractLock::<T>::insert(contract.contract_id, &contract_lock);
            return Ok(().into());
        }

        // Handle contract lock operations
        Self::handle_lock(contract, &mut contract_lock, amount_due)?;

        // Always emit a contract billed event
        let contract_bill = types::ContractBill {
            contract_id: contract.contract_id,
            timestamp: <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000,
            discount_level: discount_received.clone(),
            amount_billed: amount_due.saturated_into::<u128>(),
        };
        Self::deposit_event(Event::ContractBilled(contract_bill));

        // If the contract is in delete state, remove all associated storage
        if matches!(contract.state, types::ContractState::Deleted(_)) {
            return Self::remove_contract(contract.contract_id);
        }

        // If contract is node contract, set the amount unbilled back to 0
        if matches!(contract.contract_type, types::ContractData::NodeContract(_)) {
            let mut contract_billing_info =
                ContractBillingInformationByID::<T>::get(contract.contract_id);
            contract_billing_info.amount_unbilled = 0;
            ContractBillingInformationByID::<T>::insert(
                contract.contract_id,
                &contract_billing_info,
            );
        }

        // Finally update the lock
        ContractLock::<T>::insert(contract.contract_id, &contract_lock);

        log::info!("successfully billed contract with id {:?}", contract_id,);

        Ok(().into())
    }

    fn handle_grace(
        contract: &mut types::Contract<T>,
        usable_balance: BalanceOf<T>,
        amount_due: BalanceOf<T>,
    ) -> Result<&mut types::Contract<T>, DispatchErrorWithPostInfo> {
        let current_block = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
        let node_id = contract.get_node_id();

        match contract.state {
            types::ContractState::GracePeriod(grace_start) => {
                // if the usable balance is recharged, we can move the contract to created state again
                if usable_balance > amount_due {
                    Self::_update_contract_state(contract, &types::ContractState::Created)?;
                    Self::deposit_event(Event::ContractGracePeriodEnded {
                        contract_id: contract.contract_id,
                        node_id,
                        twin_id: contract.twin_id,
                    });
                    // If the contract is a rent contract, also move state on associated node contracts
                    Self::handle_grace_rent_contract(contract, types::ContractState::Created)?;
                } else {
                    let diff = current_block.checked_sub(grace_start).unwrap_or(0);
                    // If the contract grace period ran out, we can decomission the contract
                    if diff >= T::GracePeriod::get() {
                        Self::_update_contract_state(
                            contract,
                            &types::ContractState::Deleted(types::Cause::OutOfFunds),
                        )?;
                    }
                }
            }
            types::ContractState::Created => {
                // if the user ran out of funds, move the contract to be in a grace period
                // dont lock the tokens because there is nothing to lock
                // we can still update the internal contract lock object to figure out later how much was due
                // whilst in grace period
                if amount_due >= usable_balance {
                    log::info!(
                        "Grace period started at block {:?} due to lack of funds",
                        current_block
                    );
                    Self::_update_contract_state(
                        contract,
                        &types::ContractState::GracePeriod(current_block),
                    )?;
                    // We can't lock the amount due on the contract's lock because the user ran out of funds
                    Self::deposit_event(Event::ContractGracePeriodStarted {
                        contract_id: contract.contract_id,
                        node_id,
                        twin_id: contract.twin_id,
                        block_number: current_block.saturated_into(),
                    });
                    // If the contract is a rent contract, also move associated node contract to grace period
                    Self::handle_grace_rent_contract(
                        contract,
                        types::ContractState::GracePeriod(current_block),
                    )?;
                }
            }
            _ => (),
        }

        Ok(contract)
    }

    fn handle_grace_rent_contract(
        contract: &mut types::Contract<T>,
        state: types::ContractState,
    ) -> DispatchResultWithPostInfo {
        match &contract.contract_type {
            types::ContractData::RentContract(rc) => {
                let active_node_contracts = ActiveNodeContracts::<T>::get(rc.node_id);
                for ctr_id in active_node_contracts {
                    let mut ctr =
                        Contracts::<T>::get(ctr_id).ok_or(Error::<T>::ContractNotExists)?;
                    Self::_update_contract_state(&mut ctr, &state)?;

                    match state {
                        types::ContractState::Created => {
                            Self::deposit_event(Event::ContractGracePeriodEnded {
                                contract_id: ctr_id,
                                node_id: rc.node_id,
                                twin_id: ctr.twin_id,
                            });
                        }
                        types::ContractState::GracePeriod(block_number) => {
                            Self::deposit_event(Event::ContractGracePeriodStarted {
                                contract_id: ctr_id,
                                node_id: rc.node_id,
                                twin_id: ctr.twin_id,
                                block_number,
                            });
                        }
                        _ => (),
                    };
                }
            }
            _ => (),
        };

        Ok(().into())
    }

    fn handle_lock(
        contract: &mut types::Contract<T>,
        contract_lock: &mut types::ContractLock<BalanceOf<T>>,
        amount_due: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
        let now = <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;

        // Only lock an amount from the user's balance if the contract is in create state
        // The lock is specified on the user's account, since a user can have multiple contracts
        // Just extend the lock with the amount due for this contract billing period (lock will be created if not exists)
        let twin =
            pallet_tfgrid::Twins::<T>::get(contract.twin_id).ok_or(Error::<T>::TwinNotExists)?;
        if matches!(contract.state, types::ContractState::Created) {
            let mut locked_balance = Self::get_locked_balance(&twin.account_id);
            locked_balance = locked_balance
                .checked_add(&amount_due)
                .unwrap_or(BalanceOf::<T>::zero());
            <T as Config>::Currency::extend_lock(
                GRID_LOCK_ID,
                &twin.account_id,
                locked_balance,
                WithdrawReasons::all(),
            );
        }

        let canceled_and_not_zero =
            contract.is_state_delete() && contract_lock.has_some_amount_locked();
        // When the cultivation rewards are ready to be distributed or it's in delete state
        // Unlock all reserved balance and distribute
        if contract_lock.cycles >= T::DistributionFrequency::get() || canceled_and_not_zero {
            // First remove the lock, calculate how much locked balance needs to be unlocked and re-lock the remaining locked balance
            let locked_balance = Self::get_locked_balance(&twin.account_id);
            let new_locked_balance =
                match locked_balance.checked_sub(&contract_lock.total_amount_locked()) {
                    Some(b) => b,
                    None => BalanceOf::<T>::zero(),
                };
            <T as Config>::Currency::remove_lock(GRID_LOCK_ID, &twin.account_id);

            // Fetch twin balance, if the amount locked in the contract lock exceeds the current unlocked
            // balance we can only transfer out the remaining balance
            // https://github.com/threefoldtech/tfchain/issues/479
            let mut twin_balance = Self::get_usable_balance(&twin.account_id);

            if new_locked_balance > <T as Config>::Currency::minimum_balance() {
                // TODO: check if this is needed
                <T as Config>::Currency::set_lock(
                    GRID_LOCK_ID,
                    &twin.account_id,
                    new_locked_balance,
                    WithdrawReasons::all(),
                );
                twin_balance = Self::get_usable_balance(&twin.account_id);
            } else {
                twin_balance = twin_balance
                    .checked_sub(&<T as Config>::Currency::minimum_balance())
                    .unwrap_or(BalanceOf::<T>::zero());
            };

            // First, distribute extra cultivation rewards if any
            if contract_lock.has_extra_amount_locked() {
                log::info!(
                    "twin balance {:?} contract lock extra amount {:?}",
                    twin_balance,
                    contract_lock.extra_amount_locked
                );

                match Self::_distribute_extra_cultivation_rewards(
                    &contract,
                    twin_balance.min(contract_lock.extra_amount_locked),
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        log::error!(
                            "error while distributing extra cultivation rewards {:?}",
                            err
                        );
                        return Err(err);
                    }
                };

                // Update twin balance after distribution
                twin_balance = Self::get_usable_balance(&twin.account_id);
            }

            log::info!(
                "twin balance {:?} contract lock amount {:?}",
                twin_balance,
                contract_lock.amount_locked
            );

            // Fetch the default pricing policy
            let pricing_policy = pallet_tfgrid::PricingPolicies::<T>::get(1)
                .ok_or(Error::<T>::PricingPolicyNotExists)?;

            // Then, distribute cultivation rewards
            match Self::_distribute_cultivation_rewards(
                &contract,
                &pricing_policy,
                twin_balance.min(contract_lock.amount_locked),
            ) {
                Ok(_) => {}
                Err(err) => {
                    log::error!("error while distributing cultivation rewards {:?}", err);
                    return Err(err);
                }
            };

            // Reset contract lock values
            contract_lock.lock_updated = now;
            contract_lock.amount_locked = BalanceOf::<T>::zero();
            contract_lock.extra_amount_locked = BalanceOf::<T>::zero();
            contract_lock.cycles = 0;
        }

        Ok(().into())
    }

    pub fn remove_contract(contract_id: u64) -> DispatchResultWithPostInfo {
        let contract = Contracts::<T>::get(contract_id).ok_or(Error::<T>::ContractNotExists)?;

        match contract.contract_type.clone() {
            types::ContractData::NodeContract(mut node_contract) => {
                if node_contract.public_ips > 0 {
                    match Self::_free_ip(contract_id, &mut node_contract) {
                        Ok(_) => (),
                        Err(e) => {
                            log::info!("error while freeing ips: {:?}", e);
                        }
                    }
                }

                // remove associated storage items
                Self::remove_active_node_contract(node_contract.node_id, contract_id);
                ContractIDByNodeIDAndHash::<T>::remove(
                    node_contract.node_id,
                    &node_contract.deployment_hash,
                );
                NodeContractResources::<T>::remove(contract_id);
                ContractBillingInformationByID::<T>::remove(contract_id);

                Self::deposit_event(Event::NodeContractCanceled {
                    contract_id,
                    node_id: node_contract.node_id,
                    twin_id: contract.twin_id,
                });
            }
            types::ContractData::NameContract(name_contract) => {
                ContractIDByNameRegistration::<T>::remove(name_contract.name);
                Self::deposit_event(Event::NameContractCanceled { contract_id });
            }
            types::ContractData::RentContract(rent_contract) => {
                ActiveRentContractForNode::<T>::remove(rent_contract.node_id);
                // Remove all associated active node contracts
                let active_node_contracts = ActiveNodeContracts::<T>::get(rent_contract.node_id);
                for node_contract in active_node_contracts {
                    Self::remove_contract(node_contract)?;
                }
                Self::deposit_event(Event::RentContractCanceled { contract_id });
            }
        };

        log::debug!("removing contract");
        Contracts::<T>::remove(contract_id);
        ContractLock::<T>::remove(contract_id);

        // Clean up contract from billing loop
        // This is the only place it should be done
        log::debug!("cleaning up deleted contract from billing loop");
        Self::remove_contract_from_billing_loop(contract_id)?;

        Ok(().into())
    }

    fn _distribute_extra_cultivation_rewards(
        contract: &types::Contract<T>,
        amount: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
        log::info!(
            "Distributing extra cultivation rewards for contract {:?} with amount {:?}",
            contract.contract_id,
            amount,
        );

        // If the amount is zero, return
        if amount == BalanceOf::<T>::zero() {
            return Ok(().into());
        }

        // Fetch source twin = dedicated node user
        let src_twin =
            pallet_tfgrid::Twins::<T>::get(contract.twin_id).ok_or(Error::<T>::TwinNotExists)?;

        // Fetch destination twin = farmer
        let dst_twin = match &contract.contract_type {
            types::ContractData::RentContract(rc) => {
                let node =
                    pallet_tfgrid::Nodes::<T>::get(rc.node_id).ok_or(Error::<T>::NodeNotExists)?;
                let farm = pallet_tfgrid::Farms::<T>::get(node.farm_id)
                    .ok_or(Error::<T>::FarmNotExists)?;
                pallet_tfgrid::Twins::<T>::get(farm.twin_id).ok_or(Error::<T>::TwinNotExists)?
            }
            _ => {
                return Err(DispatchErrorWithPostInfo::from(
                    Error::<T>::InvalidContractType,
                ));
            }
        };

        // Send 100% to the node's owner (farmer)
        log::debug!(
            "Transfering: {:?} from contract twin {:?} to farmer account {:?}",
            &amount,
            &src_twin.account_id,
            &dst_twin.account_id,
        );
        <T as Config>::Currency::transfer(
            &src_twin.account_id,
            &dst_twin.account_id,
            amount,
            ExistenceRequirement::KeepAlive,
        )?;

        Ok(().into())
    }

    // Following: https://library.threefold.me/info/threefold#/tfgrid/farming/threefold__proof_of_utilization
    fn _distribute_cultivation_rewards(
        contract: &types::Contract<T>,
        pricing_policy: &pallet_tfgrid_types::PricingPolicy<T::AccountId>,
        amount: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
        log::info!(
            "Distributing cultivation rewards for contract {:?} with amount {:?}",
            contract.contract_id,
            amount,
        );

        // If the amount is zero, return
        if amount == BalanceOf::<T>::zero() {
            return Ok(().into());
        }

        // fetch source twin
        let twin =
            pallet_tfgrid::Twins::<T>::get(contract.twin_id).ok_or(Error::<T>::TwinNotExists)?;

        // Send 10% to the foundation
        let foundation_share = Perbill::from_percent(10) * amount;
        log::debug!(
            "Transfering: {:?} from contract twin {:?} to foundation account {:?}",
            &foundation_share,
            &twin.account_id,
            &pricing_policy.foundation_account
        );
        <T as Config>::Currency::transfer(
            &twin.account_id,
            &pricing_policy.foundation_account,
            foundation_share,
            ExistenceRequirement::KeepAlive,
        )?;

        // TODO: send 5% to the staking pool account
        let staking_pool_share = Perbill::from_percent(5) * amount;
        let staking_pool_account = T::StakingPoolAccount::get();
        log::debug!(
            "Transfering: {:?} from contract twin {:?} to staking pool account {:?}",
            &staking_pool_share,
            &twin.account_id,
            &staking_pool_account,
        );
        <T as Config>::Currency::transfer(
            &twin.account_id,
            &staking_pool_account,
            staking_pool_share,
            ExistenceRequirement::KeepAlive,
        )?;

        let mut sales_share = 50;

        if let Some(provider_id) = contract.solution_provider_id {
            if let Some(solution_provider) = SolutionProviders::<T>::get(provider_id) {
                let total_take: u8 = solution_provider
                    .providers
                    .iter()
                    .map(|provider| provider.take)
                    .sum();
                sales_share -= total_take;

                if !solution_provider
                    .providers
                    .iter()
                    .map(|provider| {
                        let share = Perbill::from_percent(provider.take as u32) * amount;
                        log::debug!(
                            "Transfering: {:?} from contract twin {:?} to provider account {:?}",
                            &share,
                            &twin.account_id,
                            &provider.who
                        );
                        <T as Config>::Currency::transfer(
                            &twin.account_id,
                            &provider.who,
                            share,
                            KeepAlive,
                        )
                    })
                    .filter(|result| result.is_err())
                    .collect::<Vec<DispatchResult>>()
                    .is_empty()
                {
                    return Err(DispatchErrorWithPostInfo::from(
                        Error::<T>::InvalidProviderConfiguration,
                    ));
                }
            }
        };

        if sales_share > 0 {
            let share = Perbill::from_percent(sales_share.into()) * amount;
            // Transfer the remaining share to the sales account
            // By default it is 50%, if a contract has solution providers it can be less
            log::debug!(
                "Transfering: {:?} from contract twin {:?} to sales account {:?}",
                &share,
                &twin.account_id,
                &pricing_policy.certified_sales_account
            );
            <T as Config>::Currency::transfer(
                &twin.account_id,
                &pricing_policy.certified_sales_account,
                share,
                KeepAlive,
            )?;
        }

        // Burn 35%, to not have any imbalance in the system, subtract all previously send amounts with the initial
        let amount_to_burn =
            (Perbill::from_percent(50) * amount) - foundation_share - staking_pool_share;

        let to_burn = T::Currency::withdraw(
            &twin.account_id,
            amount_to_burn,
            WithdrawReasons::FEE,
            ExistenceRequirement::KeepAlive,
        )?;

        log::debug!(
            "Burning: {:?} from contract twin {:?}",
            amount_to_burn,
            &twin.account_id
        );
        T::Burn::on_unbalanced(to_burn);

        Self::deposit_event(Event::TokensBurned {
            contract_id: contract.contract_id,
            amount: amount_to_burn,
        });

        Ok(().into())
    }

    // Inserts a contract in a billing loop where the index is the contract id % billing frequency
    // This way, we don't need to reinsert the contract everytime it gets billed
    pub fn insert_contract_in_billing_loop(contract_id: u64) {
        let index = Self::get_contract_billing_loop_index(contract_id);
        let mut contract_ids = ContractsToBillAt::<T>::get(index);

        if !contract_ids.contains(&contract_id) {
            contract_ids.push(contract_id);
            ContractsToBillAt::<T>::insert(index, &contract_ids);
            log::debug!(
                "Updated contracts after insertion: {:?}, to be billed at index {:?}",
                contract_ids,
                index
            );
        }
    }

    // Removes contract from billing loop where the index is the contract id % billing frequency
    pub fn remove_contract_from_billing_loop(
        contract_id: u64,
    ) -> Result<(), DispatchErrorWithPostInfo> {
        let index = Self::get_contract_billing_loop_index(contract_id);
        let mut contract_ids = ContractsToBillAt::<T>::get(index);

        ensure!(
            contract_ids.contains(&contract_id),
            Error::<T>::ContractWrongBillingLoopIndex
        );

        contract_ids.retain(|&c| c != contract_id);
        ContractsToBillAt::<T>::insert(index, &contract_ids);
        log::debug!(
            "Updated contracts after removal: {:?}, to be billed at index {:?}",
            contract_ids,
            index
        );

        Ok(())
    }

    // Helper function that updates the contract state and manages storage accordingly
    pub fn _update_contract_state(
        contract: &mut types::Contract<T>,
        state: &types::ContractState,
    ) -> DispatchResultWithPostInfo {
        // update the state and save the contract
        contract.state = state.clone();
        Contracts::<T>::insert(&contract.contract_id, contract.clone());

        // if the contract is a name contract, nothing to do left here
        match contract.contract_type {
            types::ContractData::NameContract(_) => return Ok(().into()),
            types::ContractData::RentContract(_) => return Ok(().into()),
            _ => (),
        };

        // if the contract is a node contract
        // manage the ActiveNodeContracts map accordingly
        let node_contract = Self::get_node_contract(contract)?;

        let mut contracts = ActiveNodeContracts::<T>::get(&node_contract.node_id);

        match contracts.iter().position(|id| id == &contract.contract_id) {
            Some(index) => {
                // if the new contract state is delete, remove the contract id from the map
                if contract.is_state_delete() {
                    contracts.remove(index);
                }
            }
            None => {
                // if the contract is not present add it to the active contracts map
                if state == &types::ContractState::Created {
                    contracts.push(contract.contract_id);
                }
            }
        };

        ActiveNodeContracts::<T>::insert(&node_contract.node_id, &contracts);

        Ok(().into())
    }

    fn remove_active_node_contract(node_id: u32, contract_id: u64) {
        let mut contracts = ActiveNodeContracts::<T>::get(&node_id);

        match contracts.iter().position(|id| id == &contract_id) {
            Some(index) => {
                contracts.remove(index);
            }
            None => (),
        };

        ActiveNodeContracts::<T>::insert(&node_id, &contracts);
    }

    pub fn _reserve_ip(
        contract_id: u64,
        node_contract: &mut types::NodeContract<T>,
    ) -> DispatchResultWithPostInfo {
        if node_contract.public_ips == 0 {
            return Ok(().into());
        }
        let node = pallet_tfgrid::Nodes::<T>::get(node_contract.node_id)
            .ok_or(Error::<T>::NodeNotExists)?;

        let mut farm =
            pallet_tfgrid::Farms::<T>::get(node.farm_id).ok_or(Error::<T>::FarmNotExists)?;

        log::debug!(
            "Number of farm ips {:?}, number of ips to reserve: {:?}",
            farm.public_ips.len(),
            node_contract.public_ips as usize
        );
        ensure!(
            farm.public_ips.len() >= node_contract.public_ips as usize,
            Error::<T>::FarmHasNotEnoughPublicIPs
        );

        let mut ips: BoundedVec<PublicIP, MaxNodeContractPublicIPs<T>> = vec![].try_into().unwrap();

        for i in 0..farm.public_ips.len() {
            if ips.len() == node_contract.public_ips as usize {
                break;
            }

            // if an ip has contract id 0 it means it's not reserved
            // reserve it now
            if farm.public_ips[i].contract_id == 0 {
                let mut ip = farm.public_ips[i].clone();
                ip.contract_id = contract_id;
                farm.public_ips[i] = ip.clone();
                ips.try_push(ip).or_else(|_| {
                    return Err(DispatchErrorWithPostInfo::from(
                        Error::<T>::FailedToReserveIP,
                    ));
                })?;
            }
        }

        // Safeguard check if we actually have the amount of ips we wanted to reserve
        ensure!(
            ips.len() == node_contract.public_ips as usize,
            Error::<T>::FarmHasNotEnoughPublicIPsFree
        );

        node_contract.public_ips_list = ips.try_into().or_else(|_| {
            return Err(DispatchErrorWithPostInfo::from(
                Error::<T>::FailedToReserveIP,
            ));
        })?;

        // Update the farm with the reserved ips
        pallet_tfgrid::Farms::<T>::insert(farm.id, farm);

        Ok(().into())
    }

    pub fn _free_ip(
        contract_id: u64,
        node_contract: &mut types::NodeContract<T>,
    ) -> DispatchResultWithPostInfo {
        let node = pallet_tfgrid::Nodes::<T>::get(node_contract.node_id)
            .ok_or(Error::<T>::NodeNotExists)?;

        let mut farm =
            pallet_tfgrid::Farms::<T>::get(node.farm_id).ok_or(Error::<T>::FarmNotExists)?;

        let mut public_ips: BoundedVec<PublicIP, MaxNodeContractPublicIPs<T>> =
            vec![].try_into().unwrap();
        for i in 0..farm.public_ips.len() {
            // if an ip has contract id 0 it means it's not reserved
            // reserve it now
            if farm.public_ips[i].contract_id == contract_id {
                let mut ip = farm.public_ips[i].clone();
                ip.contract_id = 0;
                farm.public_ips[i] = ip.clone();
                public_ips.try_push(ip).or_else(|_| {
                    return Err(DispatchErrorWithPostInfo::from(Error::<T>::FailedToFreeIPs));
                })?;
            }
        }

        pallet_tfgrid::Farms::<T>::insert(farm.id, farm);

        // Emit an event containing the IP's freed for this contract
        Self::deposit_event(Event::IPsFreed {
            contract_id,
            public_ips,
        });

        Ok(().into())
    }

    pub fn get_node_contract(
        contract: &types::Contract<T>,
    ) -> Result<types::NodeContract<T>, DispatchErrorWithPostInfo> {
        match contract.contract_type.clone() {
            types::ContractData::NodeContract(c) => Ok(c),
            _ => {
                return Err(DispatchErrorWithPostInfo::from(
                    Error::<T>::InvalidContractType,
                ))
            }
        }
    }

    pub fn get_rent_contract(
        contract: &types::Contract<T>,
    ) -> Result<types::RentContract, DispatchErrorWithPostInfo> {
        match contract.contract_type.clone() {
            types::ContractData::RentContract(c) => Ok(c),
            _ => {
                return Err(DispatchErrorWithPostInfo::from(
                    Error::<T>::InvalidContractType,
                ))
            }
        }
    }

    // Get the usable balance of an account
    // This is the balance minus the minimum balance
    fn get_usable_balance(account_id: &T::AccountId) -> BalanceOf<T> {
        let balance = pallet_balances::pallet::Pallet::<T>::usable_balance(account_id);
        let b = balance.saturated_into::<u128>();
        BalanceOf::<T>::saturated_from(b)
    }

    fn get_locked_balance(account_id: &T::AccountId) -> BalanceOf<T> {
        let usable_balance = Self::get_usable_balance(account_id);
        let free_balance = <T as Config>::Currency::free_balance(account_id);

        let locked_balance = free_balance.checked_sub(&usable_balance);
        match locked_balance {
            Some(balance) => balance,
            None => BalanceOf::<T>::zero(),
        }
    }

    fn get_stash_balance(twin_id: u32) -> BalanceOf<T> {
        let account_id = pallet_tfgrid::TwinBoundedAccountID::<T>::get(twin_id);
        match account_id {
            Some(account) => Self::get_usable_balance(&account),
            None => BalanceOf::<T>::zero(),
        }
    }

    pub fn _create_solution_provider(
        description: Vec<u8>,
        link: Vec<u8>,
        providers: Vec<types::Provider<T::AccountId>>,
    ) -> DispatchResultWithPostInfo {
        let total_take: u8 = providers.iter().map(|provider| provider.take).sum();
        ensure!(total_take <= 50, Error::<T>::InvalidProviderConfiguration);

        let mut id = SolutionProviderID::<T>::get();
        id = id + 1;

        let solution_provider = types::SolutionProvider {
            solution_provider_id: id,
            providers,
            description,
            link,
            approved: false,
        };

        SolutionProviderID::<T>::put(id);
        SolutionProviders::<T>::insert(id, &solution_provider);

        Self::deposit_event(Event::SolutionProviderCreated(solution_provider));

        Ok(().into())
    }

    pub fn _approve_solution_provider(
        solution_provider_id: u64,
        approve: bool,
    ) -> DispatchResultWithPostInfo {
        ensure!(
            SolutionProviders::<T>::contains_key(solution_provider_id),
            Error::<T>::NoSuchSolutionProvider
        );

        if let Some(mut solution_provider) = SolutionProviders::<T>::get(solution_provider_id) {
            solution_provider.approved = approve;
            SolutionProviders::<T>::insert(solution_provider_id, &solution_provider);
            Self::deposit_event(Event::SolutionProviderApproved(
                solution_provider_id,
                approve,
            ));
        }

        Ok(().into())
    }

    pub fn validate_solution_provider(
        solution_provider_id: Option<u64>,
    ) -> DispatchResultWithPostInfo {
        if let Some(provider_id) = solution_provider_id {
            ensure!(
                SolutionProviders::<T>::contains_key(provider_id),
                Error::<T>::NoSuchSolutionProvider
            );

            if let Some(solution_provider) = SolutionProviders::<T>::get(provider_id) {
                ensure!(
                    solution_provider.approved,
                    Error::<T>::SolutionProviderNotApproved
                );
                return Ok(().into());
            }
        }
        Ok(().into())
    }

    // Billing index is contract id % (mod) Billing Frequency
    // So index belongs to [0; billing_frequency - 1] range
    pub fn get_contract_billing_loop_index(contract_id: u64) -> u64 {
        contract_id % BillingFrequency::<T>::get()
    }

    // Billing index is block number % (mod) Billing Frequency
    // So index belongs to [0; billing_frequency - 1] range
    pub fn get_current_billing_loop_index() -> u64 {
        let current_block = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
        current_block % BillingFrequency::<T>::get()
    }

    pub fn _service_contract_create(
        caller: T::AccountId,
        service: T::AccountId,
        consumer: T::AccountId,
    ) -> DispatchResultWithPostInfo {
        let caller_twin_id =
            pallet_tfgrid::TwinIdByAccountID::<T>::get(&caller).ok_or(Error::<T>::TwinNotExists)?;

        let service_twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&service)
            .ok_or(Error::<T>::TwinNotExists)?;

        let consumer_twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&consumer)
            .ok_or(Error::<T>::TwinNotExists)?;

        // Only service or consumer can create contract
        ensure!(
            caller_twin_id == service_twin_id || caller_twin_id == consumer_twin_id,
            Error::<T>::TwinNotAuthorized,
        );

        // Service twin and consumer twin can not be the same
        ensure!(
            service_twin_id != consumer_twin_id,
            Error::<T>::ServiceContractCreationNotAllowed,
        );

        // Get the service contract ID map and increment
        let mut id = ServiceContractID::<T>::get();
        id = id + 1;

        // Create service contract
        let service_contract = types::ServiceContract {
            service_contract_id: id,
            service_twin_id,
            consumer_twin_id,
            base_fee: 0,
            variable_fee: 0,
            metadata: vec![].try_into().unwrap(),
            accepted_by_service: false,
            accepted_by_consumer: false,
            last_bill: 0,
            state: types::ServiceContractState::Created,
        };

        // Insert into service contract map
        ServiceContracts::<T>::insert(id, &service_contract);

        // Update Contract ID
        ServiceContractID::<T>::put(id);

        // Trigger event for service contract creation
        Self::deposit_event(Event::ServiceContractCreated(service_contract));

        Ok(().into())
    }

    pub fn _service_contract_set_metadata(
        account_id: T::AccountId,
        service_contract_id: u64,
        metadata: Vec<u8>,
    ) -> DispatchResultWithPostInfo {
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(Error::<T>::TwinNotExists)?;

        let mut service_contract = ServiceContracts::<T>::get(service_contract_id)
            .ok_or(Error::<T>::ServiceContractNotExists)?;

        // Only service or consumer can set metadata
        ensure!(
            twin_id == service_contract.service_twin_id
                || twin_id == service_contract.consumer_twin_id,
            Error::<T>::TwinNotAuthorized,
        );

        // Only allow to modify metadata if contract still not approved by both parties
        ensure!(
            !matches!(
                service_contract.state,
                types::ServiceContractState::ApprovedByBoth
            ),
            Error::<T>::ServiceContractModificationNotAllowed,
        );

        service_contract.metadata = BoundedVec::try_from(metadata)
            .map_err(|_| Error::<T>::ServiceContractMetadataTooLong)?;

        // If base_fee is set and non-zero (mandatory)
        if service_contract.base_fee != 0 {
            service_contract.state = types::ServiceContractState::AgreementReady;
        }

        // Update service contract in map after modification
        ServiceContracts::<T>::insert(service_contract_id, service_contract.clone());

        // Trigger event for service contract metadata setting
        Self::deposit_event(Event::ServiceContractMetadataSet(service_contract));

        Ok(().into())
    }

    pub fn _service_contract_set_fees(
        account_id: T::AccountId,
        service_contract_id: u64,
        base_fee: u64,
        variable_fee: u64,
    ) -> DispatchResultWithPostInfo {
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(Error::<T>::TwinNotExists)?;

        let mut service_contract = ServiceContracts::<T>::get(service_contract_id)
            .ok_or(Error::<T>::ServiceContractNotExists)?;

        // Only service can set fees
        ensure!(
            twin_id == service_contract.service_twin_id,
            Error::<T>::TwinNotAuthorized,
        );

        // Only allow to modify fees if contract still not approved by both parties
        ensure!(
            !matches!(
                service_contract.state,
                types::ServiceContractState::ApprovedByBoth
            ),
            Error::<T>::ServiceContractModificationNotAllowed,
        );

        service_contract.base_fee = base_fee;
        service_contract.variable_fee = variable_fee;

        // If metadata is filled and not empty (mandatory)
        if !service_contract.metadata.is_empty() {
            service_contract.state = types::ServiceContractState::AgreementReady;
        }

        // Update service contract in map after modification
        ServiceContracts::<T>::insert(service_contract_id, service_contract.clone());

        // Trigger event for service contract fees setting
        Self::deposit_event(Event::ServiceContractFeesSet(service_contract));

        Ok(().into())
    }

    pub fn _service_contract_approve(
        account_id: T::AccountId,
        service_contract_id: u64,
    ) -> DispatchResultWithPostInfo {
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(Error::<T>::TwinNotExists)?;

        let mut service_contract = ServiceContracts::<T>::get(service_contract_id)
            .ok_or(Error::<T>::ServiceContractNotExists)?;

        // Allow to approve contract only if agreement is ready
        ensure!(
            matches!(
                service_contract.state,
                types::ServiceContractState::AgreementReady
            ),
            Error::<T>::ServiceContractApprovalNotAllowed,
        );

        // Only service or consumer can accept agreement
        if twin_id == service_contract.service_twin_id {
            service_contract.accepted_by_service = true;
        } else if twin_id == service_contract.consumer_twin_id {
            service_contract.accepted_by_consumer = true
        } else {
            return Err(DispatchErrorWithPostInfo::from(
                Error::<T>::TwinNotAuthorized,
            ));
        }

        // If both parties (service and consumer) accept then contract is approved and can be billed
        if service_contract.accepted_by_service && service_contract.accepted_by_consumer {
            // Change contract state to approved and emit event
            service_contract.state = types::ServiceContractState::ApprovedByBoth;

            // Initialize billing time
            let now = <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;
            service_contract.last_bill = now;
        }

        // Update service contract in map after modification
        ServiceContracts::<T>::insert(service_contract_id, service_contract.clone());

        // Trigger event for service contract approval
        Self::deposit_event(Event::ServiceContractApproved(service_contract));

        Ok(().into())
    }

    pub fn _service_contract_reject(
        account_id: T::AccountId,
        service_contract_id: u64,
    ) -> DispatchResultWithPostInfo {
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(Error::<T>::TwinNotExists)?;

        let service_contract = ServiceContracts::<T>::get(service_contract_id)
            .ok_or(Error::<T>::ServiceContractNotExists)?;

        // Only service or consumer can reject agreement
        ensure!(
            twin_id == service_contract.service_twin_id
                || twin_id == service_contract.consumer_twin_id,
            Error::<T>::TwinNotAuthorized,
        );

        // Allow to reject contract only if agreement is ready
        ensure!(
            matches!(
                service_contract.state,
                types::ServiceContractState::AgreementReady
            ),
            Error::<T>::ServiceContractRejectionNotAllowed,
        );

        // If one party (service or consumer) rejects agreement
        // then contract is canceled and removed from service contract map
        Self::_service_contract_cancel(twin_id, service_contract_id, types::Cause::CanceledByUser)?;

        Ok(().into())
    }

    pub fn _service_contract_cancel(
        twin_id: u32,
        service_contract_id: u64,
        cause: types::Cause,
    ) -> DispatchResultWithPostInfo {
        let service_contract = ServiceContracts::<T>::get(service_contract_id)
            .ok_or(Error::<T>::ServiceContractNotExists)?;

        // Only service or consumer can cancel contract
        ensure!(
            twin_id == service_contract.service_twin_id
                || twin_id == service_contract.consumer_twin_id,
            Error::<T>::TwinNotAuthorized,
        );

        // Remove contract from service contract map
        // Can be done at any state of contract
        // so no need to handle state validation
        ServiceContracts::<T>::remove(service_contract_id);

        // Trigger event for service contract cancelation
        Self::deposit_event(Event::ServiceContractCanceled {
            service_contract_id,
            cause,
        });

        log::debug!(
            "successfully removed service contract with id {:?}",
            service_contract_id,
        );

        Ok(().into())
    }

    #[transactional]
    pub fn _service_contract_bill(
        account_id: T::AccountId,
        service_contract_id: u64,
        variable_amount: u64,
        metadata: Vec<u8>,
    ) -> DispatchResultWithPostInfo {
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(Error::<T>::TwinNotExists)?;

        let mut service_contract = ServiceContracts::<T>::get(service_contract_id)
            .ok_or(Error::<T>::ServiceContractNotExists)?;

        // Only service can bill consumer for service contract
        ensure!(
            twin_id == service_contract.service_twin_id,
            Error::<T>::TwinNotAuthorized,
        );

        // Allow to bill contract only if approved by both
        ensure!(
            matches!(
                service_contract.state,
                types::ServiceContractState::ApprovedByBoth
            ),
            Error::<T>::ServiceContractBillingNotApprovedByBoth,
        );

        // Get elapsed time (in seconds) to bill for service
        let now = <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;
        let elapsed_seconds_since_last_bill = now - service_contract.last_bill;

        // Billing time (window) is max 1h by design
        // So extra time will not be billed
        // It is the service responsability to bill on right frequency
        let window = elapsed_seconds_since_last_bill.min(T::BillingReferencePeriod::get());

        // Billing variable amount is bounded by contract variable fee
        ensure!(
            variable_amount
                <= ((U64F64::from_num(window)
                    / U64F64::from_num(T::BillingReferencePeriod::get()))
                    * U64F64::from_num(service_contract.variable_fee))
                .round()
                .to_num::<u64>(),
            Error::<T>::ServiceContractBillingVariableAmountTooHigh,
        );

        let bill_metadata = BoundedVec::try_from(metadata)
            .map_err(|_| Error::<T>::ServiceContractBillMetadataTooLong)?;

        // Create service contract bill
        let service_contract_bill = types::ServiceContractBill {
            variable_amount,
            window,
            metadata: bill_metadata,
        };

        // Make consumer pay for service contract bill
        let amount =
            Self::_service_contract_pay_bill(service_contract_id, service_contract_bill.clone())?;

        // Update contract in list after modification
        service_contract.last_bill = now;
        ServiceContracts::<T>::insert(service_contract_id, service_contract.clone());

        // Trigger event for service contract billing
        Self::deposit_event(Event::ServiceContractBilled {
            service_contract,
            bill: service_contract_bill,
            amount,
        });

        Ok(().into())
    }

    // Pay a service contract bill
    // Calculates how much TFT is due by the consumer and pay the amount to the service
    fn _service_contract_pay_bill(
        service_contract_id: u64,
        bill: types::ServiceContractBill,
    ) -> Result<BalanceOf<T>, DispatchErrorWithPostInfo> {
        let service_contract = ServiceContracts::<T>::get(service_contract_id)
            .ok_or(Error::<T>::ServiceContractNotExists)?;
        let amount = service_contract.calculate_bill_cost_tft::<T>(bill.clone())?;

        let service_twin_id = service_contract.service_twin_id;
        let service_twin =
            pallet_tfgrid::Twins::<T>::get(service_twin_id).ok_or(Error::<T>::TwinNotExists)?;

        let consumer_twin = pallet_tfgrid::Twins::<T>::get(service_contract.consumer_twin_id)
            .ok_or(Error::<T>::TwinNotExists)?;

        let usable_balance = Self::get_usable_balance(&consumer_twin.account_id);

        // If consumer is out of funds then contract is canceled
        // by service and removed from service contract map
        if usable_balance < amount {
            Self::_service_contract_cancel(
                service_twin_id,
                service_contract_id,
                types::Cause::OutOfFunds,
            )?;
            return Err(DispatchErrorWithPostInfo::from(
                Error::<T>::ServiceContractNotEnoughFundsToPayBill,
            ));
        }

        // Transfer amount due from consumer account to service account
        <T as Config>::Currency::transfer(
            &consumer_twin.account_id,
            &service_twin.account_id,
            amount,
            ExistenceRequirement::KeepAlive,
        )?;

        log::debug!(
            "bill successfully payed by consumer for service contract with id {:?}",
            service_contract_id,
        );

        Ok(amount)
    }

    pub fn _change_billing_frequency(frequency: u64) -> DispatchResultWithPostInfo {
        let billing_frequency = BillingFrequency::<T>::get();
        ensure!(
            frequency > billing_frequency,
            Error::<T>::CanOnlyIncreaseFrequency
        );

        BillingFrequency::<T>::put(frequency);

        Self::deposit_event(Event::BillingFrequencyChanged(frequency));

        Ok(().into())
    }

    pub fn _attach_solution_provider_id(
        account_id: T::AccountId,
        contract_id: u64,
        solution_provider_id: u64,
    ) -> DispatchResultWithPostInfo {
        let solution_provider = SolutionProviders::<T>::get(solution_provider_id)
            .ok_or(Error::<T>::NoSuchSolutionProvider)?;
        ensure!(
            solution_provider.approved,
            Error::<T>::SolutionProviderNotApproved
        );

        let mut contract = Contracts::<T>::get(contract_id).ok_or(Error::<T>::ContractNotExists)?;

        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(Error::<T>::TwinNotExists)?;

        ensure!(
            contract.twin_id == twin_id,
            Error::<T>::UnauthorizedToChangeSolutionProviderId
        );

        match contract.solution_provider_id {
            Some(_) => {
                return Err(DispatchErrorWithPostInfo::from(
                    Error::<T>::UnauthorizedToChangeSolutionProviderId,
                ))
            }
            None => {
                contract.solution_provider_id = Some(solution_provider_id);
                Contracts::<T>::insert(contract_id, &contract);
                Self::deposit_event(Event::ContractUpdated(contract));
            }
        };

        Ok(().into())
    }

    pub fn _set_dedicated_node_extra_fee(
        account_id: T::AccountId,
        node_id: u32,
        extra_fee: u64,
    ) -> DispatchResultWithPostInfo {
        // Nothing to do if fee value is 0
        if extra_fee == 0 {
            return Ok(().into());
        }

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
                let _ = Self::_update_contract_state(
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
                let _ = Self::_update_contract_state(
                    &mut contract,
                    &types::ContractState::Deleted(types::Cause::CanceledByUser),
                );
                let _ = Self::bill_contract(contract.contract_id);
            }
        }
    }
}

impl<T: Config> PublicIpModifier for Pallet<T> {
    fn ip_removed(ip: &PublicIP) {
        if let Some(mut contract) = Contracts::<T>::get(ip.contract_id) {
            match contract.contract_type {
                types::ContractData::NodeContract(mut node_contract) => {
                    if node_contract.public_ips > 0 {
                        if let Err(e) = Self::_free_ip(ip.contract_id, &mut node_contract) {
                            log::error!("error while freeing ips: {:?}", e);
                        }
                    }
                    contract.contract_type = types::ContractData::NodeContract(node_contract);

                    Contracts::<T>::insert(ip.contract_id, &contract);
                }
                _ => {}
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
