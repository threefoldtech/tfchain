use crate::pallet_tfgrid;
use crate::*;
use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};
use log::info;
use sp_std::marker::PhantomData;
use scale_info::prelude::string::String;

// ✅ ContractsToBillAt
// ✅ Contracts
// ✅ ActiveNodeContracts
// ✅ ActiveRentContractForNode
// ✅ ContractIDByNodeIDAndHash
// ✅ ContractIDByNameRegistration
// ✅ ContractLock
// ✅ SolutionProviders
// ✅ ContractBillingInformationByID
// ✅ NodeContractResources

pub struct CheckStorageStateV9<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for CheckStorageStateV9<T> {
    fn on_runtime_upgrade() -> Weight {
        if PalletVersion::<T>::get() == types::StorageVersion::V9 {
            info!(
                " >>> Starting Smart Contract pallet {:?} storage check",
                PalletVersion::<T>::get()
            );
            check_pallet_smart_contract::<T>();
            clean_pallet_smart_contract::<T>();
            check_pallet_smart_contract::<T>();
        } else {
            info!(" >>> Unused Smart Contract pallet V9 storage check");
        }

        Weight::zero()
    }
}

pub fn check_pallet_smart_contract<T: Config>() -> frame_support::weights::Weight {

    // check_contracts_to_bill_at::<T>()
    // + check_contracts::<T>()
    // + check_active_node_contracts::<T>()
    // + check_active_rent_contract_for_node::<T>()
    // + check_contract_id_by_node_id_and_hash::<T>()
    // + check_contract_id_by_name_registration::<T>()
    // + check_contract_lock::<T>()
    // + check_solution_providers::<T>()
    // + check_contract_billing_information_by_id::<T>()
    // + check_node_contract_resources::<T>()

    check_contracts_to_bill_at::<T>();
    check_contracts::<T>();
    check_active_node_contracts::<T>();
    check_active_rent_contract_for_node::<T>();
    check_contract_id_by_node_id_and_hash::<T>();
    check_contract_id_by_name_registration::<T>();
    check_contract_lock::<T>();
    check_solution_providers::<T>();
    check_contract_billing_information_by_id::<T>();
    check_node_contract_resources::<T>();

    Weight::zero()
}

pub fn clean_pallet_smart_contract<T: Config>() -> frame_support::weights::Weight {
    // clean_contracts_to_bill_at::<T>()
    // + clean_contracts::<T>()
    // + clean_active_node_contracts::<T>()
    // + clean_active_rent_contract_for_node::<T>()
    // + clean_contract_id_by_node_id_and_hash::<T>()
    // + clean_contract_id_by_name_registration::<T>()
    // + clean_contract_lock::<T>()
    // + clean_solution_providers::<T>()
    // + clean_contract_billing_information_by_id::<T>()
    // + clean_node_contract_resources::<T>()

    clean_contracts_to_bill_at::<T>();
    clean_contracts::<T>();
    clean_active_node_contracts::<T>();
    clean_active_rent_contract_for_node::<T>();
    clean_contract_id_by_node_id_and_hash::<T>();
    clean_contract_id_by_name_registration::<T>();
    clean_contract_lock::<T>();
    clean_solution_providers::<T>();
    clean_contract_billing_information_by_id::<T>();
    clean_node_contract_resources::<T>();

    Weight::zero()
}

