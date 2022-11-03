#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;

use frame_support::{
    dispatch::DispatchErrorWithPostInfo,
    ensure,
    log::info,
    pallet_prelude::DispatchResult,
    traits::{
        Currency, EnsureOrigin, ExistenceRequirement, ExistenceRequirement::KeepAlive, Get,
        LockableCurrency, OnUnbalanced, WithdrawReasons,
    },
    transactional,
    weights::Pays,
    BoundedVec,
};
use frame_system::{
    self as system, ensure_signed,
    offchain::{AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer},
};
pub use pallet::*;
use pallet_tfgrid;
use pallet_tfgrid::pallet::{InterfaceOf, LocationOf, PubConfigOf, SerialNumberOf, TfgridNode};
use pallet_tfgrid::types as pallet_tfgrid_types;
use pallet_timestamp as timestamp;
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
    traits::{CheckedSub, SaturatedConversion},
    Perbill,
};
use substrate_fixed::types::U64F64;

use tfchain_support::{
    traits::{ChangeNode, Tfgrid},
    types::{CapacityReservationPolicy, ConsumableResources, Node, NodeFeatures, Resources},
};

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"smct");

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod test_utils;

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
pub mod migration;
pub mod name_contract;
pub mod types;
pub mod weights;
pub mod migrations;

#[frame_support::pallet]
pub mod pallet {

