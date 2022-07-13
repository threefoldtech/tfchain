#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;

use frame_support::{
    dispatch::DispatchErrorWithPostInfo,
    ensure,
    traits::{Currency, ExistenceRequirement, Get, LockableCurrency, WithdrawReasons},
    transactional,
    weights::Pays,
};
use frame_system::{self as system, ensure_signed};
use pallet_tfgrid;
use pallet_tfgrid::types as pallet_tfgrid_types;
use pallet_timestamp as timestamp;
use sp_runtime::{
    traits::{CheckedSub, SaturatedConversion},
    Perbill,
};
use substrate_fixed::types::U64F64;
use tfchain_support::{traits::ChangeNode, types::Node};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;

pub mod cost;
pub mod types;

#[frame_support::pallet]
pub mod pallet {
    use super::types::*;
    use super::weights::WeightInfo;
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        log,
        traits::{Currency, Get, LockIdentifier, LockableCurrency},
    };
    use frame_system::pallet_prelude::*;
    use sp_std::convert::TryInto;
    use sp_std::vec::Vec;
    use tfchain_support::{traits::ChangeNode, types::PublicIP};
    use sp_core::H256;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;

    pub const GRID_LOCK_ID: LockIdentifier = *b"gridlock";

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // Version constant that referenced the struct version
    pub const CONTRACT_VERSION: u32 = 3;

    pub type DeploymentHash = H256;
    #[pallet::storage]
    #[pallet::getter(fn contracts)]
    pub type Contracts<T: Config> = StorageMap<_, Blake2_128Concat, u64, Contract, ValueQuery>;

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
        StorageMap<_, Blake2_128Concat, Vec<u8>, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn active_rent_contracts)]
    pub type ActiveRentContractForNode<T: Config> =
        StorageMap<_, Blake2_128Concat, u32, Contract, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn contract_id)]
    pub type ContractID<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + pallet_timestamp::Config
        + pallet_balances::Config
        + pallet_tfgrid::Config
    {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: LockableCurrency<Self::AccountId>;
        type StakingPoolAccount: Get<Self::AccountId>;
        type BillingFrequency: Get<u64>;
        type DistributionFrequency: Get<u16>;
        type GracePeriod: Get<u64>;
        type WeightInfo: WeightInfo;
        type NodeChanged: ChangeNode;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A contract got created
        ContractCreated(types::Contract),
        /// A contract was updated
        ContractUpdated(types::Contract),
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
            public_ips: Vec<PublicIP>,
        },
        /// IP got freed by a Node contract
        IPsFreed {
            contract_id: u64,
            // public ip as a string
            public_ips: Vec<Vec<u8>>,
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
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create_node_contract(
            origin: OriginFor<T>,
            node_id: u32,
            data: Vec<u8>,
            deployment_hash: DeploymentHash,
            public_ips: u32,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_create_node_contract(account_id, node_id, data, deployment_hash, public_ips)
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn update_node_contract(
            origin: OriginFor<T>,
            contract_id: u64,
            data: Vec<u8>,
            deployment_hash: DeploymentHash,
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_update_node_contract(account_id, contract_id, data, deployment_hash)
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
        ) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_create_rent_contract(account_id, node_id)
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(block: T::BlockNumber) {
            match Self::_bill_contracts_at_block(block) {
                Ok(_) => {
                    log::info!(
                        "types::NodeContract billed successfully at block: {:?}",
                        block
                    );
                }
                Err(err) => {
                    log::info!(
                        "types::NodeContract billed failed at block: {:?} with err {:?}",
                        block,
                        err
                    );
                }
            }
            // clean storage map for billed contracts at block
            let current_block_u64: u64 = block.saturated_into::<u64>();
            ContractsToBillAt::<T>::remove(current_block_u64);
        }
    }
}

