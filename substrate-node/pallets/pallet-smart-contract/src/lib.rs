#![cfg_attr(not(feature = "std"), no_std)]

use std::u8;

use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResultWithPostInfo,
    ensure,
    traits::{
        Currency, EnsureOrigin, ExistenceRequirement::KeepAlive, Get, LockableCurrency, Vec,
        WithdrawReasons,
    },
    weights::Pays,
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::{traits::SaturatedConversion, DispatchError, DispatchResult, Perbill, Percent};

use pallet_balances;
use pallet_tfgrid;
use pallet_tfgrid::types as pallet_tfgrid_types;
use pallet_tft_price;
use pallet_timestamp as timestamp;
use substrate_fixed::types::U64F64;
use tfchain_support::{
    traits::ChangeNode,
    types::{Node, NodeCertification, PublicIP, Resources},
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod benchmarking;

pub mod types;
pub mod weights;
pub mod contract_migration;

pub use weights::WeightInfo;

pub trait Config:
    system::Config + pallet_tfgrid::Config + pallet_timestamp::Config + pallet_balances::Config
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Currency: LockableCurrency<Self::AccountId>;
    type StakingPoolAccount: Get<Self::AccountId>;
    type BillingFrequency: Get<u64>;
    type DistributionFrequency: Get<u16>;
    type GracePeriod: Get<u64>;
    type WeightInfo: WeightInfo;
    type NodeChanged: ChangeNode;
    /// Origin for restricted extrinsics
    /// Can be the root or another origin configured in the runtime
    type RestrictedOrigin: EnsureOrigin<Self::Origin>;
}

pub const CONTRACT_VERSION: u32 = 3;

pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        BalanceOf = BalanceOf<T>,
    {
        ContractCreated(types::Contract),
        ContractUpdated(types::Contract),
        NodeContractCanceled(u64, u32, u32),
        NameContractCanceled(u64),
        IPsReserved(u64, Vec<PublicIP>),
        IPsFreed(u64, Vec<Vec<u8>>),
        ContractDeployed(u64, AccountId),
        // Deprecated
        ConsumptionReportReceived(types::Consumption),
        ContractBilled(types::ContractBill),
        TokensBurned(u64, BalanceOf),
        UpdatedUsedResources(types::ContractResources),
        NruConsumptionReportReceived(types::NruConsumption),
        RentContractCanceled(u64),
        ContractGracePeriodStarted(u64, u32, u32, u64),
        ContractGracePeriodEnded(u64, u32, u32),
        NodeMarkedAsDedicated(u32, bool),
        SolutionProviderCreated(types::SolutionProvider<AccountId>),
        SolutionProviderApproved(u64, bool),
    }
);

decl_error! {
    /// Error for the smart contract module.
    pub enum Error for Module<T: Config> {
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
        InvalidProviderConfiguration,
        NoSuchSolutionProvider,
        SolutionProviderNotApproved
    }
}

decl_storage! {
    trait Store for Module<T: Config> as SmartContractModule {
        pub Contracts get(fn contracts): map hasher(blake2_128_concat) u64 => types::Contract;
        pub ContractBillingInformationByID get(fn contract_billing_information_by_id): map hasher(blake2_128_concat) u64 => types::ContractBillingInformation;
        pub NodeContractResources get(fn node_contract_resources): map hasher(blake2_128_concat) u64 => types::ContractResources;

        // ContractIDByNodeIDAndHash is a mapping for a contract ID by supplying a node_id and a deployment_hash
        // this combination makes a deployment for a user / node unique
        pub ContractIDByNodeIDAndHash get(fn node_contract_by_hash): double_map hasher(blake2_128_concat) u32, hasher(blake2_128_concat) Vec<u8> => u64;

        pub ActiveNodeContracts get(fn active_node_contracts): map hasher(blake2_128_concat) u32 => Vec<u64>;
        pub ContractsToBillAt get(fn contract_to_bill_at_block): map hasher(blake2_128_concat) u64 => Vec<u64>;
        pub ContractLock get(fn contract_number_of_cylces_billed): map hasher(blake2_128_concat) u64 => types::ContractLock<BalanceOf<T>>;
        pub ContractIDByNameRegistration get(fn contract_id_by_name_registration): map hasher(blake2_128_concat) Vec<u8> => u64;
        pub ActiveRentContractForNode get(fn active_rent_contracts): map hasher(blake2_128_concat) u32 => types::Contract;

        pub SolutionProviders get(fn solution_providers): map hasher(blake2_128_concat) u64 => types::SolutionProvider<T::AccountId>;

        // ID maps
        pub ContractID: u64;
        pub SolutionProviderID: u64;

        /// The current version of the pallet.
        PalletVersion: types::PalletStorageVersion = types::PalletStorageVersion::V1;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        #[weight = 100_000_000]
        fn create_node_contract(origin, node_id: u32, data: Vec<u8>, deployment_hash: Vec<u8>, public_ips: u32, solution_provider_id: Option<u64>){
            let account_id = ensure_signed(origin)?;
            Self::_create_node_contract(account_id, node_id, data, deployment_hash, public_ips, solution_provider_id)?;
        }

        #[weight = 100_000_000]
        fn update_node_contract(origin, contract_id: u64, data: Vec<u8>, deployment_hash: Vec<u8>){
            let account_id = ensure_signed(origin)?;
            Self::_update_node_contract(account_id, contract_id, data, deployment_hash)?;
        }

        #[weight = 100_000_000]
        fn cancel_contract(origin, contract_id: u64){
            let account_id = ensure_signed(origin)?;
            Self::_cancel_contract(account_id, contract_id, types::Cause::CanceledByUser)?;
        }

        // DEPRECATED
        #[weight = <T as Config>::WeightInfo::add_reports().saturating_mul(_reports.len() as u64)]
        fn add_reports(_origin, _reports: Vec<types::Consumption>) -> DispatchResultWithPostInfo {
            // return error
            Err(DispatchError::from(Error::<T>::MethodIsDeprecated).into())
        }

        #[weight = 100_000_000]
        fn create_name_contract(origin, name: Vec<u8>) {
            let account_id = ensure_signed(origin)?;
            Self::_create_name_contract(account_id, name)?;
        }

        #[weight = <T as Config>::WeightInfo::add_reports().saturating_mul(reports.len() as u64)]
        fn add_nru_reports(origin, reports: Vec<types::NruConsumption>) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_compute_reports(account_id, reports)
        }

        #[weight = 100_000_000]
        fn report_contract_resources(origin, contract_resources: Vec<types::ContractResources>) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;
            Self::_report_contract_resources(account_id, contract_resources)
        }

        #[weight = 100_000_000]
        fn create_rent_contract(origin, node_id: u32) {
            let account_id = ensure_signed(origin)?;
            Self::_create_rent_contract(account_id, node_id)?;
        }

        #[weight = 100_000_000 + T::DbWeight::get().writes(3) + T::DbWeight::get().reads(2)]
        pub fn set_node_dedicated(origin, node_id: u32, dedicated: bool) -> DispatchResult {
            let account_id = ensure_signed(origin)?;
            Self::_set_node_dedicated(account_id, node_id, dedicated)
        }

        #[weight = 100_000_000 + T::DbWeight::get().writes(3) + T::DbWeight::get().reads(2)]
        pub fn create_solution_provider(origin, description: Vec<u8>, link: Vec<u8>, providers: Vec<types::Provider<T::AccountId>>) -> DispatchResult {
            ensure_signed(origin)?;
            Self::_create_solution_provider(description, link, providers)
        }

        #[weight = 100_000_000 + T::DbWeight::get().writes(3) + T::DbWeight::get().reads(2)]
        pub fn approve_solution_provider(origin, solution_provider_id: u64, approve: bool) -> DispatchResult {
            <T as Config>::RestrictedOrigin::ensure_origin(origin)?;
            Self::_approve_solution_provider(solution_provider_id, approve)
        }

        fn on_finalize(block: T::BlockNumber) {
            match Self::_bill_contracts_at_block(block) {
                Ok(_) => {
                    debug::info!("types::NodeContract billed successfully at block: {:?}", block);
                },
                Err(err) => {
                    debug::info!("types::NodeContract billed failed at block: {:?} with err {:?}", block, err);
                }
            }
            // clean storage map for billed contracts at block
            let current_block_u64: u64 = block.saturated_into::<u64>();
            ContractsToBillAt::remove(current_block_u64);
        }
    }
}

