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
use tfchain_support::traits::ChangeNode;

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
        NameContractNameToShort,
        NameContractNameToLong,
        InvalidProviderConfiguration,
        NoSuchSolutionProvider,
        SolutionProviderNotApproved,
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
        pub fn create_node_contract(
            origin: OriginFor<T>,
            node_id: u32,
            deployment_hash: DeploymentHash,
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

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn update_node_contract(
            origin: OriginFor<T>,
            contract_id: u64,
            deployment_hash: DeploymentHash,
            deployment_data: DeploymentDataInput<T>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_update_node_contract(account_id, contract_id, deployment_hash, deployment_data)
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
        pub fn add_nru_reports(
            origin: OriginFor<T>,
            reports: Vec<types::NruConsumption>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_compute_reports(account_id, reports)
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn report_contract_resources(
            origin: OriginFor<T>,
            contract_resources: Vec<types::ContractResources>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_report_contract_resources(account_id, contract_resources)
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_rent_contract(
            origin: OriginFor<T>,
            node_id: u32,
            solution_provider_id: Option<u64>,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_create_rent_contract(account_id, node_id, solution_provider_id)
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
    pub fn _create_node_contract(
        account_id: T::AccountId,
        node_id: u32,
        deployment_hash: DeploymentHash,
        deployment_data: DeploymentDataInput<T>,
        public_ips: u32,
        solution_provider_id: Option<u64>,
    ) -> DispatchResultWithPostInfo {
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(Error::<T>::TwinNotExists)?;

        let node = pallet_tfgrid::Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;
        let farm = pallet_tfgrid::Farms::<T>::get(node.farm_id).ok_or(Error::<T>::FarmNotExists)?;

        if farm.dedicated_farm && !ActiveRentContractForNode::<T>::contains_key(node_id) {
            return Err(Error::<T>::NodeNotAvailableToDeploy.into());
        }

        // If the user is trying to deploy on a node that has an active rent contract
        // only allow the user who created the rent contract to actually deploy a node contract on it
        if let Some(contract_id) = ActiveRentContractForNode::<T>::get(node_id) {
            let rent_contract =
                Contracts::<T>::get(contract_id).ok_or(Error::<T>::ContractNotExists)?;
            if rent_contract.twin_id != twin_id {
                return Err(Error::<T>::NodeHasRentContract.into());
            }
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

        let public_ips_list: BoundedVec<
            PublicIP<
                <T as pallet_tfgrid::Config>::PublicIP,
                <T as pallet_tfgrid::Config>::GatewayIP,
            >,
            MaxNodeContractPublicIPs<T>,
        > = vec![].try_into().unwrap();
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

        let contract = types::Contract {
            version: CONTRACT_VERSION,
            twin_id,
            contract_id: id,
            state: types::ContractState::Created,
            contract_type,
            solution_provider_id,
        };

        // Start billing frequency loop
        // Will always be block now + frequency
        Self::insert_contract_to_bill(id);

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
        deployment_hash: DeploymentHash,
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
        // Remove all associated storage
        Self::remove_contract(contract.contract_id);

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
                NodeContractResources::<T>::insert(
                    contract_resource.contract_id,
                    &contract_resource,
                );
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
        ContractBillingInformationByID::<T>::insert(report.contract_id, &contract_billing_info);
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

    // Bills a contract (NodeContract or NameContract)
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
                    Self::handle_grace_rent_contract(contract, types::ContractState::Created)?;
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
                types::ContractData::NodeContract(mut node_contract) => {
                    Self::remove_active_node_contract(node_contract.node_id, contract_id);
                    if node_contract.public_ips > 0 {
                        match Self::_free_ip(contract_id, &mut node_contract) {
                            Ok(_) => (),
                            Err(e) => {
                                log::info!("error while freeing ips: {:?}", e);
                            }
                        }
                    }

                    // remove the contract by hash from storage
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
                    let active_node_contracts =
                        ActiveNodeContracts::<T>::get(rent_contract.node_id);
                    for node_contract in active_node_contracts {
                        Self::remove_contract(node_contract);
                    }
                    Self::deposit_event(Event::RentContractCanceled { contract_id });
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

        log::info!(
            "Number of farm ips {:?}, number of ips to reserve: {:?}",
            farm.public_ips.len(),
            node_contract.public_ips as usize
        );
        ensure!(
            farm.public_ips.len() >= node_contract.public_ips as usize,
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
}

impl<T: Config> ChangeNode<LocationOf<T>, PubConfigOf<T>, InterfaceOf<T>, SerialNumberOf<T>>
    for Pallet<T>
{
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
                Self::remove_contract(node_contract_id);
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
                Self::remove_contract(contract.contract_id);
            }
        }
    }
}
