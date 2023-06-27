use crate::*;
use frame_support::{
    dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo, Pays},
    ensure, BoundedVec,
};
use pallet_tfgrid::pallet::{InterfaceOf, LocationOf, SerialNumberOf, TfgridNode};
use sp_std::{vec, vec::Vec};
use tfchain_support::{
    traits::{ChangeNode, PublicIpModifier},
    types::PublicIP,
};

impl<T: Config> Pallet<T> {
    pub fn _create_node_contract(
        account_id: T::AccountId,
        node_id: u32,
        deployment_hash: types::HexHash,
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
            DedicatedNodesExtraFee::<T>::get(node_id) > 0 || farm.dedicated_farm;

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
        let contract = Self::create_contract(
            twin_id,
            types::ContractData::NodeContract(node_contract.clone()),
            solution_provider_id,
        )?;

        let now = Self::get_current_timestamp_in_secs();
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
        let contract = Self::create_contract(
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

        let contract = Self::create_contract(
            twin_id,
            types::ContractData::NameContract(name_contract),
            None,
        )?;

        ContractIDByNameRegistration::<T>::insert(valid_name, &contract.contract_id);

        Self::deposit_event(Event::ContractCreated(contract));

        Ok(().into())
    }

    fn create_contract(
        twin_id: u32,
        mut contract_type: types::ContractData<T>,
        solution_provider_id: Option<u64>,
    ) -> Result<types::Contract<T>, DispatchErrorWithPostInfo> {
        // Get the Contract ID map and increment
        let mut id = ContractID::<T>::get();
        id = id + 1;

        if let types::ContractData::NodeContract(ref mut nc) = contract_type {
            Self::reserve_ip(id, nc)?;
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

        let now = Self::get_current_timestamp_in_secs();
        let mut contract_lock = types::ContractLock::default();
        contract_lock.lock_updated = now;
        ContractLock::<T>::insert(id, contract_lock);

        Ok(contract)
    }

    pub fn _update_node_contract(
        account_id: T::AccountId,
        contract_id: u64,
        deployment_hash: types::HexHash,
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
        Self::update_contract_state(&mut contract, &state)?;

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

        Self::update_contract_state(&mut contract, &types::ContractState::Deleted(cause))?;
        Self::bill_contract(contract.contract_id)?;

        Ok(().into())
    }

    pub fn remove_contract(contract_id: u64) -> DispatchResultWithPostInfo {
        let contract = Contracts::<T>::get(contract_id).ok_or(Error::<T>::ContractNotExists)?;

        match contract.contract_type.clone() {
            types::ContractData::NodeContract(mut node_contract) => {
                if node_contract.public_ips > 0 {
                    match Self::free_ip(contract_id, &mut node_contract) {
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

    // Helper function that updates the contract state and manages storage accordingly
    pub fn update_contract_state(
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

    fn reserve_ip(
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

    fn free_ip(
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

            report.calculate_report_cost_units_usd::<T>(&pricing_policy);

            Self::deposit_event(Event::NruConsumptionReportReceived(report.clone()));
        }

        Ok(Pays::No.into())
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

impl<T: Config> PublicIpModifier for Pallet<T> {
    fn ip_removed(ip: &PublicIP) {
        if let Some(mut contract) = Contracts::<T>::get(ip.contract_id) {
            match contract.contract_type {
                types::ContractData::NodeContract(mut node_contract) => {
                    if node_contract.public_ips > 0 {
                        if let Err(e) = Self::free_ip(ip.contract_id, &mut node_contract) {
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