impl<T: Config> Module<T> {
    pub fn _create_node_contract(
        account_id: T::AccountId,
        node_id: u32,
        deployment_data: Vec<u8>,
        deployment_hash: Vec<u8>,
        public_ips: u32,
        solution_provider_id: Option<u64>,
    ) -> DispatchResult {
        ensure!(
            pallet_tfgrid::TwinIdByAccountID::<T>::contains_key(&account_id),
            Error::<T>::TwinNotExists
        );
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id);

        ensure!(
            pallet_tfgrid::Nodes::contains_key(&node_id),
            Error::<T>::NodeNotExists
        );

        let node = pallet_tfgrid::Nodes::get(node_id);
        if node.dedicated && !ActiveRentContractForNode::contains_key(node_id) {
            return Err(DispatchError::from(Error::<T>::NodeNotAvailableToDeploy));
        }

        // If the user is trying to deploy on a node that has an active rent contract
        // only allow the user who created the rent contract to actually deploy a node contract on it
        if ActiveRentContractForNode::contains_key(node_id) {
            let contract = ActiveRentContractForNode::get(node_id);
            if contract.twin_id != twin_id {
                return Err(DispatchError::from(Error::<T>::NodeHasRentContract));
            }
        }

        // If the contract with hash and node id exists and it's in any other state then
        // contractState::Deleted then we don't allow the creation of it.
        // If it exists we allow the user to "restore" this contract
        if ContractIDByNodeIDAndHash::contains_key(node_id, &deployment_hash) {
            let contract_id = ContractIDByNodeIDAndHash::get(node_id, &deployment_hash);
            let contract = Contracts::get(contract_id);
            if !contract.is_state_delete() {
                return Err(DispatchError::from(Error::<T>::ContractIsNotUnique));
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
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id);
        let contract = Self::_create_contract(
            twin_id,
            types::ContractData::NodeContract(node_contract.clone()),
            solution_provider_id,
        )?;

        let now = <timestamp::Module<T>>::get().saturated_into::<u64>() / 1000;
        let contract_billing_information = types::ContractBillingInformation {
            last_updated: now,
            amount_unbilled: 0,
            previous_nu_reported: 0,
        };
        ContractBillingInformationByID::insert(contract.contract_id, contract_billing_information);

        // Insert contract id by (node_id, hash)
        ContractIDByNodeIDAndHash::insert(node_id, deployment_hash, contract.contract_id);

        // Insert contract into active contracts map
        let mut node_contracts = ActiveNodeContracts::get(&node_contract.node_id);
        node_contracts.push(contract.contract_id);
        ActiveNodeContracts::insert(&node_contract.node_id, &node_contracts);

        Self::deposit_event(RawEvent::ContractCreated(contract));

        Ok(())
    }

    pub fn _create_rent_contract(account_id: T::AccountId, node_id: u32) -> DispatchResult {
        ensure!(
            pallet_tfgrid::TwinIdByAccountID::<T>::contains_key(&account_id),
            Error::<T>::TwinNotExists
        );
        ensure!(
            pallet_tfgrid::Nodes::contains_key(&node_id),
            Error::<T>::NodeNotExists
        );

        let active_node_contracts = ActiveNodeContracts::get(node_id);
        ensure!(
            active_node_contracts.len() == 0,
            Error::<T>::NodeHasActiveContracts
        );

        ensure!(
            !ActiveRentContractForNode::contains_key(node_id),
            Error::<T>::NodeHasRentContract
        );

        let node = pallet_tfgrid::Nodes::get(node_id);
        ensure!(node.dedicated, Error::<T>::NodeIsNotDedicated);

        // Create contract
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id);
        let contract = Self::_create_contract(
            twin_id,
            types::ContractData::RentContract(types::RentContract { node_id }),
            None,
        )?;

        // Insert active rent contract for node
        ActiveRentContractForNode::insert(node_id, contract.clone());

        Self::deposit_event(RawEvent::ContractCreated(contract));