use frame_support::pallet_prelude::DispatchResultWithPostInfo;
// Internal functions of the pallet
impl<T: Config> Pallet<T> {
    pub fn _create_node_contract(
        account_id: T::AccountId,
        node_id: u32,
        deployment_data: Vec<u8>,
        deployment_hash: DeploymentHash,
        public_ips: u32,
    ) -> DispatchResultWithPostInfo {
        ensure!(
            pallet_tfgrid::pallet::TwinIdByAccountID::<T>::contains_key(&account_id),
            Error::<T>::TwinNotExists
        );
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id).unwrap();

        ensure!(
            pallet_tfgrid::Nodes::<T>::contains_key(&node_id),
            Error::<T>::NodeNotExists
        );

        let node = pallet_tfgrid::Nodes::<T>::get(node_id);
        ensure!(
            pallet_tfgrid::Farms::<T>::contains_key(node.farm_id),
            Error::<T>::FarmNotExists
        );
        let farm = pallet_tfgrid::Farms::<T>::get(node.farm_id);

        if farm.dedicated_farm && !ActiveRentContractForNode::<T>::contains_key(node_id) {
            return Err(Error::<T>::NodeNotAvailableToDeploy.into());
        }

        // If the user is trying to deploy on a node that has an active rent contract
        // only allow the user who created the rent contract to actually deploy a node contract on it
        if ActiveRentContractForNode::<T>::contains_key(node_id) {
            let contract = ActiveRentContractForNode::<T>::get(node_id);
            if contract.twin_id != twin_id {
                return Err(Error::<T>::NodeHasRentContract.into());
            }
        }

        // If the contract with hash and node id exists and it's in any other state then
        // contractState::Deleted then we don't allow the creation of it.
        // If it exists we allow the user to "restore" this contract
        if ContractIDByNodeIDAndHash::<T>::contains_key(node_id, &deployment_hash) {
            let contract_id = ContractIDByNodeIDAndHash::<T>::get(node_id, &deployment_hash);
            let contract = Contracts::<T>::get(contract_id);
            if !contract.is_state_delete() {
                return Err(Error::<T>::ContractIsNotUnique.into());
            }
        }

        // Prepare NodeContract struct
        let node_contract = types::NodeContract {
            node_id,
            deployment_data,
            deployment_hash: deployment_hash.clone(),
            public_ips,
            public_ips_list: Vec::new(),
        };

        // Create contract
        let contract = Self::_create_contract(
            twin_id,
            types::ContractData::NodeContract(node_contract.clone()),
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
    ) -> DispatchResultWithPostInfo {
        ensure!(
            pallet_tfgrid::TwinIdByAccountID::<T>::contains_key(&account_id),
            Error::<T>::TwinNotExists
        );
        ensure!(
            pallet_tfgrid::Nodes::<T>::contains_key(&node_id),
            Error::<T>::NodeNotExists
        );

        ensure!(
            !ActiveRentContractForNode::<T>::contains_key(node_id),
            Error::<T>::NodeHasRentContract
        );

        let node = pallet_tfgrid::Nodes::<T>::get(node_id);
        ensure!(
            pallet_tfgrid::Farms::<T>::contains_key(node.farm_id),
            Error::<T>::FarmNotExists
        );

        let active_node_contracts = ActiveNodeContracts::<T>::get(node_id);
        let farm = pallet_tfgrid::Farms::<T>::get(node.farm_id);
        ensure!(
            farm.dedicated_farm || active_node_contracts.is_empty(),
            Error::<T>::NodeNotAvailableToDeploy
        );

        // Create contract
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id).unwrap();
        let contract = Self::_create_contract(
            twin_id,
            types::ContractData::RentContract(types::RentContract { node_id }),
        )?;