    use super::types::*;
    use super::weights::WeightInfo;
    use super::*;
    use codec::FullCodec;
    use frame_support::pallet_prelude::*;
    use frame_support::traits::Hooks;
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        traits::{Currency, Get, LockIdentifier, LockableCurrency, OnUnbalanced},
    };
    use frame_system::pallet_prelude::*;
    use sp_core::H256;
    use sp_std::{
        convert::{TryFrom, TryInto},
        fmt::Debug,
        vec::Vec,
    };
    use tfchain_support::traits::ChangeNode;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;
    pub type NegativeImbalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::NegativeImbalance;

    pub const GRID_LOCK_ID: LockIdentifier = *b"gridlock";

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // Version constant that referenced the struct version
    pub const CONTRACT_VERSION: u32 = 4;

    pub type MaxNodeContractPublicIPs<T> = <T as Config>::MaxNodeContractPublicIps;
    pub type MaxDeploymentDataLength<T> = <T as Config>::MaxDeploymentDataLength;
    pub type DeploymentDataInput<T> = BoundedVec<u8, MaxDeploymentDataLength<T>>;
    pub type DeploymentHash = H256;
    pub type NameContractNameOf<T> = <T as Config>::NameContractName;
    pub type ContractPublicIP<T> =
        PublicIP<<T as pallet_tfgrid::Config>::PublicIP, <T as pallet_tfgrid::Config>::GatewayIP>;

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
    pub type ContractIDByNodeIDAndHash<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u32,
        Blake2_128Concat,
        DeploymentHash,
        u64,
        ValueQuery,
    >;

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
    #[pallet::getter(fn group_id)]
    pub type GroupID<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn groups)]
    pub type Groups<T: Config> = StorageMap<_, Blake2_128Concat, u32, Group, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn capacity_reservation_id_by_node_group_config)]
    pub type CapacityReservationIDByNodeGroupConfig<T: Config> =
        StorageMap<_, Blake2_128Concat, types::NodeGroupConfig, u64, ValueQuery>;

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

    #[pallet::config]
    pub trait Config:
        CreateSignedTransaction<Call<Self>>
        + frame_system::Config
        + pallet_timestamp::Config
        + pallet_balances::Config
        + pallet_tfgrid::Config
        + pallet_tft_price::Config
    {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: LockableCurrency<Self::AccountId>;
        /// Handler for the unbalanced decrement when slashing (burning collateral)
        type Burn: OnUnbalanced<NegativeImbalanceOf<Self>>;
        type StakingPoolAccount: Get<Self::AccountId>;
        type BillingFrequency: Get<u64>;
        type DistributionFrequency: Get<u16>;
        type GracePeriod: Get<u64>;
        type WeightInfo: WeightInfo;
        type NodeChanged: ChangeNode<
            LocationOf<Self>,
            PubConfigOf<Self>,
            InterfaceOf<Self>,
            SerialNumberOf<Self>,
        >;
        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
        type Call: From<Call<Self>>;
        type Tfgrid: Tfgrid<Self::AccountId, FarmName<Self>, ContractPublicIP<Self>>;

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
            + MaxEncodedLen;

        type RestrictedOrigin: EnsureOrigin<Self::Origin>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A contract got created
        ContractCreated(types::Contract<T>),
        /// A contract was updated
        ContractUpdated(types::Contract<T>),
        /// deprecated event
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
            public_ips: BoundedVec<ContractPublicIP<T>, MaxNodeContractPublicIPs<T>>,
        },
        /// IP got freed by a Node contract
        IPsFreed {
            contract_id: u64,
            // public ip as a string
            public_ips: BoundedVec<ContractPublicIP<T>, MaxNodeContractPublicIPs<T>>,
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
        /// Deprecated event
        UpdatedUsedResources(types::ContractResources),
        /// Network resources report received for contract
        NruConsumptionReportReceived(types::NruConsumption),
        /// Deprecated event
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
        GroupCreated {
            group_id: u32,
            twin_id: u32,
        },
        GroupDeleted {
            group_id: u32,
        },
        CapacityReservationContractCanceled {
            contract_id: u64,
            node_id: u32,
            twin_id: u32,
        },
        DeploymentContractCanceled {
            contract_id: u64,
            capacity_reservation_contract_id: u64,
            twin_id: u32,
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
        NameExists,
        NameNotValid,
        InvalidContractType,
        TFTPriceValueError,
        NotEnoughResourcesOnNode,
        NodeNotAuthorizedToReportResources,
        MethodIsDeprecated,
        NodeHasActiveContracts,
        NodeHasRentContract,
        NodeIsNotDedicated,
        NodeNotAvailableToDeploy,
        CannotUpdateContractInGraceState,
        NumOverflow,
        OffchainSignedTxError,
        NameContractNameTooShort,
        NameContractNameTooLong,
        InvalidProviderConfiguration,
        NoSuchSolutionProvider,
        SolutionProviderNotApproved,
        NoSuitableNodeInFarm,
        NotAuthorizedToCreateDeploymentContract,
        GroupNotExists,
        TwinNotAuthorizedToDeleteGroup,
        GroupHasActiveMembers,
        CapacityReservationNotExists,
        CapacityReservationHasActiveContracts,
        ResourcesUsedByActiveContracts,
        NotEnoughResourcesInCapacityReservation,
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
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_group(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_create_group(account_id)
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn delete_group(origin: OriginFor<T>, group_id: u32) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_delete_group(account_id, group_id)
        }

        // DEPRECATED
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_node_contract(
            _origin: OriginFor<T>,
            _node_id: u32,
            _deployment_hash: DeploymentHash,
            _deployment_data: DeploymentDataInput<T>,
            _public_ips: u32,
            _solution_provider_id: Option<u64>,
        ) -> DispatchResultWithPostInfo {
            Err(DispatchErrorWithPostInfo::from(Error::<T>::MethodIsDeprecated).into())
        }

        // DEPRECATED
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn update_node_contract(
            _origin: OriginFor<T>,
            _contract_id: u64,
            _deployment_hash: DeploymentHash,
            _deployment_data: DeploymentDataInput<T>,
        ) -> DispatchResultWithPostInfo {
            Err(DispatchErrorWithPostInfo::from(Error::<T>::MethodIsDeprecated).into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn cancel_contract(
            origin: OriginFor<T>,
            contract_id: u64,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_cancel_contract(account_id, contract_id, types::Cause::CanceledByUser)
        }

        // DEPRECATED
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn add_reports(
            _origin: OriginFor<T>,
            _reports: Vec<types::Consumption>,
        ) -> DispatchResultWithPostInfo {
            // return error
            Err(DispatchErrorWithPostInfo::from(Error::<T>::MethodIsDeprecated).into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_name_contract(
            origin: OriginFor<T>,
            name: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_create_name_contract(account_id, name)
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_capacity_reservation_contract(
            origin: OriginFor<T>,
            farm_id: u32,
            policy: CapacityReservationPolicy,
            solution_provider_id: Option<u64>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_create_capacity_reservation_contract(
                account_id,
                farm_id,
                policy,
                solution_provider_id,
            )
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn update_capacity_reservation_contract(
            origin: OriginFor<T>,
            capacity_reservation_id: u64,
            resources: Resources,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_update_capacity_reservation_contract(
                account_id,
                capacity_reservation_id,
                resources,
            )
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_deployment_contract(
            origin: OriginFor<T>,
            capacity_reservation_id: u64,
            deployment_hash: DeploymentHash,
            deployment_data: DeploymentDataInput<T>,
            resources: Resources,
            public_ips: u32,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_create_deployment_contract(
                account_id,
                capacity_reservation_id,
                deployment_hash,
                deployment_data,
                resources,
                public_ips,
            )
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn update_deployment_contract(
            origin: OriginFor<T>,
            contract_id: u64,
            deployment_hash: DeploymentHash,
            deployment_data: DeploymentDataInput<T>,
            resources: Option<Resources>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_update_deployment_contract(
                account_id,
                contract_id,
                deployment_hash,
                deployment_data,
                resources,
            )
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn add_nru_reports(
            origin: OriginFor<T>,
            reports: Vec<types::NruConsumption>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_compute_reports(account_id, reports)
        }

        // DEPRECATED
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn report_contract_resources(
            _origin: OriginFor<T>,
            _contract_resources: Vec<types::ContractResources>,
        ) -> DispatchResultWithPostInfo {
            Err(DispatchErrorWithPostInfo::from(Error::<T>::MethodIsDeprecated).into())
        }

        // DEPRECATED
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_rent_contract(
            _origin: OriginFor<T>,
            _node_id: u32,
            _solution_provider_id: Option<u64>,
        ) -> DispatchResultWithPostInfo {
            // return error
            Err(DispatchErrorWithPostInfo::from(Error::<T>::MethodIsDeprecated).into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_solution_provider(
            origin: OriginFor<T>,
            description: Vec<u8>,
            link: Vec<u8>,
            providers: Vec<types::Provider<T::AccountId>>,
        ) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;
            Self::_create_solution_provider(description, link, providers)
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn approve_solution_provider(
            origin: OriginFor<T>,
            solution_provider_id: u64,
            approve: bool,
        ) -> DispatchResultWithPostInfo {
            <T as Config>::RestrictedOrigin::ensure_origin(origin)?;
            Self::_approve_solution_provider(solution_provider_id, approve)
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn bill_contract_for_block(
            origin: OriginFor<T>,
            contract_id: u64,
        ) -> DispatchResultWithPostInfo {
            let _account_id = ensure_signed(origin)?;
            Self::bill_contract(contract_id)
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(block_number: T::BlockNumber) {
            // Let offchain worker check if there are contracts on the map at current index
            // Index being current block number % (mod) Billing Frequency
            let current_index: u64 =
                block_number.saturated_into::<u64>() % BillingFrequency::<T>::get();

            let contracts = ContractsToBillAt::<T>::get(current_index);
            if contracts.is_empty() {
                log::info!(
                    "No contracts to bill at block {:?}, index: {:?}",
                    block_number,
                    current_index
                );
                return;
            }

            log::info!(
                "{:?} contracts to bill at block {:?}",
                contracts,
                block_number
            );

            for contract_id in contracts {
                let _res = Self::bill_contract_using_signed_transaction(contract_id);
            }
        }
    }
}

use frame_support::pallet_prelude::DispatchResultWithPostInfo;
use pallet::NameContractNameOf;
// use pallet_tfgrid::pub_ip::{GatewayIP, PublicIP as PalletTfgridPublicIP};
use sp_std::convert::{TryFrom, TryInto};
use tfchain_support::types::PublicIP;
// Internal functions of the pallet
impl<T: Config> Pallet<T> {
    pub fn _create_group(account_id: T::AccountId) -> DispatchResultWithPostInfo {
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(Error::<T>::TwinNotExists)?;

        let mut id = GroupID::<T>::get();
        id = id + 1;

        let new_group = types::Group {
            id: id,
            twin_id: twin_id,
            capacity_reservation_contract_ids: vec![],
        };

        Groups::<T>::insert(id, &new_group);

        Self::deposit_event(Event::GroupCreated {
            group_id: id,
            twin_id: twin_id,
        });

        Ok(().into())
    }

    pub fn _delete_group(account_id: T::AccountId, group_id: u32) -> DispatchResultWithPostInfo {
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(Error::<T>::TwinNotExists)?;

        let group = Groups::<T>::get(group_id).ok_or(Error::<T>::GroupNotExists)?;

        ensure!(
            twin_id == group.twin_id,
            Error::<T>::TwinNotAuthorizedToDeleteGroup
        );
        ensure!(
            group.capacity_reservation_contract_ids.is_empty(),
            Error::<T>::GroupHasActiveMembers
        );

        Groups::<T>::remove(group_id);

        Self::deposit_event(Event::GroupDeleted { group_id: group_id });

        Ok(().into())
    }

    pub fn _add_capacity_reservation_contract_to_group(
        group_id: u32,
        capacity_rservation_id: u64,
        node_id: u32,
    ) -> DispatchResultWithPostInfo {
        let mut group = Groups::<T>::get(group_id).ok_or(Error::<T>::GroupNotExists)?;
        group
            .capacity_reservation_contract_ids
            .push(capacity_rservation_id);
        Groups::<T>::insert(group_id, &group);

        CapacityReservationIDByNodeGroupConfig::<T>::insert(
            types::NodeGroupConfig {
                group_id: group_id,
                node_id: node_id,
            },
            capacity_rservation_id,
        );

        Ok(().into())
    }

    pub fn _remove_capacity_reservation_contract_from_group(
        group_id: u32,
        capacity_reservation_id: u64,
        node_id: u32,
    ) -> DispatchResultWithPostInfo {
        let mut group = Groups::<T>::get(group_id).ok_or(Error::<T>::GroupNotExists)?;
        group
            .capacity_reservation_contract_ids
            .retain(|&id| id != capacity_reservation_id);
        Groups::<T>::insert(group_id, &group);

        CapacityReservationIDByNodeGroupConfig::<T>::remove(types::NodeGroupConfig {
            node_id: node_id,
            group_id: group_id,
        });

        Ok(().into())
    }

    #[transactional]
    pub fn _create_capacity_reservation_contract(
        account_id: T::AccountId,
        farm_id: u32,
        policy: CapacityReservationPolicy,
        solution_provider_id: Option<u64>,
    ) -> DispatchResultWithPostInfo {
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(Error::<T>::TwinNotExists)?;
        ensure!(
            pallet_tfgrid::Farms::<T>::contains_key(farm_id),
            Error::<T>::FarmNotExists
        );

        let (node_id, resources, group_id) = match policy {
            CapacityReservationPolicy::Any {
                resources,
                features,
            } => {
                let farm =
                    pallet_tfgrid::Farms::<T>::get(farm_id).ok_or(Error::<T>::FarmNotExists)?;
                ensure!(!farm.dedicated_farm, Error::<T>::NodeNotAvailableToDeploy);
                let n_id = Self::_find_suitable_node_in_farm(farm_id, resources, None, features)?;
                (n_id, resources, None)
            }
            CapacityReservationPolicy::Exclusive {
                group_id,
                resources,
                features,
            } => {
                let farm =
                    pallet_tfgrid::Farms::<T>::get(farm_id).ok_or(Error::<T>::FarmNotExists)?;
                ensure!(!farm.dedicated_farm, Error::<T>::NodeNotAvailableToDeploy);
                let n_id = Self::_find_suitable_node_in_farm(
                    farm_id,
                    resources,
                    Some(group_id),
                    features,
                )?;
                (n_id, resources, Some(group_id))
            }
            CapacityReservationPolicy::Node { node_id } => {
                let node =
                    pallet_tfgrid::Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;
                ensure!(
                    node.resources.used_resources == Resources::empty(),
                    Error::<T>::NodeNotAvailableToDeploy
                );
                (node_id, node.resources.total_resources, None)
            }
        };

        let new_capacity_reservation = types::CapacityReservationContract {
            node_id: node_id,
            resources: ConsumableResources {
                total_resources: resources,
                used_resources: Resources::empty(),
            },
            group_id: group_id,
            public_ips: 0,
            deployment_contracts: vec![],
        };

        // Create contract
        let contract = Self::_create_contract(
            twin_id,
            types::ContractData::CapacityReservationContract(new_capacity_reservation.clone()),
            solution_provider_id,
        )?;

        // insert billing information
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

        // insert NodeGroup configuration
        if let Some(group_id) = group_id {
            Self::_add_capacity_reservation_contract_to_group(
                group_id,
                contract.contract_id,
                node_id,
            )?;
        }

        // Insert contract into active contracts map
        let mut capacity_reservation_contracts =
            ActiveNodeContracts::<T>::get(&new_capacity_reservation.node_id);
        capacity_reservation_contracts.push(contract.contract_id);
        ActiveNodeContracts::<T>::insert(
            &new_capacity_reservation.node_id,
            &capacity_reservation_contracts,
        );

        Self::deposit_event(Event::ContractCreated(contract));

        Ok(().into())
    }

    #[transactional]
    pub fn _update_capacity_reservation_contract(
        account_id: T::AccountId,
        capacity_reservation_id: u64,
        resources: Resources,
    ) -> DispatchResultWithPostInfo {
        let mut contract = Contracts::<T>::get(capacity_reservation_id)
            .ok_or(Error::<T>::CapacityReservationNotExists)?;
        let twin =
            pallet_tfgrid::Twins::<T>::get(contract.twin_id).ok_or(Error::<T>::TwinNotExists)?;
        ensure!(
            twin.account_id == account_id,
            Error::<T>::TwinNotAuthorizedToUpdateContract
        );

        // Don't allow updates for contracts that are in grace state
        ensure!(
            !matches!(contract.state, types::ContractState::GracePeriod(_)),
            Error::<T>::CannotUpdateContractInGraceState
        );

        let mut capacity_reservation_contract = Self::get_capacity_reservation_contract(&contract)?;

        let resources_increase = capacity_reservation_contract
            .resources
            .calculate_increase_in_resources(&resources);
        let resources_reduction = capacity_reservation_contract
            .resources
            .calculate_reduction_in_resources(&resources);

        // we can only reduce as much as we have free resources in our reservation
        ensure!(
            capacity_reservation_contract
                .resources
                .can_consume_resources(&resources_reduction),
            Error::<T>::ResourcesUsedByActiveContracts
        );
        if !resources_increase.is_empty() {
            Self::_claim_resources_on_node(
                capacity_reservation_contract.node_id,
                resources_increase,
            )?;
        }
        if !resources_reduction.is_empty() {
            Self::_unclaim_resources_on_node(
                capacity_reservation_contract.node_id,
                resources_reduction,
            )?;
        }

        capacity_reservation_contract.resources.total_resources = resources;

        contract.contract_type =
            types::ContractData::CapacityReservationContract(capacity_reservation_contract);

        contract.state = contract.state.clone();
        Contracts::<T>::insert(&contract.contract_id, contract.clone());

        Self::deposit_event(Event::ContractUpdated(contract));

        Ok(().into())
    }

    #[transactional]
    pub fn _create_deployment_contract(
        account_id: T::AccountId,
        capacity_reservation_id: u64,
        deployment_hash: DeploymentHash,
        deployment_data: DeploymentDataInput<T>,
        resources: Resources,
        public_ips: u32,
    ) -> DispatchResultWithPostInfo {
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(Error::<T>::TwinNotExists)?;

        let cr_contract = Contracts::<T>::get(capacity_reservation_id)
            .ok_or(Error::<T>::CapacityReservationNotExists)?;
        let capacity_reservation_contract = Self::get_capacity_reservation_contract(&cr_contract)?;
        ensure!(
            cr_contract.twin_id == twin_id,
            Error::<T>::NotAuthorizedToCreateDeploymentContract
        );

        // If the contract with hash and node id exists and it's in any other state then
        // contractState::Deleted then we don't allow the creation of it.
        // If it exists we allow the user to "restore" this contract
        if ContractIDByNodeIDAndHash::<T>::contains_key(
            capacity_reservation_contract.node_id,
            &deployment_hash,
        ) {
            let contract_id = ContractIDByNodeIDAndHash::<T>::get(
                capacity_reservation_contract.node_id,
                &deployment_hash,
            );
            let contract = Contracts::<T>::get(contract_id).ok_or(Error::<T>::ContractNotExists)?;
            if !contract.is_state_delete() {
                return Err(Error::<T>::ContractIsNotUnique.into());
            }
        }

        let public_ips_list: BoundedVec<
            PublicIP<
                <T as pallet_tfgrid::Config>::PublicIP,
                <T as pallet_tfgrid::Config>::GatewayIP,
            >,
            MaxNodeContractPublicIPs<T>,
        > = vec![].try_into().unwrap();
        // Prepare DeploymentContract struct
        let deployment_contract = types::DeploymentContract {
            capacity_reservation_id: capacity_reservation_id,
            deployment_hash: deployment_hash.clone(),
            deployment_data: deployment_data,
            public_ips: public_ips,
            public_ips_list: public_ips_list,
            resources: resources,
        };

        // Create contract
        let contract = Self::_create_contract(
            twin_id,
            types::ContractData::DeploymentContract(deployment_contract.clone()),
            None,
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
        ContractIDByNodeIDAndHash::<T>::insert(
            capacity_reservation_contract.node_id,
            deployment_hash,
            contract.contract_id,
        );

        Self::deposit_event(Event::ContractCreated(contract));

        Ok(().into())
    }

    pub fn _get_node_id_from_contract(contract_id: u64) -> Result<u32, DispatchErrorWithPostInfo> {
        let contract = Contracts::<T>::get(contract_id).ok_or(Error::<T>::ContractNotExists)?;
        let node_id = match contract.contract_type {
            types::ContractData::CapacityReservationContract(ref capacity_reservation_contract) => {
                capacity_reservation_contract.node_id
            }
            types::ContractData::DeploymentContract(ref deployment_contract) => {
                Self::_get_node_id_from_contract(deployment_contract.capacity_reservation_id)?
            }
            _ => 0,
        };
        ensure!(node_id != 0, Error::<T>::InvalidContractType);

        Ok(node_id.into())
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

    fn _claim_resources_on_node(node_id: u32, resources: Resources) -> DispatchResultWithPostInfo {
        let mut node = pallet_tfgrid::Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;

        ensure!(
            node.resources.can_consume_resources(&resources),
            Error::<T>::NotEnoughResourcesOnNode
        );
        //update the available resources
        node.resources.consume(&resources);

        pallet_tfgrid::Nodes::<T>::insert(node.id, &node);

        T::Tfgrid::node_resources_changed(node.id);

        Ok(().into())
    }

    fn _unclaim_resources_on_node(
        node_id: u32,
        resources: Resources,
    ) -> DispatchResultWithPostInfo {
        let mut node = pallet_tfgrid::Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;

        //update the available resources
        node.resources.free(&resources);

        pallet_tfgrid::Nodes::<T>::insert(node.id, &node);

        T::Tfgrid::node_resources_changed(node.id);

        Ok(().into())
    }

    fn _claim_resources_on_capacity_reservation(
        capacity_reservation_id: u64,
        resources: Resources,
    ) -> DispatchResultWithPostInfo {
        let mut contract = Contracts::<T>::get(capacity_reservation_id)
            .ok_or(Error::<T>::CapacityReservationNotExists)?;
        let mut capacity_reservation_contract = Self::get_capacity_reservation_contract(&contract)?;

        ensure!(
            capacity_reservation_contract
                .resources
                .can_consume_resources(&resources),
            Error::<T>::NotEnoughResourcesInCapacityReservation
        );

        //update the available resources
        capacity_reservation_contract.resources.consume(&resources);

        contract.contract_type =
            types::ContractData::CapacityReservationContract(capacity_reservation_contract);

        Contracts::<T>::insert(capacity_reservation_id, contract);

        Ok(().into())
    }

    fn _unclaim_resources_on_capacity_reservation(
        capacity_reservation_id: u64,
        resources: Resources,
    ) -> DispatchResultWithPostInfo {
        let mut contract = Contracts::<T>::get(capacity_reservation_id)
            .ok_or(Error::<T>::CapacityReservationNotExists)?;
        let mut capacity_reservation_contract = Self::get_capacity_reservation_contract(&contract)?;

        //update the available resources
        capacity_reservation_contract.resources.free(&resources);

        contract.contract_type =
            types::ContractData::CapacityReservationContract(capacity_reservation_contract);

        Contracts::<T>::insert(capacity_reservation_id, contract);

        Ok(().into())
    }

    fn _add_deployment_contract_to_capacity_reservation_contract(
        capacity_reservation_id: u64,
        deployment_contract_id: u64,
    ) -> DispatchResultWithPostInfo {
        let mut contract = Contracts::<T>::get(capacity_reservation_id)
            .ok_or(Error::<T>::CapacityReservationNotExists)?;
        let mut capacity_reservation_contract = Self::get_capacity_reservation_contract(&contract)?;

        if !capacity_reservation_contract
            .deployment_contracts
            .contains(&deployment_contract_id)
        {
            //update the available resources
            capacity_reservation_contract
                .deployment_contracts
                .push(deployment_contract_id);

            contract.contract_type =
                types::ContractData::CapacityReservationContract(capacity_reservation_contract);

            Contracts::<T>::insert(capacity_reservation_id, contract);
        }

        Ok(().into())
    }

    fn _remove_deployment_contract_from_capacity_reservation_contract(
        capacity_reservation_id: u64,
        deployment_contract_id: u64,
    ) -> DispatchResultWithPostInfo {
        let mut contract = Contracts::<T>::get(capacity_reservation_id)
            .ok_or(Error::<T>::CapacityReservationNotExists)?;
        let mut capacity_reservation_contract = Self::get_capacity_reservation_contract(&contract)?;

        if capacity_reservation_contract
            .deployment_contracts
            .contains(&deployment_contract_id)
        {
            //update the available resources
            capacity_reservation_contract
                .deployment_contracts
                .retain(|&id| id != deployment_contract_id);

            contract.contract_type =
                types::ContractData::CapacityReservationContract(capacity_reservation_contract);

            Contracts::<T>::insert(capacity_reservation_id, contract);
        }

        Ok(().into())
    }

    fn _find_suitable_node_in_farm(
        farm_id: u32,
        resources: Resources,
        group_id: Option<u32>,
        features: Option<Vec<NodeFeatures>>,
    ) -> Result<u32, DispatchErrorWithPostInfo> {
        ensure!(
            pallet_tfgrid::Farms::<T>::contains_key(farm_id),
            Error::<T>::FarmNotExists
        );
        let nodes_in_farm = pallet_tfgrid::NodesByFarmID::<T>::get(farm_id);
        let mut suitable_nodes = Vec::new();
        for node_id in nodes_in_farm {
            let node = pallet_tfgrid::Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;
            suitable_nodes.push(node);
        }

        // only keep nodes with enough resources
        suitable_nodes.retain_mut(|node| node.resources.can_consume_resources(&resources));

        // only keep nodes that DON'T have a contract configured on them that belong to the same group
        if let Some(g_id) = group_id {
            ensure!(Groups::<T>::contains_key(g_id), Error::<T>::GroupNotExists);
            suitable_nodes.retain_mut(|node| {
                !CapacityReservationIDByNodeGroupConfig::<T>::contains_key(types::NodeGroupConfig {
                    node_id: node.id,
                    group_id: g_id,
                })
            });
        }

        // only keep nodes with the required features
        if let Some(features) = features {
            for feature in features {
                match feature {
                    NodeFeatures::PublicNode => {
                        suitable_nodes.retain_mut(|node| node.public_config.is_some())
                    }
                }
            }
        }

        ensure!(!suitable_nodes.is_empty(), Error::<T>::NoSuitableNodeInFarm);

        // sort the suitable nodes on power state: the nodes that are Up will be first in the list
        suitable_nodes.sort_by(|a, b| a.power.state.cmp(&b.power.state));

        Ok(suitable_nodes[0].id)
    }

    fn _create_contract(
        twin_id: u32,
        mut contract_type: types::ContractData<T>,
        solution_provider_id: Option<u64>,
    ) -> Result<types::Contract<T>, DispatchErrorWithPostInfo> {
        // Get the Contract ID map and increment
        let mut id = ContractID::<T>::get();
        id = id + 1;

        match contract_type {
            types::ContractData::DeploymentContract(ref mut c) => {
                Self::_reserve_ip(id, c)?;
                Self::_claim_resources_on_capacity_reservation(
                    c.capacity_reservation_id,
                    c.resources,
                )?;
                Self::_add_deployment_contract_to_capacity_reservation_contract(
                    c.capacity_reservation_id,
                    id,
                )?;
            }
            types::ContractData::CapacityReservationContract(ref mut c) => {
                Self::_claim_resources_on_node(c.node_id, c.resources.total_resources)?;
            }
            _ => {}
        }

        Self::validate_solution_provider(solution_provider_id)?;

        let contract = types::Contract {
            version: CONTRACT_VERSION,
            twin_id,
            contract_id: id,
            state: types::ContractState::Created,
            contract_type: contract_type.clone(),
            solution_provider_id,
        };

        // Start billing frequency loop
        // Will always be block now + frequency
        if !matches!(contract_type, types::ContractData::DeploymentContract(_)) {
            Self::insert_contract_to_bill(id);
        }

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

    #[transactional]
    pub fn _update_deployment_contract(
        account_id: T::AccountId,
        contract_id: u64,
        deployment_hash: DeploymentHash,
        deployment_data: DeploymentDataInput<T>,
        resources: Option<Resources>,
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

        let mut deployment_contract = Self::get_deployment_contract(&contract.clone())?;
        let cr_contract = Contracts::<T>::get(deployment_contract.capacity_reservation_id)
            .ok_or(Error::<T>::ContractNotExists)?;
        let capacity_reservation_contract = Self::get_capacity_reservation_contract(&cr_contract)?;

        // remove and reinsert contract id by node id and hash because that hash can have changed
        ContractIDByNodeIDAndHash::<T>::remove(
            capacity_reservation_contract.node_id,
            deployment_contract.deployment_hash,
        );
        ContractIDByNodeIDAndHash::<T>::insert(
            capacity_reservation_contract.node_id,
            &deployment_hash,
            contract_id,
        );

        deployment_contract.deployment_hash = deployment_hash;
        deployment_contract.deployment_data = deployment_data;

        // update the resources with the extra resources
        if let Some(resources) = resources {
            if deployment_contract.resources != resources {
                // first unclaim all resources from deployment contract
                Self::_unclaim_resources_on_capacity_reservation(
                    cr_contract.contract_id,
                    deployment_contract.resources,
                )?;
                // then claim the new required resources
                Self::_claim_resources_on_capacity_reservation(cr_contract.contract_id, resources)?;
                deployment_contract.resources = resources;
            }
        }

        // override values
        contract.contract_type = types::ContractData::DeploymentContract(deployment_contract);
        contract.state = contract.state.clone();
        Contracts::<T>::insert(&contract.contract_id, contract.clone());

        Self::deposit_event(Event::ContractUpdated(contract));

        Ok(().into())
    }

    #[transactional]
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

        // If it's a capacity reservation contract and it still has active workloads, don't allow cancellation.
        match contract.contract_type {
            types::ContractData::CapacityReservationContract(ref capacity_reservation_contract) => {
                ensure!(
                    capacity_reservation_contract.deployment_contracts.len() == 0,
                    Error::<T>::CapacityReservationHasActiveContracts
                );
            }
            _ => {}
        }
        contract = Contracts::<T>::get(contract_id).ok_or(Error::<T>::ContractNotExists)?;
        contract.state = types::ContractState::Deleted(cause);
        Contracts::<T>::insert(&contract.contract_id, contract.clone());
        Self::bill_contract(contract.contract_id)?;
        // Remove all associated storage
        Self::remove_contract(contract.contract_id);

        Ok(().into())
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
            let deployment_contract = Self::get_deployment_contract(&contract)?;
            let contract_cr = Contracts::<T>::get(deployment_contract.capacity_reservation_id)
                .ok_or(Error::<T>::CapacityReservationNotExists)?;
            let capacity_reservation_contract =
                Self::get_capacity_reservation_contract(&contract_cr)?;
            ensure!(
                capacity_reservation_contract.node_id == node_id,
                Error::<T>::NodeNotAuthorizedToComputeReport
            );

            Self::_calculate_report_cost(contract_cr.contract_id, &report, &pricing_policy);
            Self::deposit_event(Event::NruConsumptionReportReceived(report.clone()));
        }

        Ok(Pays::No.into())
    }

    // Calculates the total cost of a report.
    // Takes in a report for NRU (network resource units)
    // Updates the contract's billing information in storage
    pub fn _calculate_report_cost(
        contract_id: u64,
        report: &types::NruConsumption,
        pricing_policy: &pallet_tfgrid_types::PricingPolicy<T::AccountId>,
    ) {
        let mut contract_billing_info = ContractBillingInformationByID::<T>::get(contract_id);
        if report.timestamp < contract_billing_info.last_updated {
            return;
        }

        // seconds elapsed is the report.window
        let seconds_elapsed = report.window;
        log::info!("seconds elapsed: {:?}", seconds_elapsed);

        // calculate NRU used and the cost
        let used_nru = U64F64::from_num(report.nru) / pricing_policy.nu.factor_base_1000();
        let nu_cost = used_nru
            * (U64F64::from_num(pricing_policy.nu.value) / 3600)
            * U64F64::from_num(seconds_elapsed);
        log::info!("nu cost: {:?}", nu_cost);

        // save total
        let total = nu_cost.ceil().to_num::<u64>();
        log::info!("total cost: {:?}", total);

        // update contract billing info
        contract_billing_info.amount_unbilled += total;
        contract_billing_info.last_updated = report.timestamp;
        ContractBillingInformationByID::<T>::insert(contract_id, &contract_billing_info);
    }

    fn bill_contract_using_signed_transaction(contract_id: u64) -> Result<(), Error<T>> {
        let signer = Signer::<T, <T as pallet::Config>::AuthorityId>::any_account();
        if !signer.can_sign() {
            log::error!(
                "failed billing contract {:?} account cannot be used to sign transaction",
                contract_id,
            );
            return Err(<Error<T>>::OffchainSignedTxError);
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
                return Err(<Error<T>>::OffchainSignedTxError);
            }
            return Ok(());
        }
        log::error!("No local account available");
        return Err(<Error<T>>::OffchainSignedTxError);
    }

    fn bill_contract_using_signed_transaction(contract_id: u64) -> Result<(), Error<T>> {
        let signer = Signer::<T, <T as pallet::Config>::AuthorityId>::any_account();
        if !signer.can_sign() {
            log::error!(
                "failed billing contract {:?} account cannot be used to sign transaction",
                contract_id,
            );
            return Err(<Error<T>>::OffchainSignedTxError);
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
                return Err(<Error<T>>::OffchainSignedTxError);
            }
            return Ok(());
        }
        log::error!("No local account available");
        return Err(<Error<T>>::OffchainSignedTxError);
    }

    // Calculates how much TFT is due by the user and distributes the rewards
    fn bill_contract(contract_id: u64) -> DispatchResultWithPostInfo {
        // Clean up contract from blling loop if it not exists anymore
        if !Contracts::<T>::contains_key(contract_id) {
            log::debug!("cleaning up deleted contract from storage");

            let index = Self::get_contract_index();

            // Remove contract from billing list
            let mut contracts = ContractsToBillAt::<T>::get(index);
            contracts.retain(|&c| c != contract_id);
            ContractsToBillAt::<T>::insert(index, contracts);

            return Ok(().into());
        }

        let mut contract = Contracts::<T>::get(contract_id).ok_or(Error::<T>::ContractNotExists)?;

        let twin =
            pallet_tfgrid::Twins::<T>::get(contract.twin_id).ok_or(Error::<T>::TwinNotExists)?;
        let usable_balance = Self::get_usable_balance(&twin.account_id);

        let mut seconds_elapsed = BillingFrequency::<T>::get() * 6;
        // Calculate amount of seconds elapsed based on the contract lock struct

        let now = <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;
        // this will set the seconds elapsed to the default billing cycle duration in seconds
        // if there is no contract lock object yet. A contract lock object will be created later in this function
        // https://github.com/threefoldtech/tfchain/issues/261
        let contract_lock = ContractLock::<T>::get(contract.contract_id);
        if contract_lock.lock_updated != 0 {
            seconds_elapsed = now.checked_sub(contract_lock.lock_updated).unwrap_or(0);
        }

        let (amount_due, discount_received) =
            contract.calculate_contract_cost_tft(usable_balance, seconds_elapsed)?;

        // If there is nothing to be paid and the contract is not in state delete, return
        // Can be that the users cancels the contract in the same block that it's getting billed
        // where elapsed seconds would be 0, but we still have to distribute rewards
        if amount_due == BalanceOf::<T>::saturated_from(0 as u128) && !contract.is_state_delete() {
            log::debug!("amount to be billed is 0, nothing to do");
            return Ok(().into());
        };

        // Handle grace
        let contract = Self::handle_grace(&mut contract, usable_balance, amount_due)?;

        // If still in grace period, no need to continue doing locking and other stuff
        if matches!(contract.state, types::ContractState::GracePeriod(_)) {
            log::debug!("contract {} is still in grace", contract.contract_id);
            return Ok(().into());
        }

        // If still in grace period, no need to continue doing locking and other stuff
        if matches!(contract.state, types::ContractState::GracePeriod(_)) {
            log::debug!("contract {} is still in grace", contract.contract_id);
            return Ok(().into());
        }

        // Handle contract lock operations
        Self::handle_lock(contract, amount_due)?;

        // Always emit a contract billed event
        let contract_bill = types::ContractBill {
            contract_id: contract.contract_id,
            timestamp: <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000,
            discount_level: discount_received.clone(),
            amount_billed: amount_due.saturated_into::<u128>(),
        };
        Self::deposit_event(Event::ContractBilled(contract_bill));

        // set the amount unbilled back to 0
        let mut contract_billing_info =
            ContractBillingInformationByID::<T>::get(contract.contract_id);
        contract_billing_info.amount_unbilled = 0;
        ContractBillingInformationByID::<T>::insert(contract.contract_id, &contract_billing_info);

        // If the contract is in delete state, remove all associated storage
        if matches!(contract.state, types::ContractState::Deleted(_)) {
            Self::remove_contract(contract.contract_id);
        }

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
                    Self::handle_grace_capacity_reservation_contract(
                        contract,
                        types::ContractState::Created,
                    )?;
                } else {
                    let diff = current_block - grace_start;
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
                    Self::handle_grace_capacity_reservation_contract(
                        contract,
                        types::ContractState::GracePeriod(current_block),
                    )?;
                }
            }
            _ => (),
        }

        Ok(contract)
    }

    fn handle_grace_capacity_reservation_contract(
        contract: &mut types::Contract<T>,
        state: types::ContractState,
    ) -> DispatchResultWithPostInfo {
        match &contract.contract_type {
            types::ContractData::CapacityReservationContract(c) => {
                for &ctr_id in &c.deployment_contracts {
                    let mut ctr =
                        Contracts::<T>::get(ctr_id).ok_or(Error::<T>::ContractNotExists)?;
                    Self::_update_contract_state(&mut ctr, &state)?;

                    match state {
                        types::ContractState::Created => {
                            Self::deposit_event(Event::ContractGracePeriodEnded {
                                contract_id: ctr_id,
                                node_id: c.node_id,
                                twin_id: ctr.twin_id,
                            });
                        }
                        types::ContractState::GracePeriod(block_number) => {
                            Self::deposit_event(Event::ContractGracePeriodStarted {
                                contract_id: ctr_id,
                                node_id: c.node_id,
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
        amount_due: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
        let now = <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;

        let mut contract_lock = ContractLock::<T>::get(contract.contract_id);
        // Only update contract lock in state (Created, GracePeriod)
        if !matches!(contract.state, types::ContractState::Deleted(_)) {
            // increment cycles billed and update the internal lock struct
            contract_lock.lock_updated = now;
            contract_lock.cycles += 1;
            contract_lock.amount_locked = contract_lock.amount_locked + amount_due;
            ContractLock::<T>::insert(contract.contract_id, &contract_lock);
        }

        // Only lock an amount from the user's balance if the contract is in create state
        // The lock is specified on the user's account, since a user can have multiple contracts
        // Just extend the lock with the amount due for this contract billing period (lock will be created if not exists)
        let twin =
            pallet_tfgrid::Twins::<T>::get(contract.twin_id).ok_or(Error::<T>::TwinNotExists)?;
        if matches!(contract.state, types::ContractState::Created) {
            let mut locked_balance = Self::get_locked_balance(&twin.account_id);
            locked_balance += amount_due;
            <T as Config>::Currency::extend_lock(
                GRID_LOCK_ID,
                &twin.account_id,
                locked_balance,
                WithdrawReasons::all(),
            );
        }

        let canceled_and_not_zero =
            contract.is_state_delete() && contract_lock.amount_locked.saturated_into::<u64>() > 0;
        // When the cultivation rewards are ready to be distributed or it's in delete state
        // Unlock all reserved balance and distribute
        if contract_lock.cycles >= T::DistributionFrequency::get() || canceled_and_not_zero {
            // Deprecated locking system
            // If there is a lock with ID being the contract ID, remove it
            // Code can be removed in a later phase
            <T as Config>::Currency::remove_lock(
                contract.contract_id.to_be_bytes(),
                &twin.account_id,
            );

            // First remove the lock, calculate how much locked balance needs to be unlocked and re-lock the remaining locked balance
            let locked_balance = Self::get_locked_balance(&twin.account_id);
            let new_locked_balance = match locked_balance.checked_sub(&contract_lock.amount_locked)
            {
                Some(b) => b,
                None => BalanceOf::<T>::saturated_from(0 as u128),
            };
            <T as Config>::Currency::remove_lock(GRID_LOCK_ID, &twin.account_id);
            <T as Config>::Currency::set_lock(
                GRID_LOCK_ID,
                &twin.account_id,
                new_locked_balance,
                WithdrawReasons::all(),
            );

            // Fetch twin balance, if the amount locked in the contract lock exceeds the current unlocked
            // balance we can only transfer out the remaining balance
            // https://github.com/threefoldtech/tfchain/issues/479
            let twin_balance = Self::get_usable_balance(&twin.account_id);

            // Fetch the default pricing policy
            let pricing_policy = pallet_tfgrid::PricingPolicies::<T>::get(1)
                .ok_or(Error::<T>::PricingPolicyNotExists)?;
            // Distribute cultivation rewards
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
            // Reset values
            contract_lock.lock_updated = now;
            contract_lock.amount_locked = BalanceOf::<T>::saturated_from(0 as u128);
            contract_lock.cycles = 0;
            ContractLock::<T>::insert(contract.contract_id, &contract_lock);
        }

        Ok(().into())
    }

    pub fn remove_contract(contract_id: u64) {
        let contract = Contracts::<T>::get(contract_id);
        if contract.is_none() {
            return;
        }

        if let Some(contract) = contract {
            match contract.contract_type.clone() {
                types::ContractData::DeploymentContract(mut deployment_contract) => {
                    let node_id = match Self::_get_node_id_from_contract(contract_id) {
                        Ok(n_id) => n_id,
                        Err(e) => {
                            log::error!("error while getting the node id from the deployment contract: {:?}", e);
                            0
                        }
                    };
                    if node_id == 0 {
                        return;
                    }
                    // free the public ips requested by the contract
                    if deployment_contract.public_ips > 0 {
                        match Self::_free_ip(contract_id, &mut deployment_contract) {
                            Ok(_) => (),
                            Err(e) => {
                                log::error!("error while freeing ips: {:?}", e);
                            }
                        }
                    }
                    // unclaim all resources used by this contract on the capacity reservation contract
                    match Self::_unclaim_resources_on_capacity_reservation(
                        deployment_contract.capacity_reservation_id,
                        deployment_contract.resources,
                    ) {
                        Ok(_) => (),
                        Err(e) => {
                            log::error!(
                                "error while freeing resources from deployment contract: {:?}",
                                e
                            );
                        }
                    }
                    // remove the deployment contract from the capacity reservation contract
                    match Self::_remove_deployment_contract_from_capacity_reservation_contract(
                        deployment_contract.capacity_reservation_id,
                        contract_id,
                    ) {
                        Ok(_) => (),
                        Err(e) => {
                            log::error!(
                                "error while removing the deployment contract from the capacity reservation contract: {:?}", 
                                e
                            );
                        }
                    }
                    // remove the contract by hash from storage
                    ContractIDByNodeIDAndHash::<T>::remove(
                        node_id,
                        &deployment_contract.deployment_hash,
                    );
                    //NodeContractResources::<T>::remove(contract_id);
                    ContractBillingInformationByID::<T>::remove(contract_id);

                    Self::deposit_event(Event::DeploymentContractCanceled {
                        contract_id,
                        capacity_reservation_contract_id: deployment_contract
                            .capacity_reservation_id,
                        twin_id: contract.twin_id,
                    });
                }
                types::ContractData::NameContract(name_contract) => {
                    ContractIDByNameRegistration::<T>::remove(name_contract.name);
                    Self::deposit_event(Event::NameContractCanceled { contract_id });
                }
                types::ContractData::CapacityReservationContract(capacity_reservation_contract) => {
                    // first remove all deployment contracts created with that reserved capacity
                    for deployment_contract_id in capacity_reservation_contract.deployment_contracts
                    {
                        Self::remove_contract(deployment_contract_id);
                    }

                    // remove groups
                    if let Some(group_id) = capacity_reservation_contract.group_id {
                        match Self::_remove_capacity_reservation_contract_from_group(
                            group_id,
                            contract_id,
                            capacity_reservation_contract.node_id,
                        ) {
                            Ok(_) => (),
                            Err(e) => {
                                log::error!("error while removing the capacity reservation contract fromt its group: {:?}", e);
                            }
                        }
                    }

                    // unclaim the resources on the node
                    match Self::_unclaim_resources_on_node(
                        capacity_reservation_contract.node_id,
                        capacity_reservation_contract.resources.total_resources,
                    ) {
                        Ok(_) => (),
                        Err(e) => {
                            log::error!(
                                "error while freeing resources from capacity reservation contract: {:?}",
                                e
                            );
                        }
                    }
                    // remove the contract id from the active contracts on that node
                    Self::remove_active_node_contract(
                        capacity_reservation_contract.node_id,
                        contract_id,
                    );
                    Self::deposit_event(Event::CapacityReservationContractCanceled {
                        contract_id: contract_id,
                        node_id: capacity_reservation_contract.node_id,
                        twin_id: contract.twin_id,
                    });
                }
            };
            info!("removing contract");
            Contracts::<T>::remove(contract_id);
            ContractLock::<T>::remove(contract_id);
        }
    }

    // Following: https://library.threefold.me/info/threefold#/tfgrid/farming/threefold__proof_of_utilization
    fn _distribute_cultivation_rewards(
        contract: &types::Contract<T>,
        pricing_policy: &pallet_tfgrid_types::PricingPolicy<T::AccountId>,
        amount: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
        // fetch source twin
        let twin =
            pallet_tfgrid::Twins::<T>::get(contract.twin_id).ok_or(Error::<T>::TwinNotExists)?;

        // Send 10% to the foundation
        let foundation_share = Perbill::from_percent(10) * amount;
        log::info!(
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
        log::info!(
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
                        log::info!(
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
            log::info!(
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
        let mut amount_to_burn =
            (Perbill::from_percent(50) * amount) - foundation_share - staking_pool_share;

        let existential_deposit_requirement = <T as Config>::Currency::minimum_balance();
        let free_balance = <T as Config>::Currency::free_balance(&twin.account_id);
        if amount_to_burn > free_balance - existential_deposit_requirement {
            amount_to_burn = <T as Config>::Currency::free_balance(&twin.account_id)
                - existential_deposit_requirement;
        }

        let to_burn = T::Currency::withdraw(
            &twin.account_id,
            amount_to_burn,
            WithdrawReasons::FEE,
            ExistenceRequirement::KeepAlive,
        )?;

        log::info!(
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

    // Inserts a contract in a list where the index is the current block % billing frequency
    // This way, we don't need to reinsert the contract everytime it gets billed
    pub fn insert_contract_to_bill(contract_id: u64) {
        if contract_id == 0 {
            return;
        }

        // Save the contract to be billed in (now -1 %(mod) BILLING_FREQUENCY_IN_BLOCKS)
        let index = Self::get_contract_index().checked_sub(1).unwrap_or(0);
        let mut contracts = ContractsToBillAt::<T>::get(index);

        if !contracts.contains(&contract_id) {
            contracts.push(contract_id);
            ContractsToBillAt::<T>::insert(index, &contracts);
            log::info!(
                "Insert contracts: {:?}, to be billed at index {:?}",
                contracts,
                index
            );
        }
        }
    }

    // Helper function that updates the contract state and manages storage accordingly
    pub fn _update_contract_state(
        contract: &mut types::Contract<T>,
        state: &types::ContractState,
    ) -> DispatchResultWithPostInfo {
        // update the state and save the contract
        contract.state = state.clone();
        Contracts::<T>::insert(&contract.contract_id, contract.clone());

        Ok(().into())
    }

    fn remove_active_node_contract(node_id: u32, contract_id: u64) {
        let mut contracts = ActiveNodeContracts::<T>::get(&node_id);

        contracts.retain(|&id| id != contract_id);

        ActiveNodeContracts::<T>::insert(&node_id, &contracts);
    }

    pub fn _reserve_ip(
        contract_id: u64,
        deployment_contract: &mut types::DeploymentContract<T>,
    ) -> DispatchResultWithPostInfo {
        let mut contract = Contracts::<T>::get(deployment_contract.capacity_reservation_id)
            .ok_or(Error::<T>::CapacityReservationNotExists)?;
        let mut capacity_reservation_contract = Self::get_capacity_reservation_contract(&contract)?;
        if deployment_contract.public_ips == 0 {
            return Ok(().into());
        }
        let node = pallet_tfgrid::Nodes::<T>::get(capacity_reservation_contract.node_id)
            .ok_or(Error::<T>::NodeNotExists)?;

        let mut farm =
            pallet_tfgrid::Farms::<T>::get(node.farm_id).ok_or(Error::<T>::FarmNotExists)?;

        log::info!(
            "Number of farm ips {:?}, number of ips to reserve: {:?}",
            farm.public_ips.len(),
            capacity_reservation_contract.public_ips as usize
        );
        ensure!(
            farm.public_ips.len() >= deployment_contract.public_ips as usize,
            Error::<T>::FarmHasNotEnoughPublicIPs
        );

        let mut ips: BoundedVec<
            PublicIP<
                <T as pallet_tfgrid::Config>::PublicIP,
                <T as pallet_tfgrid::Config>::GatewayIP,
            >,
            MaxNodeContractPublicIPs<T>,
        > = vec![].try_into().unwrap();

        for i in 0..farm.public_ips.len() {
            if ips.len() == deployment_contract.public_ips as usize {
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
            ips.len() == deployment_contract.public_ips as usize,
            Error::<T>::FarmHasNotEnoughPublicIPsFree
        );

        deployment_contract.public_ips_list = ips.try_into().or_else(|_| {
            return Err(DispatchErrorWithPostInfo::from(
                Error::<T>::FailedToReserveIP,
            ));
        })?;

        // Update the farm with the reserved ips
        pallet_tfgrid::Farms::<T>::insert(farm.id, farm);

        // Update the capacity reservation contract for billing pub ips
        capacity_reservation_contract.public_ips += deployment_contract.public_ips;
        contract.contract_type =
            types::ContractData::CapacityReservationContract(capacity_reservation_contract);
        Contracts::<T>::insert(contract.contract_id, &contract);

        Ok(().into())
    }

    pub fn _free_ip(
        contract_id: u64,
        deployment_contract: &mut types::DeploymentContract<T>,
    ) -> DispatchResultWithPostInfo {
        let mut contract = Contracts::<T>::get(deployment_contract.capacity_reservation_id)
            .ok_or(Error::<T>::CapacityReservationNotExists)?;
        let mut capacity_reservation_contract = Self::get_capacity_reservation_contract(&contract)?;
        let node = pallet_tfgrid::Nodes::<T>::get(capacity_reservation_contract.node_id)
            .ok_or(Error::<T>::NodeNotExists)?;

        let mut farm =
            pallet_tfgrid::Farms::<T>::get(node.farm_id).ok_or(Error::<T>::FarmNotExists)?;

        let mut public_ips: BoundedVec<
            PublicIP<
                <T as pallet_tfgrid::Config>::PublicIP,
                <T as pallet_tfgrid::Config>::GatewayIP,
            >,
            MaxNodeContractPublicIPs<T>,
        > = vec![].try_into().unwrap();
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

        // Update the capacity reservation contract for billing pub ips
        capacity_reservation_contract.public_ips -= deployment_contract.public_ips;
        contract.contract_type =
            types::ContractData::CapacityReservationContract(capacity_reservation_contract);
        Contracts::<T>::insert(contract.contract_id, &contract);

        // Emit an event containing the IP's freed for this contract
        Self::deposit_event(Event::IPsFreed {
            contract_id,
            public_ips,
        });

        Ok(().into())
    }

    pub fn get_deployment_contract(
        contract: &types::Contract<T>,
    ) -> Result<types::DeploymentContract<T>, DispatchErrorWithPostInfo> {
        match contract.contract_type.clone() {
            types::ContractData::DeploymentContract(c) => Ok(c),
            _ => {
                return Err(DispatchErrorWithPostInfo::from(
                    Error::<T>::InvalidContractType,
                ))
            }
        }
    }

    pub fn get_capacity_reservation_contract(
        contract: &types::Contract<T>,
    ) -> Result<types::CapacityReservationContract, DispatchErrorWithPostInfo> {
        match contract.contract_type.clone() {
            types::ContractData::CapacityReservationContract(c) => Ok(c),
            _ => {
                return Err(DispatchErrorWithPostInfo::from(
                    Error::<T>::InvalidContractType,
                ))
            }
        }
    }

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
            None => BalanceOf::<T>::saturated_from(0 as u128),
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

    pub fn get_contract_index() -> u64 {
        let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
        now % BillingFrequency::<T>::get()
    }

    pub fn get_contract_index() -> u64 {
        let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
        now % BillingFrequency::<T>::get()
    }
}

impl<T: Config> ChangeNode<LocationOf<T>, PubConfigOf<T>, InterfaceOf<T>, SerialNumberOf<T>>
    for Pallet<T>
{
    fn node_changed(_node: Option<&TfgridNode<T>>, _new_node: &TfgridNode<T>) {}

    fn node_deleted(node: &TfgridNode<T>) {
        // Clean up all active contracts
        for contract_id in ActiveNodeContracts::<T>::get(node.id) {
            if let Some(mut contract) = Contracts::<T>::get(contract_id) {
                // Bill contract
                let _ = Self::_update_contract_state(
                    &mut contract,
                    &types::ContractState::Deleted(types::Cause::CanceledByUser),
                );
                let _ = Self::bill_contract(contract_id);
                Self::remove_contract(contract_id);
            }
        }
    }
}