        Ok(())
    }

    // Registers a DNS name for a Twin
    // Ensures uniqueness and also checks if it's a valid DNS name
    pub fn _create_name_contract(source: T::AccountId, name: Vec<u8>) -> DispatchResult {
        ensure!(
            pallet_tfgrid::TwinIdByAccountID::<T>::contains_key(&source),
            Error::<T>::TwinNotExists
        );
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&source);

        // Validate name uniqueness
        ensure!(
            !ContractIDByNameRegistration::contains_key(&name),
            Error::<T>::NameExists
        );

        for character in &name {
            match character {
                c if *c == 45 => (),
                c if *c >= 48 && *c <= 57 => (),
                c if *c >= 65 && *c <= 122 => (),
                _ => return Err(DispatchError::from(Error::<T>::NameNotValid)),
            }
        }
        let name_contract = types::NameContract { name: name.clone() };

        let contract = Self::_create_contract(
            twin_id,
            types::ContractData::NameContract(name_contract),
            None,
        )?;

        ContractIDByNameRegistration::insert(name, &contract.contract_id);

        Self::deposit_event(RawEvent::ContractCreated(contract));

        Ok(())
    }

    fn _create_contract(
        twin_id: u32,
        mut contract_type: types::ContractData,
        solution_provider_id: Option<u64>,
    ) -> Result<types::Contract, DispatchError> {
        // Get the Contract ID map and increment
        let mut id = ContractID::get();
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
        Self::_reinsert_contract_to_bill(id);

        // insert into contracts map
        Contracts::insert(id, &contract);

        // Update Contract ID
        ContractID::put(id);

        let now = <timestamp::Module<T>>::get().saturated_into::<u64>() / 1000;
        let mut contract_lock = types::ContractLock::default();
        contract_lock.lock_updated = now;
        ContractLock::<T>::insert(id, contract_lock);

        Ok(contract)
    }

    pub fn _update_node_contract(
        account_id: T::AccountId,
        contract_id: u64,
        deployment_data: Vec<u8>,
        deployment_hash: Vec<u8>,
    ) -> DispatchResult {
        ensure!(
            Contracts::contains_key(contract_id),
            Error::<T>::ContractNotExists
        );

        let mut contract = Contracts::get(contract_id);
        let twin = pallet_tfgrid::Twins::<T>::get(contract.twin_id);
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
        ContractIDByNodeIDAndHash::remove(node_contract.node_id, node_contract.deployment_hash);
        ContractIDByNodeIDAndHash::insert(node_contract.node_id, &deployment_hash, contract_id);

        node_contract.deployment_data = deployment_data;
        node_contract.deployment_hash = deployment_hash;

        // override values
        contract.contract_type = types::ContractData::NodeContract(node_contract);

        let state = contract.state.clone();
        Self::_update_contract_state(&mut contract, &state)?;

        Self::deposit_event(RawEvent::ContractUpdated(contract));

        Ok(())
    }

    pub fn _cancel_contract(
        account_id: T::AccountId,
        contract_id: u64,
        cause: types::Cause,
    ) -> DispatchResult {
        ensure!(
            Contracts::contains_key(contract_id),
            Error::<T>::ContractNotExists
        );

        let mut contract = Contracts::get(contract_id);
        let twin = pallet_tfgrid::Twins::<T>::get(contract.twin_id);
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
            let active_node_contracts = ActiveNodeContracts::get(rent_contract.node_id);
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

        Ok(())
    }

    pub fn _report_contract_resources(
        source: T::AccountId,
        contract_resources: Vec<types::ContractResources>,
    ) -> DispatchResultWithPostInfo {
        ensure!(
            pallet_tfgrid::TwinIdByAccountID::<T>::contains_key(&source),
            Error::<T>::TwinNotExists
        );
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&source);
        ensure!(
            pallet_tfgrid::NodeIdByTwinID::contains_key(twin_id),
            Error::<T>::NodeNotExists
        );
        let node_id = pallet_tfgrid::NodeIdByTwinID::get(twin_id);

        for contract_resource in contract_resources {
            if !Contracts::contains_key(contract_resource.contract_id) {
                continue;
            }
            // we know contract exists, fetch it
            // if the node is trying to send garbage data we can throw an error here
            let contract = Contracts::get(contract_resource.contract_id);
            let node_contract = Self::get_node_contract(&contract)?;
            ensure!(
                node_contract.node_id == node_id,
                Error::<T>::NodeNotAuthorizedToComputeReport
            );

            // Do insert
            NodeContractResources::insert(contract_resource.contract_id, &contract_resource);
            // deposit event
            Self::deposit_event(RawEvent::UpdatedUsedResources(contract_resource));
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
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&source);
        ensure!(
            pallet_tfgrid::NodeIdByTwinID::contains_key(twin_id),
            Error::<T>::NodeNotExists
        );

        // fetch the node from the source account (signee)
        let node_id = pallet_tfgrid::NodeIdByTwinID::get(&twin_id);
        let node = pallet_tfgrid::Nodes::get(node_id);

        ensure!(
            pallet_tfgrid::Farms::contains_key(&node.farm_id),
            Error::<T>::FarmNotExists
        );
        let farm = pallet_tfgrid::Farms::get(node.farm_id);

        ensure!(
            pallet_tfgrid::PricingPolicies::<T>::contains_key(farm.pricing_policy_id),
            Error::<T>::PricingPolicyNotExists
        );
        let pricing_policy = pallet_tfgrid::PricingPolicies::<T>::get(farm.pricing_policy_id);

        // validation
        for report in &reports {
            if !Contracts::contains_key(report.contract_id) {
                continue;
            }
            if !ContractBillingInformationByID::contains_key(report.contract_id) {
                continue;
            }

            // we know contract exists, fetch it
            // if the node is trying to send garbage data we can throw an error here
            let contract = Contracts::get(report.contract_id);
            let node_contract = Self::get_node_contract(&contract)?;
            ensure!(
                node_contract.node_id == node_id,
                Error::<T>::NodeNotAuthorizedToComputeReport
            );

            Self::_calculate_report_cost(&report, &pricing_policy);
            Self::deposit_event(RawEvent::NruConsumptionReportReceived(report.clone()));
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
        let mut contract_billing_info = ContractBillingInformationByID::get(report.contract_id);
        if report.timestamp < contract_billing_info.last_updated {
            return;
        }

        // seconds elapsed is the report.window
        let seconds_elapsed = report.window;
        debug::info!("seconds elapsed: {:?}", seconds_elapsed);

        // calculate NRU used and the cost
        let used_nru = U64F64::from_num(report.nru) / pricing_policy.nu.factor();
        let nu_cost = used_nru
            * (U64F64::from_num(pricing_policy.nu.value) / 3600)
            * U64F64::from_num(seconds_elapsed);
        debug::info!("nu cost: {:?}", nu_cost);

        // save total
        let total = nu_cost.ceil().to_num::<u64>();
        debug::info!("total cost: {:?}", total);

        // update contract billing info
        contract_billing_info.amount_unbilled += total;
        contract_billing_info.last_updated = report.timestamp;
        ContractBillingInformationByID::insert(report.contract_id, &contract_billing_info);
    }

    pub fn _bill_contracts_at_block(block: T::BlockNumber) -> DispatchResult {
        let current_block_u64: u64 = block.saturated_into::<u64>();
        let contracts = ContractsToBillAt::get(current_block_u64);
        for contract_id in contracts {
            let mut contract = Contracts::get(contract_id);
            if contract.contract_id == 0 {
                continue;
            }

            // Try to bill contract
            match Self::bill_contract(&mut contract) {
                Ok(_) => {
                    debug::info!(
                        "billed contract with id {:?} at block {:?}",
                        contract_id,
                        block
                    );
                }
                Err(err) => {
                    debug::info!(
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
            let contract = Contracts::get(contract_id);
            if contract.contract_id != 0 && contract.is_state_delete() {
                Self::remove_contract(contract.contract_id);
                continue;
            }

            // Reinsert into the next billing frequency
            Self::_reinsert_contract_to_bill(contract.contract_id);
        }
        Ok(())
    }

    // Bills a contract (NodeContract or NameContract)
    // Calculates how much TFT is due by the user and distributes the rewards
    fn bill_contract(contract: &mut types::Contract) -> DispatchResult {
        if !pallet_tfgrid::Twins::<T>::contains_key(contract.twin_id) {
            return Err(DispatchError::from(Error::<T>::TwinNotExists));
        }
        let twin = pallet_tfgrid::Twins::<T>::get(contract.twin_id);
        let usable_balance = Self::get_usable_balance(&twin.account_id);

        let mut seconds_elapsed = T::BillingFrequency::get() * 6;
        // Calculate amount of seconds elapsed based on the contract lock struct

        let now = <timestamp::Module<T>>::get().saturated_into::<u64>() / 1000;
        // this will set the seconds elapsed to the default billing cycle duration in seconds
        // if there is no contract lock object yet. A contract lock object will be created later in this function
        // https://github.com/threefoldtech/tfchain/issues/261
        let contract_lock = ContractLock::<T>::get(contract.contract_id);
        if contract_lock.lock_updated != 0 {
            seconds_elapsed = now.checked_sub(contract_lock.lock_updated).unwrap_or(0);
        }

        let (amount_due, discount_received) =
            Self::calculate_contract_cost_tft(contract, usable_balance, seconds_elapsed)?;

        // If there is nothing to be paid, return
        if amount_due == BalanceOf::<T>::saturated_from(0 as u128) {
            return Ok(());
        };

        // Handle grace
        let contract = Self::handle_grace(contract, usable_balance, amount_due)?;

        // Handle contract lock operations
        Self::handle_lock(contract, amount_due);

        // Always emit a contract billed event
        let contract_bill = types::ContractBill {
            contract_id: contract.contract_id,
            timestamp: <timestamp::Module<T>>::get().saturated_into::<u64>() / 1000,
            discount_level: discount_received.clone(),
            amount_billed: amount_due.saturated_into::<u128>(),
        };
        Self::deposit_event(RawEvent::ContractBilled(contract_bill));

        // set the amount unbilled back to 0
        let mut contract_billing_info = ContractBillingInformationByID::get(contract.contract_id);
        contract_billing_info.amount_unbilled = 0;
        ContractBillingInformationByID::insert(contract.contract_id, &contract_billing_info);

        // If the contract is in delete state, remove all associated storage
        if matches!(contract.state, types::ContractState::Deleted(_)) {
            Self::remove_contract(contract.contract_id);
        }

        Ok(())
    }

    fn handle_grace(
        contract: &mut types::Contract,
        usable_balance: BalanceOf<T>,
        amount_due: BalanceOf<T>,
    ) -> Result<&mut types::Contract, DispatchError> {
        let current_block = <frame_system::Module<T>>::block_number().saturated_into::<u64>();
        let node_id = contract.get_node_id();

        match contract.state {
            types::ContractState::GracePeriod(grace_start) => {
                // if the usable balance is recharged, we can move the contract to created state again
                if usable_balance > amount_due {
                    Self::_update_contract_state(contract, &types::ContractState::Created)?;
                    Self::deposit_event(RawEvent::ContractGracePeriodEnded(
                        contract.contract_id,
                        node_id,
                        contract.twin_id,
                    ))
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
                    Self::deposit_event(RawEvent::ContractGracePeriodStarted(
                        contract.contract_id,
                        node_id,
                        contract.twin_id,
                        current_block.saturated_into(),
                    ));
                }
            }
        }

        Ok(contract)
    }

    fn handle_lock(contract: &mut types::Contract, amount_due: BalanceOf<T>) {
        let now = <timestamp::Module<T>>::get().saturated_into::<u64>() / 1000;
        let mut contract_lock = ContractLock::<T>::get(contract.contract_id);
        let new_amount_locked = contract_lock.amount_locked + amount_due;

        // increment cycles billed and update the internal lock struct
        contract_lock.lock_updated = now;
        contract_lock.cycles += 1;
        contract_lock.amount_locked = new_amount_locked;
        ContractLock::<T>::insert(contract.contract_id, &contract_lock);

        // Contract is in grace state, don't actually lock tokens or distribute rewards
        if matches!(contract.state, types::ContractState::GracePeriod(_)) {
            return;
        }

        let twin = pallet_tfgrid::Twins::<T>::get(contract.twin_id);
        // Only lock an amount from the user's balance if the contract is in create state
        // Update lock for contract and ContractLock in storage
        <T as Config>::Currency::extend_lock(
            contract.contract_id.to_be_bytes(),
            &twin.account_id,
            new_amount_locked.into(),
            WithdrawReasons::RESERVE,
        );

        let is_canceled = matches!(contract.state, types::ContractState::Deleted(_));
        let canceled_and_not_zero =
            is_canceled && contract_lock.amount_locked.saturated_into::<u64>() > 0;
        // When the cultivation rewards are ready to be distributed or it's in delete state
        // Unlock all reserved balance and distribute
        if contract_lock.cycles >= T::DistributionFrequency::get() || canceled_and_not_zero {
            // Remove lock
            <T as Config>::Currency::remove_lock(
                contract.contract_id.to_be_bytes(),
                &twin.account_id,
            );
            // Fetch the default pricing policy
            let pricing_policy = pallet_tfgrid::PricingPolicies::<T>::get(1);
            // Distribute cultivation rewards
            match Self::_distribute_cultivation_rewards(
                &contract,
                &pricing_policy,
                contract_lock.amount_locked,
            ) {
                Ok(_) => (),
                Err(err) => debug::info!("error while distributing cultivation rewards {:?}", err),
            };
            // Reset values
            contract_lock.lock_updated = now;
            contract_lock.amount_locked = BalanceOf::<T>::saturated_from(0 as u128);
            contract_lock.cycles = 0;
            ContractLock::<T>::insert(contract.contract_id, &contract_lock);
        }
    }

    fn calculate_contract_cost_tft(
        contract: &types::Contract,
        balance: BalanceOf<T>,
        seconds_elapsed: u64,
    ) -> Result<(BalanceOf<T>, types::DiscountLevel), DispatchError> {
        // Fetch the default pricing policy and certification type
        let pricing_policy = pallet_tfgrid::PricingPolicies::<T>::get(1);
        let certification_type = NodeCertification::Diy;

        // Calculate the cost for a contract, can be any of:
        // - NodeContract
        // - RentContract
        // - NameContract
        let total_cost = Self::calculate_contract_cost(contract, &pricing_policy, seconds_elapsed)?;
        // If cost is 0, reinsert to be billed at next interval
        if total_cost == 0 {
            return Ok((
                BalanceOf::<T>::saturated_from(0 as u128),
                types::DiscountLevel::None,
            ));
        }

        let total_cost_tft_64 = Self::calculate_cost_in_tft_from_musd(total_cost)?;

        // Calculate the amount due and discount received based on the total_cost amount due
        let (amount_due, discount_received) =
            Self::calculate_discount(total_cost_tft_64, balance, certification_type);

        return Ok((amount_due, discount_received));
    }

    pub fn calculate_contract_cost(
        contract: &types::Contract,
        pricing_policy: &pallet_tfgrid_types::PricingPolicy<T::AccountId>,
        seconds_elapsed: u64,
    ) -> Result<u64, DispatchError> {
        let total_cost = match &contract.contract_type {
            // Calculate total cost for a node contract
            types::ContractData::NodeContract(node_contract) => {
                // Get the contract billing info to view the amount unbilled for NRU (network resource units)
                let contract_billing_info =
                    ContractBillingInformationByID::get(contract.contract_id);
                // Get the node
                if !pallet_tfgrid::Nodes::contains_key(node_contract.node_id) {
                    return Err(DispatchError::from(Error::<T>::NodeNotExists));
                }

                // We know the contract is using resources, now calculate the cost for each used resource
                let node_contract_resources = NodeContractResources::get(contract.contract_id);

                let mut bill_resources = true;
                // If this node contract is deployed on a node which has a rent contract
                // We can ignore billing for the resources used by this node contract
                if ActiveRentContractForNode::contains_key(node_contract.node_id) {
                    bill_resources = false
                }

                let contract_cost = Self::calculate_resources_cost(
                    node_contract_resources.used,
                    node_contract.public_ips,
                    seconds_elapsed,
                    pricing_policy.clone(),
                    bill_resources,
                );
                contract_cost + contract_billing_info.amount_unbilled
            }
            types::ContractData::RentContract(rent_contract) => {
                if !pallet_tfgrid::Nodes::contains_key(rent_contract.node_id) {
                    return Err(DispatchError::from(Error::<T>::NodeNotExists));
                }
                let node = pallet_tfgrid::Nodes::get(rent_contract.node_id);

                let contract_cost = Self::calculate_resources_cost(
                    node.resources,
                    0,
                    seconds_elapsed,
                    pricing_policy.clone(),
                    true,
                );
                Percent::from_percent(pricing_policy.discount_for_dedication_nodes) * contract_cost
            }
            // Calculate total cost for a name contract
            types::ContractData::NameContract(_) => {
                // bill user for name usage for number of seconds elapsed
                let total_cost_u64f64 = (U64F64::from_num(pricing_policy.unique_name.value) / 3600)
                    * U64F64::from_num(seconds_elapsed);
                total_cost_u64f64.to_num::<u64>()
            }
        };

        Ok(total_cost)
    }

    // Calculates the total cost of a node contract.
    pub fn calculate_resources_cost(
        resources: Resources,
        ipu: u32,
        seconds_elapsed: u64,
        pricing_policy: pallet_tfgrid_types::PricingPolicy<T::AccountId>,
        bill_resources: bool,
    ) -> u64 {
        let mut total_cost = U64F64::from_num(0);

        if bill_resources {
            let hru = U64F64::from_num(resources.hru) / pricing_policy.su.factor();
            let sru = U64F64::from_num(resources.sru) / pricing_policy.su.factor();
            let mru = U64F64::from_num(resources.mru) / pricing_policy.cu.factor();
            let cru = U64F64::from_num(resources.cru);

            let su_used = hru / 1200 + sru / 200;
            // the pricing policy su cost value is expressed in 1 hours or 3600 seconds.
            // we bill every 3600 seconds but here we need to calculate the cost per second and multiply it by the seconds elapsed.
            let su_cost = (U64F64::from_num(pricing_policy.su.value) / 3600)
                * U64F64::from_num(seconds_elapsed)
                * su_used;
            debug::info!("su cost: {:?}", su_cost);

            let cu = Self::calculate_cu(cru, mru);

            let cu_cost = (U64F64::from_num(pricing_policy.cu.value) / 3600)
                * U64F64::from_num(seconds_elapsed)
                * cu;
            debug::info!("cu cost: {:?}", cu_cost);
            total_cost = su_cost + cu_cost;
        }

        if ipu > 0 {
            let total_ip_cost = U64F64::from_num(ipu)
                * (U64F64::from_num(pricing_policy.ipu.value) / 3600)
                * U64F64::from_num(seconds_elapsed);
            debug::info!("ip cost: {:?}", total_ip_cost);
            total_cost += total_ip_cost;
        }

        return total_cost.ceil().to_num::<u64>();
    }

    pub fn remove_contract(contract_id: u64) {
        let contract = Contracts::get(contract_id);

        match contract.contract_type.clone() {
            types::ContractData::NodeContract(mut node_contract) => {
                Self::remove_active_node_contract(node_contract.node_id, contract_id);
                if node_contract.public_ips > 0 {
                    match Self::_free_ip(contract_id, &mut node_contract) {
                        Ok(_) => (),
                        Err(e) => {
                            debug::info!("error while freeing ips: {:?}", e);
                        }
                    }
                }

                // remove the contract by hash from storage
                ContractIDByNodeIDAndHash::remove(
                    node_contract.node_id,
                    &node_contract.deployment_hash,
                );
                NodeContractResources::remove(contract_id);
                ContractBillingInformationByID::remove(contract_id);

                Self::deposit_event(RawEvent::NodeContractCanceled(
                    contract_id,
                    node_contract.node_id,
                    contract.twin_id,
                ));
            }
            types::ContractData::NameContract(name_contract) => {
                ContractIDByNameRegistration::remove(name_contract.name);
                Self::deposit_event(RawEvent::NameContractCanceled(contract_id));
            }
            types::ContractData::RentContract(rent_contract) => {
                ActiveRentContractForNode::remove(rent_contract.node_id);
                // Remove all associated active node contracts
                let active_node_contracts = ActiveNodeContracts::get(rent_contract.node_id);
                for node_contract in active_node_contracts {
                    Self::remove_contract(node_contract);
                }
                Self::deposit_event(RawEvent::RentContractCanceled(contract_id));
            }
        };

        Contracts::remove(contract_id);
        ContractLock::<T>::remove(contract_id);
    }

    pub fn calculate_cost_in_tft_from_musd(total_cost_musd: u64) -> Result<u64, DispatchError> {
        let tft_price_musd = U64F64::from_num(pallet_tft_price::AverageTftPrice::get()) * 1000;
        ensure!(tft_price_musd > 0, Error::<T>::TFTPriceValueError);

        let total_cost_musd = U64F64::from_num(total_cost_musd) / 10000;

        let total_cost_tft = (total_cost_musd / tft_price_musd) * U64F64::from_num(1e7);
        let total_cost_tft_64: u64 = U64F64::to_num(total_cost_tft);
        Ok(total_cost_tft_64)
    }

    // Following: https://library.threefold.me/info/threefold#/tfgrid/farming/threefold__proof_of_utilization
    fn _distribute_cultivation_rewards(
        contract: &types::Contract,
        pricing_policy: &pallet_tfgrid_types::PricingPolicy<T::AccountId>,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        // fetch source twin
        let twin = pallet_tfgrid::Twins::<T>::get(contract.twin_id);

        // Send 10% to the foundation
        let foundation_share = Perbill::from_percent(10) * amount;
        debug::info!(
            "Transfering: {:?} from contract twin {:?} to foundation account {:?}",
            &foundation_share,
            &twin.account_id,
            &pricing_policy.foundation_account
        );
        <T as Config>::Currency::transfer(
            &twin.account_id,
            &pricing_policy.foundation_account,
            foundation_share,
            KeepAlive,
        )
        .map_err(|_| DispatchError::Other("Can't make foundation share transfer"))?;

        // TODO: send 5% to the staking pool account
        let staking_pool_share = Perbill::from_percent(5) * amount;
        let staking_pool_account = T::StakingPoolAccount::get();
        debug::info!(
            "Transfering: {:?} from contract twin {:?} to staking pool account {:?}",
            &staking_pool_share,
            &twin.account_id,
            &staking_pool_account,
        );
        <T as Config>::Currency::transfer(
            &twin.account_id,
            &staking_pool_account,
            staking_pool_share,
            KeepAlive,
        )
        .map_err(|_| DispatchError::Other("Can't make staking pool share transfer"))?;

        let mut sales_share = 50;

        let _ = match contract.solution_provider_id {
            Some(provider_id) => {
                let solution_provider = SolutionProviders::<T>::get(provider_id);
                let total_take: u8 = solution_provider
                    .providers
                    .iter()
                    .map(|provider| provider.take)
                    .sum();
                sales_share = sales_share - total_take;

                let _ = solution_provider.providers.iter().map(|provider| {
                    let share = Perbill::from_percent(provider.take as u32) * amount;
                    debug::info!(
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
                    .unwrap_or(debug::info!("error"));
                });
            }
            None => (),
        };

        if sales_share > 0 {
            let share = Perbill::from_percent(sales_share.into()) * amount;
            // Send 50% to the sales channel
            debug::info!(
                "Transfering: {:?} from contract twin {:?} to sales account {:?}",
                &sales_share,
                &twin.account_id,
                &pricing_policy.certified_sales_account
            );
            <T as Config>::Currency::transfer(
                &twin.account_id,
                &pricing_policy.certified_sales_account,
                share,
                KeepAlive,
            )
            .map_err(|_| DispatchError::Other("Can't make staking pool share transfer"))?;
        }

        // Burn 35%, to not have any imbalance in the system, subtract all previously send amounts with the initial
        let mut amount_to_burn =
            amount - foundation_share - staking_pool_share - (Perbill::from_percent(50) * amount);

        let existential_deposit_requirement = <T as Config>::Currency::minimum_balance();
        let free_balance = <T as Config>::Currency::free_balance(&twin.account_id);
        if amount_to_burn > free_balance - existential_deposit_requirement {
            amount_to_burn = <T as Config>::Currency::free_balance(&twin.account_id)
                - existential_deposit_requirement;
        }

        <T as Config>::Currency::slash(&twin.account_id, amount_to_burn);
        Self::deposit_event(RawEvent::TokensBurned(contract.contract_id, amount_to_burn));
        Ok(())
    }

    // Calculates the discount that will be applied to the billing of the contract
    // Returns an amount due as balance object and a static string indicating which kind of discount it received
    // (default, bronze, silver, gold or none)
    pub fn calculate_discount(
        amount_due: u64,
        balance: BalanceOf<T>,
        certification_type: NodeCertification,
    ) -> (BalanceOf<T>, types::DiscountLevel) {
        let balance_as_u128: u128 = balance.saturated_into::<u128>();

        // calculate amount due on a monthly basis
        // we bill every one hour so we can infer the amount due monthly (30 days ish)
        let amount_due_monthly = amount_due * 24 * 30;

        // see how many months a user can pay for this deployment given his balance
        let discount_level =
            U64F64::from_num(balance_as_u128) / U64F64::from_num(amount_due_monthly);

        // predefined discount levels
        // https://wiki.threefold.io/#/threefold__grid_pricing
        let discount_received = match discount_level.floor().to_num::<u64>() {
            d if d >= 3 && d < 6 => types::DiscountLevel::Default,
            d if d >= 6 && d < 12 => types::DiscountLevel::Bronze,
            d if d >= 12 && d < 36 => types::DiscountLevel::Silver,
            d if d >= 36 => types::DiscountLevel::Gold,
            _ => types::DiscountLevel::None,
        };

        // calculate the new amount due given the discount
        let mut amount_due = U64F64::from_num(amount_due) * discount_received.price_multiplier();

        // Certified capacity costs 25% more
        if certification_type == NodeCertification::Certified {
            amount_due = amount_due * U64F64::from_num(1.25);
        }

        // convert to balance object
        let amount_due: BalanceOf<T> =
            BalanceOf::<T>::saturated_from(amount_due.ceil().to_num::<u64>());

        (amount_due, discount_received)
    }

    // Reinserts a contract by id at the next interval we need to bill the contract
    pub fn _reinsert_contract_to_bill(contract_id: u64) {
        if contract_id == 0 {
            return;
        }

        let now = <frame_system::Module<T>>::block_number().saturated_into::<u64>();
        // Save the contract to be billed in now + BILLING_FREQUENCY_IN_BLOCKS
        let future_block = now + T::BillingFrequency::get();
        let mut contracts = ContractsToBillAt::get(future_block);
        contracts.push(contract_id);
        ContractsToBillAt::insert(future_block, &contracts);
        debug::info!(
            "Insert contracts: {:?}, to be billed at block {:?}",
            contracts,
            future_block
        );
    }

    // Helper function that updates the contract state and manages storage accordingly
    pub fn _update_contract_state(
        contract: &mut types::Contract,
        state: &types::ContractState,
    ) -> DispatchResult {
        // update the state and save the contract
        contract.state = state.clone();
        Contracts::insert(&contract.contract_id, contract.clone());

        // if the contract is a name contract, nothing to do left here
        match contract.contract_type {
            types::ContractData::NameContract(_) => return Ok(()),
            types::ContractData::RentContract(_) => return Ok(()),
            _ => (),
        };

        // if the contract is a node contract
        // manage the ActiveNodeContracts map accordingly
        let node_contract = Self::get_node_contract(contract)?;

        let mut contracts = ActiveNodeContracts::get(&node_contract.node_id);

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

        ActiveNodeContracts::insert(&node_contract.node_id, &contracts);

        Ok(())
    }

    fn remove_active_node_contract(node_id: u32, contract_id: u64) {
        let mut contracts = ActiveNodeContracts::get(&node_id);

        match contracts.iter().position(|id| id == &contract_id) {
            Some(index) => {
                contracts.remove(index);
            }
            None => (),
        };

        ActiveNodeContracts::insert(&node_id, &contracts);
    }

    pub fn _reserve_ip(
        contract_id: u64,
        node_contract: &mut types::NodeContract,
    ) -> DispatchResult {
        if node_contract.public_ips == 0 {
            return Ok(());
        }
        let node = pallet_tfgrid::Nodes::get(node_contract.node_id);

        ensure!(
            pallet_tfgrid::Farms::contains_key(&node.farm_id),
            Error::<T>::FarmNotExists
        );
        let mut farm = pallet_tfgrid::Farms::get(node.farm_id);

        debug::info!(
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
        pallet_tfgrid::Farms::insert(farm.id, farm);

        node_contract.public_ips_list = ips;

        Ok(())
    }

    pub fn _free_ip(contract_id: u64, node_contract: &mut types::NodeContract) -> DispatchResult {
        let node = pallet_tfgrid::Nodes::get(node_contract.node_id);

        ensure!(
            pallet_tfgrid::Farms::contains_key(&node.farm_id),
            Error::<T>::FarmNotExists
        );
        let mut farm = pallet_tfgrid::Farms::get(node.farm_id);

        let mut ips_freed = Vec::new();
        for i in 0..farm.public_ips.len() {
            let mut ip = farm.public_ips[i].clone();

            // if an ip has contract id 0 it means it's not reserved
            // reserve it now
            if ip.contract_id == contract_id {
                ip.contract_id = 0;
                farm.public_ips[i] = ip.clone();
                ips_freed.push(ip.ip);
            }
        }

        pallet_tfgrid::Farms::insert(farm.id, farm);

        // Emit an event containing the IP's freed for this contract
        Self::deposit_event(RawEvent::IPsFreed(contract_id, ips_freed));

        Ok(())
    }

    pub fn get_node_contract(
        contract: &types::Contract,
    ) -> Result<types::NodeContract, DispatchError> {
        match contract.contract_type.clone() {
            types::ContractData::NodeContract(c) => Ok(c),
            _ => return Err(DispatchError::from(Error::<T>::InvalidContractType)),
        }
    }

    pub fn get_rent_contract(
        contract: &types::Contract,
    ) -> Result<types::RentContract, DispatchError> {
        match contract.contract_type.clone() {
            types::ContractData::RentContract(c) => Ok(c),
            _ => return Err(DispatchError::from(Error::<T>::InvalidContractType)),
        }
    }

    fn get_usable_balance(account_id: &T::AccountId) -> BalanceOf<T> {
        let balance = pallet_balances::pallet::Pallet::<T>::usable_balance(account_id);
        let b = balance.saturated_into::<u128>();
        BalanceOf::<T>::saturated_from(b)
    }

    // cu1 = MAX(cru/2, mru/4)
    // cu2 = MAX(cru, mru/8)
    // cu3 = MAX(cru/4, mru/2)

    // CU = MIN(cu1, cu2, cu3)
    pub(crate) fn calculate_cu(cru: U64F64, mru: U64F64) -> U64F64 {
        let mru_used_1 = mru / 4;
        let cru_used_1 = cru / 2;
        let cu1 = if mru_used_1 > cru_used_1 {
            mru_used_1
        } else {
            cru_used_1
        };

        let mru_used_2 = mru / 8;
        let cru_used_2 = cru;
        let cu2 = if mru_used_2 > cru_used_2 {
            mru_used_2
        } else {
            cru_used_2
        };

        let mru_used_3 = mru / 2;
        let cru_used_3 = cru / 4;
        let cu3 = if mru_used_3 > cru_used_3 {
            mru_used_3
        } else {
            cru_used_3
        };

        let mut cu = if cu1 > cu2 { cu2 } else { cu1 };

        cu = if cu > cu3 { cu3 } else { cu };

        cu
    }

    pub fn decomssion_workloads_on_node(node_id: u32) {
        // Clean up all active contracts
        let active_node_contracts = ActiveNodeContracts::get(node_id);
        for node_contract_id in active_node_contracts {
            let mut contract = Contracts::get(node_contract_id);
            // Bill contract
            let _ = Self::_update_contract_state(
                &mut contract,
                &types::ContractState::Deleted(types::Cause::CanceledByUser),
            );
            let _ = Self::bill_contract(&mut contract);
            Self::remove_contract(node_contract_id);
        }

        // First clean up rent contract if it exists
        let mut rent_contract = ActiveRentContractForNode::get(node_id);
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

    pub fn _set_node_dedicated(
        origin: T::AccountId,
        node_id: u32,
        mark_dedicated: bool,
    ) -> DispatchResult {
        // If the node is marked as dedicated check for active contracts and throw and error if there are
        // This makes sure that a node is "empty" before it can be fully rented
        if mark_dedicated {
            ensure!(
                !ActiveRentContractForNode::contains_key(node_id),
                Error::<T>::NodeHasRentContract
            );
            let active_node_contracts = ActiveNodeContracts::get(node_id);
            ensure!(
                active_node_contracts.len() == 0,
                Error::<T>::NodeHasActiveContracts
            );
        }

        ensure!(
            pallet_tfgrid::Nodes::contains_key(node_id),
            pallet_tfgrid::Error::<T>::NodeNotExists
        );
        let node = pallet_tfgrid::Nodes::get(node_id);

        ensure!(
            pallet_tfgrid::Farms::contains_key(node.farm_id),
            pallet_tfgrid::Error::<T>::FarmNotExists
        );
        let farm = pallet_tfgrid::Farms::get(node.farm_id);

        let farm_twin = pallet_tfgrid::Twins::<T>::get(farm.twin_id);
        ensure!(
            farm_twin.account_id == origin,
            pallet_tfgrid::Error::<T>::NodeUpdateNotAuthorized
        );

        let mut stored_node = pallet_tfgrid::Nodes::get(node_id);

        // If the node is toggled from dedicated to non-dedicated -> make sure to delete all contracts
        if stored_node.dedicated && !mark_dedicated {
            Self::decomssion_workloads_on_node(node_id);
        }

        stored_node.dedicated = mark_dedicated;
        pallet_tfgrid::Nodes::insert(node_id, &stored_node);

        Self::deposit_event(RawEvent::NodeMarkedAsDedicated(node_id, mark_dedicated));

        Ok(())
    }

    pub fn _create_solution_provider(
        description: Vec<u8>,
        link: Vec<u8>,
        providers: Vec<types::Provider<T::AccountId>>,
    ) -> DispatchResult {
        let total_take: u8 = providers.iter().map(|provider| provider.take).sum();
        ensure!(total_take <= 50, Error::<T>::InvalidProviderConfiguration);

        let mut id = SolutionProviderID::get();
        id = id + 1;

        let solution_provider = types::SolutionProvider {
            solution_provider_id: id,
            providers,
            description,
            link,
            approved: false,
        };

        SolutionProviderID::put(id);
        SolutionProviders::<T>::insert(id, &solution_provider);

        Self::deposit_event(RawEvent::SolutionProviderCreated(solution_provider));

        Ok(())
    }

    pub fn _approve_solution_provider(solution_provider_id: u64, approve: bool) -> DispatchResult {
        ensure!(
            SolutionProviders::<T>::contains_key(solution_provider_id),
            Error::<T>::NoSuchSolutionProvider
        );

        let mut solution_provider = SolutionProviders::<T>::get(solution_provider_id);
        solution_provider.approved = approve;
        SolutionProviders::<T>::insert(solution_provider_id, &solution_provider);

        Self::deposit_event(RawEvent::SolutionProviderApproved(
            solution_provider_id,
            approve,
        ));

        Ok(())
    }

    pub fn validate_solution_provider(solution_provider_id: Option<u64>) -> DispatchResult {
        match solution_provider_id {
            Some(provider_id) => {
                ensure!(
                    SolutionProviders::<T>::contains_key(provider_id),
                    Error::<T>::NoSuchSolutionProvider
                );
                let solution_provider = SolutionProviders::<T>::get(provider_id);
                ensure!(
                    solution_provider.approved,
                    Error::<T>::SolutionProviderNotApproved
                );
                return Ok(());
            }
            None => return Ok(()),
        };
    }
}

impl<T: Config> ChangeNode for Module<T> {
    fn node_changed(_node: Option<&Node>, _new_node: &Node) {}

    fn node_deleted(node: &Node) {
        Self::decomssion_workloads_on_node(node.id);
    }
}