        // Insert active rent contract for node
        ActiveRentContractForNode::<T>::insert(node_id, contract.clone());

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
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&source).unwrap();

        // Validate name uniqueness
        ensure!(
            !ContractIDByNameRegistration::<T>::contains_key(&name),
            Error::<T>::NameExists
        );

        for character in &name {
            match character {
                c if *c == 45 => (),
                c if *c >= 48 && *c <= 57 => (),
                c if *c >= 65 && *c <= 122 => (),
                _ => return Err(Error::<T>::NameNotValid.into()),
            }
        }
        let name_contract = types::NameContract { name: name.clone() };

        let contract =
            Self::_create_contract(twin_id, types::ContractData::NameContract(name_contract))?;

        ContractIDByNameRegistration::<T>::insert(name, &contract.contract_id);

        Self::deposit_event(Event::ContractCreated(contract));

        Ok(().into())
    }

    fn _create_contract(
        twin_id: u32,
        mut contract_type: types::ContractData,
    ) -> Result<types::Contract, DispatchErrorWithPostInfo> {
        // Get the Contract ID map and increment
        let mut id = ContractID::<T>::get();
        id = id + 1;

        if let types::ContractData::NodeContract(ref mut nc) = contract_type {
            Self::_reserve_ip(id, nc)?;
        };

        let contract = types::Contract {
            version: CONTRACT_VERSION,
            twin_id,
            contract_id: id,
            state: types::ContractState::Created,
            contract_type,
        };

        // Start billing frequency loop
        // Will always be block now + frequency
        Self::_reinsert_contract_to_bill(id);

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
        deployment_data: Vec<u8>,
        deployment_hash: DeploymentHash,
    ) -> DispatchResultWithPostInfo {
        ensure!(
            Contracts::<T>::contains_key(contract_id),
            Error::<T>::ContractNotExists
        );

        let mut contract = Contracts::<T>::get(contract_id);
        ensure!(
            pallet_tfgrid::Twins::<T>::contains_key(contract.twin_id),
            Error::<T>::TwinNotExists
        );
        let twin = pallet_tfgrid::Twins::<T>::get(contract.twin_id).unwrap();
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

        node_contract.deployment_data = deployment_data;
        node_contract.deployment_hash = deployment_hash;

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
        ensure!(
            Contracts::<T>::contains_key(contract_id),
            Error::<T>::ContractNotExists
        );
        let mut contract = Contracts::<T>::get(contract_id);
        ensure!(
            pallet_tfgrid::Twins::<T>::contains_key(contract.twin_id),
            Error::<T>::TwinNotExists
        );
        let twin = pallet_tfgrid::Twins::<T>::get(contract.twin_id).unwrap();
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

        // Update state
        Self::_update_contract_state(&mut contract, &types::ContractState::Deleted(cause))?;
        // Bill contract
        Self::bill_contract(&mut contract)?;
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
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&source).unwrap();
        ensure!(
            pallet_tfgrid::NodeIdByTwinID::<T>::contains_key(twin_id),
            Error::<T>::NodeNotExists
        );
        let node_id = pallet_tfgrid::NodeIdByTwinID::<T>::get(twin_id);

        for contract_resource in contract_resources {
            if !Contracts::<T>::contains_key(contract_resource.contract_id) {
                continue;
            }
            // we know contract exists, fetch it
            // if the node is trying to send garbage data we can throw an error here
            let contract = Contracts::<T>::get(contract_resource.contract_id);
            let node_contract = Self::get_node_contract(&contract)?;
            ensure!(
                node_contract.node_id == node_id,
                Error::<T>::NodeNotAuthorizedToComputeReport
            );

            // Do insert
            NodeContractResources::<T>::insert(contract_resource.contract_id, &contract_resource);
            // deposit event
            Self::deposit_event(Event::UpdatedUsedResources(contract_resource));
        }

        Ok(Pays::No.into())
    }

    pub fn _compute_reports(
        source: T::AccountId,
        reports: Vec<types::NruConsumption>,
    ) -> DispatchResultWithPostInfo {
        ensure!(
            pallet_tfgrid::TwinIdByAccountID::<T>::contains_key(&source),
            Error::<T>::TwinNotExists
        );
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&source).unwrap();
        ensure!(
            pallet_tfgrid::NodeIdByTwinID::<T>::contains_key(twin_id),
            Error::<T>::NodeNotExists
        );

        // fetch the node from the source account (signee)
        let node_id = pallet_tfgrid::NodeIdByTwinID::<T>::get(&twin_id);
        let node = pallet_tfgrid::Nodes::<T>::get(node_id);

        ensure!(
            pallet_tfgrid::Farms::<T>::contains_key(&node.farm_id),
            Error::<T>::FarmNotExists
        );
        let farm = pallet_tfgrid::Farms::<T>::get(node.farm_id);

        ensure!(
            pallet_tfgrid::PricingPolicies::<T>::contains_key(farm.pricing_policy_id),
            Error::<T>::PricingPolicyNotExists
        );
        let pricing_policy =
            pallet_tfgrid::PricingPolicies::<T>::get(farm.pricing_policy_id).unwrap();

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
            let contract = Contracts::<T>::get(report.contract_id);
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
        let used_nru = U64F64::from_num(report.nru) / pricing_policy.nu.factor();
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

    pub fn _bill_contracts_at_block(block: T::BlockNumber) -> DispatchResultWithPostInfo {
        let current_block_u64: u64 = block.saturated_into::<u64>();
        let contracts = ContractsToBillAt::<T>::get(current_block_u64);
        for contract_id in contracts {
            let mut contract = Contracts::<T>::get(contract_id);
            if contract.contract_id == 0 {
                continue;
            }

            // Try to bill contract
            match Self::bill_contract(&mut contract) {
                Ok(_) => {
                    log::info!(
                        "billed contract with id {:?} at block {:?}",
                        contract_id,
                        block
                    );
                }
                Err(err) => {
                    log::info!(
                        "error while billing contract with id {:?}: {:?}",
                        contract_id,
                        err
                    );
                    // If billing the contract failed, we should delete the contract and clean up storage
                    Self::remove_contract(contract.contract_id);
                    continue;
                }
            }

            // https://github.com/threefoldtech/tfchain/issues/264
            // if a contract is still in storage and actively getting billed whilst it is in state delete
            // remove all associated storage and continue
            let contract = Contracts::<T>::get(contract_id);
            if contract.contract_id != 0 && contract.is_state_delete() {
                Self::remove_contract(contract.contract_id);
                continue;
            }

            // Reinsert into the next billing frequency
            Self::_reinsert_contract_to_bill(contract.contract_id);
        }
        Ok(().into())
    }

    // Bills a contract (NodeContract or NameContract)
    // Calculates how much TFT is due by the user and distributes the rewards
    #[transactional]
    fn bill_contract(contract: &mut types::Contract) -> DispatchResultWithPostInfo {
        if !pallet_tfgrid::Twins::<T>::contains_key(contract.twin_id) {
            return Err(DispatchErrorWithPostInfo::from(Error::<T>::TwinNotExists));
        }
        let twin = pallet_tfgrid::Twins::<T>::get(contract.twin_id).unwrap();
        let usable_balance = Self::get_usable_balance(&twin.account_id);

        let mut seconds_elapsed = T::BillingFrequency::get() * 6;
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
            contract.calculate_contract_cost_tft::<T>(usable_balance, seconds_elapsed)?;

        // If there is nothing to be paid, return
        if amount_due == BalanceOf::<T>::saturated_from(0 as u128) {
            return Ok(().into());
        };

        // Handle grace
        let contract = Self::handle_grace(contract, usable_balance, amount_due)?;

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

        Ok(().into())
    }

    fn handle_grace(
        contract: &mut types::Contract,
        usable_balance: BalanceOf<T>,
        amount_due: BalanceOf<T>,
    ) -> Result<&mut types::Contract, DispatchErrorWithPostInfo> {
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
                    })
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
            _ => {
                // if the user ran out of funds, move the contract to be in a grace period
                // dont lock the tokens because there is nothing to lock
                // we can still update the internal contract lock object to figure out later how much was due
                // whilst in grace period
                if amount_due >= usable_balance {
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
                }
            }
        }

        Ok(contract)
    }

    fn handle_lock(
        contract: &mut types::Contract,
        amount_due: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
        let now = <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;

        // increment cycles billed and update the internal lock struct
        let mut contract_lock = ContractLock::<T>::get(contract.contract_id);
        contract_lock.lock_updated = now;
        contract_lock.cycles += 1;
        contract_lock.amount_locked = contract_lock.amount_locked + amount_due;
        ContractLock::<T>::insert(contract.contract_id, &contract_lock);

        // Contract is in grace state, don't actually lock tokens or distribute rewards
        if matches!(contract.state, types::ContractState::GracePeriod(_)) {
            return Ok(().into());
        }

        // Only lock an amount from the user's balance if the contract is in create state
        // The lock is specified on the user's account, since a user can have multiple contracts
        // Just extend the lock with the amount due for this contract billing period (lock will be created if not exists)
        let twin = pallet_tfgrid::Twins::<T>::get(contract.twin_id).unwrap();
        let mut locked_balance = Self::get_locked_balance(&twin.account_id);
        locked_balance += amount_due;
        <T as Config>::Currency::extend_lock(
            GRID_LOCK_ID,
            &twin.account_id,
            locked_balance,
            WithdrawReasons::all(),
        );

        let is_canceled = matches!(contract.state, types::ContractState::Deleted(_));
        let canceled_and_not_zero =
            is_canceled && contract_lock.amount_locked.saturated_into::<u64>() > 0;
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

            // Fetch the default pricing policy
            let pricing_policy = pallet_tfgrid::PricingPolicies::<T>::get(1).unwrap();
            // Distribute cultivation rewards
            match Self::_distribute_cultivation_rewards(
                &contract,
                &pricing_policy,
                contract_lock.amount_locked,
            ) {
                Ok(_) => (),
                Err(err) => log::info!("error while distributing cultivation rewards {:?}", err),
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
                let active_node_contracts = ActiveNodeContracts::<T>::get(rent_contract.node_id);
                for node_contract in active_node_contracts {
                    Self::remove_contract(node_contract);
                }
                Self::deposit_event(Event::RentContractCanceled { contract_id });
            }
        };

        Contracts::<T>::remove(contract_id);
        ContractLock::<T>::remove(contract_id);
    }

    // Following: https://library.threefold.me/info/threefold#/tfgrid/farming/threefold__proof_of_utilization
    fn _distribute_cultivation_rewards(
        contract: &types::Contract,
        pricing_policy: &pallet_tfgrid_types::PricingPolicy<T::AccountId>,
        amount: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
        // fetch source twin
        let twin = pallet_tfgrid::Twins::<T>::get(contract.twin_id).unwrap();

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

        // Send 50% to the sales channel
        let sales_share = Perbill::from_percent(50) * amount;
        log::info!(
            "Transfering: {:?} from contract twin {:?} to sales account {:?}",
            &sales_share,
            &twin.account_id,
            &pricing_policy.certified_sales_account
        );
        <T as Config>::Currency::transfer(
            &twin.account_id,
            &pricing_policy.certified_sales_account,
            sales_share,
            ExistenceRequirement::KeepAlive,
        )?;

        // Burn 35%, to not have any imbalance in the system, subtract all previously send amounts with the initial
        let mut amount_to_burn = amount - foundation_share - staking_pool_share - sales_share;

        let existential_deposit_requirement = <T as Config>::Currency::minimum_balance();
        let free_balance = <T as Config>::Currency::free_balance(&twin.account_id);
        if amount_to_burn > free_balance - existential_deposit_requirement {
            amount_to_burn = <T as Config>::Currency::free_balance(&twin.account_id)
                - existential_deposit_requirement;
        }

        <T as Config>::Currency::slash(&twin.account_id, amount_to_burn);
        Self::deposit_event(Event::TokensBurned {
            contract_id: contract.contract_id,
            amount: amount_to_burn,
        });
        Ok(().into())
    }

    // Reinserts a contract by id at the next interval we need to bill the contract
    pub fn _reinsert_contract_to_bill(contract_id: u64) {
        if contract_id == 0 {
            return;
        }

        let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
        // Save the contract to be billed in now + BILLING_FREQUENCY_IN_BLOCKS
        let future_block = now + T::BillingFrequency::get();
        let mut contracts = ContractsToBillAt::<T>::get(future_block);
        contracts.push(contract_id);
        ContractsToBillAt::<T>::insert(future_block, &contracts);
        log::info!(
            "Insert contracts: {:?}, to be billed at block {:?}",
            contracts,
            future_block
        );
    }

    // Helper function that updates the contract state and manages storage accordingly
    pub fn _update_contract_state(
        contract: &mut types::Contract,
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
        node_contract: &mut types::NodeContract,
    ) -> DispatchResultWithPostInfo {
        if node_contract.public_ips == 0 {
            return Ok(().into());
        }
        let node = pallet_tfgrid::Nodes::<T>::get(node_contract.node_id);

        ensure!(
            pallet_tfgrid::Farms::<T>::contains_key(&node.farm_id),
            Error::<T>::FarmNotExists
        );
        let mut farm = pallet_tfgrid::Farms::<T>::get(node.farm_id);

        log::info!(
            "Number of farm ips {:?}, number of ips to reserve: {:?}",
            farm.public_ips.len(),
            node_contract.public_ips as usize
        );
        ensure!(
            farm.public_ips.len() >= node_contract.public_ips as usize,
            Error::<T>::FarmHasNotEnoughPublicIPs
        );

        let mut ips = Vec::new();
        for i in 0..farm.public_ips.len() {
            let mut ip = farm.public_ips[i].clone();

            if ips.len() == node_contract.public_ips as usize {
                break;
            }

            // if an ip has contract id 0 it means it's not reserved
            // reserve it now
            if ip.contract_id == 0 {
                ip.contract_id = contract_id;
                farm.public_ips[i] = ip.clone();
                ips.push(ip);
            }
        }

        // Safeguard check if we actually have the amount of ips we wanted to reserve
        ensure!(
            ips.len() == node_contract.public_ips as usize,
            Error::<T>::FarmHasNotEnoughPublicIPsFree
        );

        // Update the farm with the reserved ips
        pallet_tfgrid::Farms::<T>::insert(farm.id, farm);

        node_contract.public_ips_list = ips;

        Ok(().into())
    }

    pub fn _free_ip(
        contract_id: u64,
        node_contract: &mut types::NodeContract,
    ) -> DispatchResultWithPostInfo {
        let node = pallet_tfgrid::Nodes::<T>::get(node_contract.node_id);

        ensure!(
            pallet_tfgrid::Farms::<T>::contains_key(&node.farm_id),
            Error::<T>::FarmNotExists
        );
        let mut farm = pallet_tfgrid::Farms::<T>::get(node.farm_id);

        let mut public_ips = Vec::new();
        for i in 0..farm.public_ips.len() {
            let mut ip = farm.public_ips[i].clone();

            // if an ip has contract id 0 it means it's not reserved
            // reserve it now
            if ip.contract_id == contract_id {
                ip.contract_id = 0;
                farm.public_ips[i] = ip.clone();
                public_ips.push(ip.ip);
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
        contract: &types::Contract,
    ) -> Result<types::NodeContract, DispatchErrorWithPostInfo> {
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
        contract: &types::Contract,
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
}

impl<T: Config> ChangeNode for Pallet<T> {
    fn node_changed(_node: Option<&Node>, _new_node: &Node) {}

    fn node_deleted(node: &Node) {
        // Clean up all active contracts
        let active_node_contracts = ActiveNodeContracts::<T>::get(node.id);
        for node_contract_id in active_node_contracts {
            let mut contract = Contracts::<T>::get(node_contract_id);
            // Bill contract
            let _ = Self::_update_contract_state(
                &mut contract,
                &types::ContractState::Deleted(types::Cause::CanceledByUser),
            );
            let _ = Self::bill_contract(&mut contract);
            Self::remove_contract(node_contract_id);
        }

        // First clean up rent contract if it exists
        let mut rent_contract = ActiveRentContractForNode::<T>::get(node.id);
        if rent_contract.contract_id != 0 {
            // Bill contract
            let _ = Self::_update_contract_state(
                &mut rent_contract,
                &types::ContractState::Deleted(types::Cause::CanceledByUser),
            );
            let _ = Self::bill_contract(&mut rent_contract);
            Self::remove_contract(rent_contract.contract_id);
        }
    }
}