// ContractsToBillAt
pub fn check_contracts_to_bill_at<T: Config>() -> frame_support::weights::Weight {
    info!(
        "🔎  Smart Contract pallet {:?} checking ContractsToBillAt storage map START",
        PalletVersion::<T>::get()
    );

    let mut contract_id_count = vec![0; (ContractID::<T>::get() + 1) as usize];

    for (_index, contract_ids) in ContractsToBillAt::<T>::iter() {
        for contract_id in contract_ids {
            contract_id_count[contract_id as usize] += 1;
        }
    }

    // A contract id should be in billing loop only if contract still exists
    // In this case it should exactly be stored once unless it has no pub ips and no resources
    for (contract_id, count) in contract_id_count.iter().enumerate() {
        if let Some(c) = Contracts::<T>::get(contract_id as u64)  {
            match count {
                0 => {
                    if let types::ContractData::NodeContract(node_contract) = c.contract_type {
                        let contract_resource = NodeContractResources::<T>::get(contract_id as u64);
                        if node_contract.public_ips == 0 && contract_resource.used.is_empty() {
                            continue;
                        }
                    }
                    info!(
                        " ⚠️    Contract (id: {}) should be in billing loop",
                        contract_id
                    );
                }
                1 => {
                    if let types::ContractData::NodeContract(node_contract) = c.contract_type {
                        let contract_resource = NodeContractResources::<T>::get(contract_id as u64);
                        if node_contract.public_ips == 0 && contract_resource.used.is_empty() {
                            info!(
                                " ⚠️    Node Contract (id: {}) with no ips / no resources should not be in billing loop",
                                contract_id
                            );
                        }
                    }
                }
                _ => {
                    info!(
                        " ⚠️    Contract (id: {}) duplicated {} times in billing loop",
                        contract_id, count
                    );
                }
            }
        } else if count > &0 {
            info!(
                " ⚠️    Contract (id: {}) not exists and should not be in billing loop",
                contract_id
            );
        }
    }

    info!(
        "🏁  Smart Contract pallet {:?} checking ContractsToBillAt storage map END",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

// Contracts
pub fn check_contracts<T: Config>() -> frame_support::weights::Weight {
    info!(
        "🔎  Smart Contract pallet {:?} checking Contracts storage map START",
        PalletVersion::<T>::get()
    );

    let contract_id_range = 1..=ContractID::<T>::get();
    for (contract_id, contract) in Contracts::<T>::iter() {
        if contract_id != contract.contract_id {
            info!(
                " ⚠️    Contracts[id: {}]: wrong id ({})",
                contract_id, contract.contract_id
            );
        }
        if !contract_id_range.contains(&contract_id) {
            info!(
                " ⚠️    Contracts[id: {}]: id not in range {:?}",
                contract_id, contract_id_range
            );
        }

        match contract.contract_type {
            types::ContractData::NodeContract(node_contract) => check_node_contract::<T>(
                node_contract.node_id,
                contract_id,
                node_contract.deployment_hash,
            ),
            types::ContractData::NameContract(name_contract) => { check_name_contract::<T>(contract_id, &name_contract.name) }
            types::ContractData::RentContract(ref rent_contract) => {
                check_rent_contract::<T>(rent_contract.node_id, &contract)
            }
        }

        // ContractLock
        if !ContractLock::<T>::contains_key(contract_id) {
            info!(
                " ⚠️    Contract (id: {}): no contract lock found",
                contract_id
            );
        }

        // SolutionProviders
        if let Some(sp_id) = contract.solution_provider_id {
            if SolutionProviders::<T>::get(sp_id).is_none() {
                info!(
                    " ⚠️    Contract (id: {}): solution provider (id: {}) not exists",
                    contract_id, sp_id
                );
            }
        }
    }

    info!(
        "🏁  Smart Contract pallet {:?} checking Contracts storage map END",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

fn check_node_contract<T: Config>(node_id: u32, contract_id: u64, deployment_hash: HexHash) {
    let node = pallet_tfgrid::Nodes::<T>::get(node_id);
    if let Some(_) = node {
        // ActiveNodeContracts
        let active_node_contracts = ActiveNodeContracts::<T>::get(node_id);
        if !active_node_contracts.contains(&contract_id) {
            info!(
                " ⚠️    Node Contract (id: {}) on node {}: contract not in active list ({:?})",
                contract_id, node_id, active_node_contracts
            );
        }

        // ContractIDByNodeIDAndHash
        let ctr_id = ContractIDByNodeIDAndHash::<T>::get(node_id, &deployment_hash);
        if ctr_id == 0 {
            info!(
                " ⚠️    Node Contract (id: {}) on node {}: key not exists in node/deployment map",
                contract_id, node_id
            );
        } else if ctr_id != contract_id {
            info!(
                " ⚠️    Node Contract (id: {}) on node {}: wrong contract (id: {}) in node/deployment map",
                contract_id, node_id, ctr_id
            );
        }

        // ContractBillingInformationByID
        if !ContractBillingInformationByID::<T>::contains_key(contract_id) {
            info!(
                " ⚠️    Node Contract (id: {}) on node {}: no related billing information found",
                contract_id, node_id
            );
        }

        // NodeContractResources
        // Nothing to check from here 
        // A node contract needs a call to report_contract_resources() to
        // have associated ressources in NodeContractResources storage map
    } else {
        info!(
            " ⚠️    Node Contract (id: {}) on node {}: node not exists",
            contract_id, node_id
        );
    }
}

fn check_name_contract<T: Config>(contract_id: u64, name: &T::NameContractName) {
    // ContractIDByNameRegistration
    let ctr_id = ContractIDByNameRegistration::<T>::get(name);
    if ctr_id == 0 {
        info!(
            " ⚠️    Name Contract (id: {}): key (name: {:?}) not exists",
            contract_id, name
        );
    }
    else if ctr_id != contract_id  {
        info!(
            " ⚠️    Name Contract (id: {}): wrong contract (id: {}) in name registration map",
            contract_id, ctr_id
        );
    }
}

fn check_rent_contract<T: Config>(node_id: u32, contract: &types::Contract<T>) {
    let node = pallet_tfgrid::Nodes::<T>::get(node_id);
    if let Some(_) = node {
        // ActiveRentContractForNode
        let active_rent_contract = ActiveRentContractForNode::<T>::get(node_id);
        if active_rent_contract != Some(contract.contract_id) {
            info!(
                " ⚠️    Rent Contract (id: {}) on node {}: contract not activated ({:?})",
                contract.contract_id, node_id, active_rent_contract
            );
        }
        // ActiveNodeContracts
        let active_node_contracts = ActiveNodeContracts::<T>::get(node_id);
        for ctr_id in active_node_contracts {
            if let Some(node_contract) = Contracts::<T>::get(ctr_id) {
                if contract.twin_id != node_contract.twin_id {
                    info!(
                        " ⚠️    Rent Contract (id: {}) on node {}: not matching twin on node contract (id: {})",
                        contract.contract_id, node_id, ctr_id
                    );
                }
            }
        }
    } else {
        info!(
            " ⚠️    Rent Contract (id: {}) on node {}: node not exists",
            contract.contract_id, node_id
        );
    }
}

// ActiveNodeContracts
pub fn check_active_node_contracts<T: Config>() -> frame_support::weights::Weight {
    info!(
        "🔎  Smart Contract pallet {:?} checking ActiveNodeContracts storage map START",
        PalletVersion::<T>::get()
    );

    for (node_id, contract_ids) in ActiveNodeContracts::<T>::iter() {
        let node = pallet_tfgrid::Nodes::<T>::get(node_id);
        if node.is_none() {
            info!(
                " ⚠️    ActiveNodeContracts[node: {}]: node not exists",
                node_id
            );
        }

        for ctr_id in contract_ids {
            if let Some(c) = Contracts::<T>::get(ctr_id) {
                match c.contract_type {
                    types::ContractData::NodeContract(_) => (),
                    _ => {
                        info!(
                        " ⚠️    ActiveNodeContracts[node: {}, contract: {}]: type is not node contract",
                        node_id, ctr_id
                    );
                    }
                }
            } else {
                info!(
                    " ⚠️    ActiveNodeContracts[node: {}]: contract {} not exists",
                    node_id, ctr_id
                );
            }
        }
    }

    info!(
        "🏁  Smart Contract pallet {:?} checking ActiveNodeContracts storage map END",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

// ActiveRentContractForNode
pub fn check_active_rent_contract_for_node<T: Config>() -> frame_support::weights::Weight {
    info!(
        "🔎  Smart Contract pallet {:?} checking ActiveRentContractForNode storage map START",
        PalletVersion::<T>::get()
    );

    for (node_id, contract_id) in ActiveRentContractForNode::<T>::iter() {
        let node = pallet_tfgrid::Nodes::<T>::get(node_id);
        if node.is_none() {
            info!(
                " ⚠️    ActiveRentContractForNode[node: {}]: node not exists",
                node_id
            );
        }

        if let Some(c) = Contracts::<T>::get(contract_id) {
            match c.contract_type {
                types::ContractData::RentContract(_) => (),
                _ => {
                    info!(
                        " ⚠️    ActiveRentContractForNode[node: {}]: contract {} is not a rent contract",
                        node_id, contract_id
                    );
                }
            }
        } else {
            info!(
                " ⚠️    ActiveRentContractForNode[node: {}]: contract {} not exists",
                node_id, contract_id
            );
        }
    }

    info!(
        "🏁  Smart Contract pallet {:?} checking ActiveRentContractForNode storage map END",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

// ContractIDByNodeIDAndHash
pub fn check_contract_id_by_node_id_and_hash<T: Config>() -> frame_support::weights::Weight {
    info!(
        "🔎  Smart Contract pallet {:?} checking ContractIDByNodeIDAndHash storage map START",
        PalletVersion::<T>::get()
    );

    for (node_id, hash, contract_id) in ContractIDByNodeIDAndHash::<T>::iter() {
        let node = pallet_tfgrid::Nodes::<T>::get(node_id);
        if node.is_none() {
            info!(
                " ⚠️    ContractIDByNodeIDAndHash[node: {}, hash: {:?}]: node not exists",
                node_id, String::from_utf8_lossy(&hash)
            );
        }

        if let Some(c) = Contracts::<T>::get(contract_id) {
            match c.contract_type {
                types::ContractData::NodeContract(node_contract) => {
                    if node_contract.deployment_hash != hash {
                        info!(
                            " ⚠️    ContractIDByNodeIDAndHash[node: {}, hash: {:?}]: deployment hash ({:?}) on contract {} is not matching",
                            node_id, String::from_utf8_lossy(&hash), String::from_utf8_lossy(&node_contract.deployment_hash), contract_id, 
                        );
                    }
                }
                _ => {
                    info!(
                        " ⚠️    ContractIDByNodeIDAndHash[node: {}, hash: {:?}]: contract {} is not a node contract",
                        node_id, String::from_utf8_lossy(&hash), contract_id
                    );
                }
            }
        } else {
            info!(
                " ⚠️    ContractIDByNodeIDAndHash[node: {}, hash: {:?}]: contract {} not exists",
                node_id, String::from_utf8_lossy(&hash), contract_id
            );
        }
    }

    info!(
        "🏁  Smart Contract pallet {:?} checking ContractIDByNodeIDAndHash storage map END",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

// ContractIDByNameRegistration
pub fn check_contract_id_by_name_registration<T: Config>() -> frame_support::weights::Weight {
    info!(
        "🔎  Smart Contract pallet {:?} checking ContractIDByNameRegistration storage map START",
        PalletVersion::<T>::get()
    );

    for (name, contract_id) in ContractIDByNameRegistration::<T>::iter() {
        if let Some(c) = Contracts::<T>::get(contract_id) {
            match c.contract_type {
                types::ContractData::NameContract(name_contract) => {
                    if name_contract.name != name {
                        info!(
                            " ⚠️    ContractIDByNameRegistration[name: {:?}]: name ({:?}) on contract {} is not matching",
                            String::from_utf8_lossy(&name.into()), String::from_utf8_lossy(&name_contract.name.into()), contract_id, 
                        );
                    }
                }
                _ => {
                    info!(
                        " ⚠️    ContractIDByNameRegistration[name: {:?}]: contract {} is not a name contract",
                        String::from_utf8_lossy(&name.into()), contract_id
                    );
                }
            }
        } else {
            info!(
                " ⚠️    ContractIDByNameRegistration[name: {:?}]: contract {} not exists",
                String::from_utf8_lossy(&name.into()), contract_id
            );
        }
    }

    info!(
        "🏁  Smart Contract pallet {:?} checking ContractIDByNameRegistration storage map END",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

// ContractLock
pub fn check_contract_lock<T: Config>() -> frame_support::weights::Weight {
    info!(
        "🔎  Smart Contract pallet {:?} checking ContractLock storage map START",
        PalletVersion::<T>::get()
    );

    for (contract_id, _contract_lock) in ContractLock::<T>::iter() {
        if Contracts::<T>::get(contract_id).is_none() {        
            info!(
                " ⚠️    ContractLock[contract: {}]: contract not exists",
                contract_id
            );
        }
    }

    info!(
        "🏁  Smart Contract pallet {:?} checking ContractLock storage map END",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

// SolutionProviders
pub fn check_solution_providers<T: Config>() -> frame_support::weights::Weight {
    info!(
        "🔎  Smart Contract pallet {:?} checking SolutionProviders storage map START",
        PalletVersion::<T>::get()
    );

    let solution_provider_id_range = 1..=SolutionProviderID::<T>::get();
    for (solution_provider_id, _solution_provider) in SolutionProviders::<T>::iter() {
        if !solution_provider_id_range.contains(&solution_provider_id) {
            info!(
                " ⚠️    SolutionProviders[id: {}]: id not in range {:?}",
                solution_provider_id, solution_provider_id_range
            );
        }
    }

    info!(
        "🏁  Smart Contract pallet {:?} checking SolutionProviders storage map END",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

// ContractBillingInformationByID
pub fn check_contract_billing_information_by_id<T: Config>() -> frame_support::weights::Weight {
    info!(
        "🔎  Smart Contract pallet {:?} checking ContractBillingInformationByID storage map START",
        PalletVersion::<T>::get()
    );

    for (contract_id, _contract_billing_information) in ContractBillingInformationByID::<T>::iter() {
        if let Some(c) = Contracts::<T>::get(contract_id) {
            match c.contract_type {
                types::ContractData::NodeContract(_) => (),
                _ => {
                    info!(
                        " ⚠️    ContractBillingInformationByID[contract: {}]: contract is not a node contract",
                        contract_id
                    );
                }
            }
        } else {
            info!(
                " ⚠️    ContractBillingInformationByID[contract: {}]: contract not exists",
                contract_id
            );
        }
    }

    info!(
        "🏁  Smart Contract pallet {:?} checking ContractBillingInformationByID storage map END",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

// NodeContractResources
pub fn check_node_contract_resources<T: Config>() -> frame_support::weights::Weight {
    info!(
        "🔎  Smart Contract pallet {:?} checking NodeContractResources storage map START",
        PalletVersion::<T>::get()
    );

    for (contract_id, contract_resource) in NodeContractResources::<T>::iter() { 
        if contract_resource.contract_id != contract_id {
            info!(
                " ⚠️    NodeContractResources[contract: {}]: wrong contract id on resource ({})",
               contract_id, contract_resource.contract_id
            );
        }

        if let Some(c) = Contracts::<T>::get(contract_id) {
            match c.contract_type {
                types::ContractData::NodeContract(_) => (),
                _ => {
                    info!(
                        " ⚠️    NodeContractResources[contract: {}]: contract is not a node contract",
                        contract_id
                    );
                }
            }
        } else {
            info!(
                " ⚠️    NodeContractResources[contract: {}]: contract not exists",
                contract_id
            );
        }
    }

    info!(
        "🏁  Smart Contract pallet {:?} checking NodeContractResources storage map END",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

// ContractsToBillAt
pub fn clean_contracts_to_bill_at<T: Config>() -> frame_support::weights::Weight {
    info!(
        "🧼  Smart Contract pallet {:?} cleaning ContractsToBillAt storage map START",
        PalletVersion::<T>::get()
    );

    // Collect ContractsToBillAt storage in memory
    let contracts_to_bill_at = ContractsToBillAt::<T>::iter().collect::<Vec<_>>();
    let mut contract_id_count = vec![0; (ContractID::<T>::get() + 1) as usize];

    for (_index, contract_ids) in contracts_to_bill_at.clone() {
        for contract_id in contract_ids {
            contract_id_count[contract_id as usize] += 1;
        }
    }

    for (index, contract_ids) in contracts_to_bill_at {
        let mut new_contract_ids = Vec::new();
        for contract_id in contract_ids {
            if let Some(contract) = Contracts::<T>::get(contract_id) {
                match contract.contract_type {
                    types::ContractData::NodeContract(node_contract) => {
                        let contract_resource = NodeContractResources::<T>::get(contract_id);
                        if node_contract.public_ips > 0 || !contract_resource.used.is_empty() {
                            if contract_id_count[contract_id as usize] == 1 {
                                new_contract_ids.push(contract.contract_id);
                            }
                            contract_id_count[contract_id as usize] -= 1;
                        }
                    }
                    _ => {
                        if contract_id_count[contract_id as usize] == 1 {
                            new_contract_ids.push(contract.contract_id);
                        }
                        contract_id_count[contract_id as usize] -= 1;
                    }
                };
            }
        }
        ContractsToBillAt::<T>::insert(index, new_contract_ids);
    }

    info!(
        "✨  Smart Contract pallet {:?} cleaning ContractsToBillAt storage map END",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

// Contracts
pub fn clean_contracts<T: Config>() -> frame_support::weights::Weight {
    info!(
        "🧼  Smart Contract pallet {:?} cleaning Contracts storage map START",
        PalletVersion::<T>::get()
    );

    for (contract_id, contract) in Contracts::<T>::iter() {
        match contract.contract_type {
            types::ContractData::NodeContract(node_contract) => clean_node_contract::<T>(
                node_contract.node_id,
                contract_id,
                node_contract.deployment_hash,
            ),
            types::ContractData::NameContract(_) => clean_name_contract::<T>(),
            types::ContractData::RentContract(_) => clean_rent_contract::<T>(),
        }
    }

    info!(
        "✨  Smart Contract pallet {:?} cleaning Contracts storage map END",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

fn clean_node_contract<T: Config>(node_id: u32, contract_id: u64, deployment_hash: HexHash) {
    // ActiveNodeContracts
    let mut active_node_contracts = ActiveNodeContracts::<T>::get(node_id);
    if !active_node_contracts.contains(&contract_id) {
        active_node_contracts.push(contract_id);
        ActiveNodeContracts::<T>::insert(node_id, active_node_contracts);
    }

    // ContractIDByNodeIDAndHash
    if !ContractIDByNodeIDAndHash::<T>::contains_key(node_id, deployment_hash) {
        ContractIDByNodeIDAndHash::<T>::insert(node_id, deployment_hash, contract_id);
    }
}

fn clean_name_contract<T: Config>() {
    // Nothing to do here
}

fn clean_rent_contract<T: Config>() {
    // Nothing to do here
}

// ActiveNodeContracts
pub fn clean_active_node_contracts<T: Config>() -> frame_support::weights::Weight {
    info!(
        "🧼  Smart Contract pallet {:?} cleaning ActiveNodeContracts storage map START",
        PalletVersion::<T>::get()
    );

    // Collect ActiveNodeContracts storage in memory
    let active_node_contracts = ActiveNodeContracts::<T>::iter().collect::<Vec<_>>();
    for (node_id, mut contract_ids) in active_node_contracts {
        if pallet_tfgrid::Nodes::<T>::get(node_id).is_none() {
            ActiveNodeContracts::<T>::remove(node_id);
        } else {
            contract_ids.retain(|contract_id| Contracts::<T>::get(contract_id).is_some());
            ActiveNodeContracts::<T>::insert(node_id, contract_ids);
        }
    }

    info!(
        "✨  Smart Contract pallet {:?} cleaning ActiveNodeContracts storage map END",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

// ActiveRentContractForNode
pub fn clean_active_rent_contract_for_node<T: Config>() -> frame_support::weights::Weight {
    info!(
        "🧼  Smart Contract pallet {:?} cleaning ActiveRentContractForNode storage map START",
        PalletVersion::<T>::get()
    );

    let to_remove: Vec<_> = ActiveRentContractForNode::<T>::iter()
        .filter(|(_, contract_id)| Contracts::<T>::get(contract_id).is_none())
        .map(|(node_id, _)| node_id)
        .collect();

    for node_id in to_remove {
        ActiveRentContractForNode::<T>::remove(node_id);
    }

    info!(
        "✨  Smart Contract pallet {:?} cleaning ActiveRentContractForNode storage map END",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

// ContractIDByNodeIDAndHash
pub fn clean_contract_id_by_node_id_and_hash<T: Config>() -> frame_support::weights::Weight {
    info!(
        "🧼  Smart Contract pallet {:?} cleaning ContractIDByNodeIDAndHash storage map START",
        PalletVersion::<T>::get()
    );

    let to_remove: Vec<_> = ContractIDByNodeIDAndHash::<T>::iter()
        .filter(|(_, _, contract_id)| Contracts::<T>::get(contract_id).is_none())
        .map(|(node_id, hash, _)| (node_id, hash))
        .collect();

    info!("   contract ids to remove: {:?}", to_remove);

    for (node_id, hash) in to_remove {
        info!("   contract id to remove: [node: {}, hash: {:?}]", node_id, String::from_utf8_lossy(&hash));
        ContractIDByNodeIDAndHash::<T>::remove(node_id, hash);
    }

    // for (node_id, hash, contract_id) in ContractIDByNodeIDAndHash::<T>::iter() {
    //     let node = pallet_tfgrid::Nodes::<T>::get(node_id);
    //     if node.is_none() {
    //         info!(
    //             " ⚠️    ContractIDByNodeIDAndHash[node: {}, hash: {:?}]: node not exists",
    //             node_id, String::from_utf8_lossy(&hash)
    //         );
    //     }

    //     let contract = Contracts::<T>::get(contract_id);
    //     if let Some(c) = contract {
    //         match c.contract_type {
    //             types::ContractData::NodeContract(node_contract) => {
    //                 if node_contract.deployment_hash != hash {
    //                     info!(
    //                         " ⚠️    ContractIDByNodeIDAndHash[node: {}, hash: {:?}]: deployment hash ({:?}) on contract {} is not matching",
    //                         node_id, String::from_utf8_lossy(&hash), String::from_utf8_lossy(&node_contract.deployment_hash), contract_id, 
    //                     );
    //                 }
    //             }
    //             _ => {
    //                 info!(
    //                     " ⚠️    ContractIDByNodeIDAndHash[node: {}, hash: {:?}]: contract {} is not a node contract",
    //                     node_id, String::from_utf8_lossy(&hash), contract_id
    //                 );
    //             }
    //         }
    //     } else {
    //         info!(
    //             " ⚠️    ContractIDByNodeIDAndHash[node: {}, hash: {:?}]: contract {} not exists",
    //             node_id, String::from_utf8_lossy(&hash), contract_id
    //         );
    //     }
    // }

    info!(
        "✨  Smart Contract pallet {:?} cleaning ContractIDByNodeIDAndHash storage map END",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

// ContractIDByNameRegistration
pub fn clean_contract_id_by_name_registration<T: Config>() -> frame_support::weights::Weight {
    info!(
        "🧼  Smart Contract pallet {:?} cleaning ContractIDByNameRegistration storage map START",
        PalletVersion::<T>::get()
    );

    let to_remove: Vec<_> = ContractIDByNameRegistration::<T>::iter()
        .filter(|(_, contract_id)| Contracts::<T>::get(contract_id).is_none())
        .map(|(name, _)| name)
        .collect();

    for contract_id in to_remove {
        ContractIDByNameRegistration::<T>::remove(contract_id);
    }

    info!(
        "✨  Smart Contract pallet {:?} cleaning ContractIDByNameRegistration storage map END",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

// ContractLock
pub fn clean_contract_lock<T: Config>() -> frame_support::weights::Weight {
    info!(
        "🧼  Smart Contract pallet {:?} cleaning ContractLock storage map START",
        PalletVersion::<T>::get()
    );

    // Nothing to do here

    info!(
        "✨  Smart Contract pallet {:?} cleaning ContractLock storage map END",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

// SolutionProviders
pub fn clean_solution_providers<T: Config>() -> frame_support::weights::Weight {
    info!(
        "🧼  Smart Contract pallet {:?} cleaning SolutionProviders storage map START",
        PalletVersion::<T>::get()
    );

    // Nothing to do here

    info!(
        "✨  Smart Contract pallet {:?} cleaning SolutionProviders storage map END",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

// ContractBillingInformationByID
pub fn clean_contract_billing_information_by_id<T: Config>() -> frame_support::weights::Weight {
    info!(
        "🧼  Smart Contract pallet {:?} cleaning ContractBillingInformationByID storage map START",
        PalletVersion::<T>::get()
    );

    let to_remove: Vec<u64> = ContractBillingInformationByID::<T>::iter()
        .filter(|(contract_id, _)| {
            if let Some(c) = Contracts::<T>::get(contract_id) {
                match c.contract_type {
                    types::ContractData::NodeContract(_) => false,
                    _ => true,
                }
            } else {
                true
            }
        })
        .map(|(id, _)| id)
        .collect();

    for contract_id in to_remove {
        ContractBillingInformationByID::<T>::remove(contract_id); 
    }

    info!(
        "✨  Smart Contract pallet {:?} cleaning ContractBillingInformationByID storage map END",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

// NodeContractResources
pub fn clean_node_contract_resources<T: Config>() -> frame_support::weights::Weight {
    info!(
        "🧼  Smart Contract pallet {:?} cleaning NodeContractResources storage map START",
        PalletVersion::<T>::get()
    );

    let to_remove: Vec<u64> = NodeContractResources::<T>::iter()
        .filter(|(contract_id, _)| Contracts::<T>::get(contract_id).is_none())
        .map(|(id, _)| id)
        .collect();

    for contract_id in to_remove {
        NodeContractResources::<T>::remove(contract_id);
    }

    info!(
        "✨  Smart Contract pallet {:?} cleaning NodeContractResources storage map END",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}