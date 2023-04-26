use crate::pallet_tfgrid;
use crate::*;
use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};
use log::{info, debug};
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

pub struct CheckStorageStateV8<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for CheckStorageStateV8<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        assert!(PalletVersion::<T>::get() == types::StorageVersion::V8 || PalletVersion::<T>::get() == types::StorageVersion::V9);

        check_pallet_smart_contract::<T>();

        Ok(vec![])
    }

    fn on_runtime_upgrade() -> Weight {
        if PalletVersion::<T>::get() == types::StorageVersion::V8 ||
         PalletVersion::<T>::get() == types::StorageVersion::V9 {
            info!(
                " >>> Starting Smart Contract pallet {:?} storage cleaning",
                PalletVersion::<T>::get()
            );
            let weight = clean_pallet_smart_contract::<T>();
            PalletVersion::<T>::put(types::StorageVersion::V9);
            return weight;
        } else {
            info!(" >>> Unused Smart Contract pallet V8 storage cleaning");
            Weight::zero()
        }    
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_: Vec<u8>) -> Result<(), &'static str> {
        assert!(PalletVersion::<T>::get() == types::StorageVersion::V9);

        check_pallet_smart_contract::<T>();

        info!(
            "👥  Smart Contract pallet to {:?} passes POST migrate checks ✅",
            PalletVersion::<T>::get()
        );

        Ok(())
    }
}

pub fn check_pallet_smart_contract<T: Config>() {
    debug!("💥💥💥💥💥💥💥💥💥💥 CHECKING PALLET SMART CONTRACT STORAGE 💥💥💥💥💥💥💥💥💥💥");
    check_contracts::<T>();
    check_contracts_to_bill_at::<T>();
    check_active_node_contracts::<T>();
    check_active_rent_contract_for_node::<T>();
    check_contract_id_by_node_id_and_hash::<T>();
    check_contract_id_by_name_registration::<T>();
    check_contract_lock::<T>();
    check_solution_providers::<T>();
    check_contract_billing_information_by_id::<T>();
    check_node_contract_resources::<T>();
}

pub fn clean_pallet_smart_contract<T: Config>() -> frame_support::weights::Weight {
    debug!("💥💥💥💥💥💥💥💥💥💥 CLEANING PALLET SMART CONTRACT STORAGE 💥💥💥💥💥💥💥💥💥💥");
    clean_contracts::<T>() +
    clean_contracts_to_bill_at::<T>() +
    clean_active_node_contracts::<T>() +
    clean_active_rent_contract_for_node::<T>() +
    clean_contract_id_by_node_id_and_hash::<T>() +
    clean_contract_id_by_name_registration::<T>() +
    clean_contract_lock::<T>() +
    clean_solution_providers::<T>() +
    clean_contract_billing_information_by_id::<T>() +
    clean_node_contract_resources::<T>()
}

// Contracts
pub fn check_contracts<T: Config>() {
    debug!(
        "🔎  Smart Contract pallet {:?} checking Contracts storage map START",
        PalletVersion::<T>::get()
    );

    let contract_id_range = 1..=ContractID::<T>::get();

    for (contract_id, contract) in Contracts::<T>::iter() {
        if contract_id != contract.contract_id {
            debug!(
                " ⚠️    Contracts[id: {}]: wrong id ({})",
                contract_id, contract.contract_id
            );
        }
        if !contract_id_range.contains(&contract_id) {
            debug!(
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
            types::ContractData::NameContract(name_contract) => {
                check_name_contract::<T>(contract_id, &name_contract.name)
            }
            types::ContractData::RentContract(ref rent_contract) => {
                check_rent_contract::<T>(rent_contract.node_id, &contract)
            }
        }

        // ContractLock
        if !ContractLock::<T>::contains_key(contract_id) {
            debug!(
                " ⚠️    Contract (id: {}): no contract lock found",
                contract_id
            );
        }

        // SolutionProviders
        if let Some(sp_id) = contract.solution_provider_id {
            if SolutionProviders::<T>::get(sp_id).is_none() {
                debug!(
                    " ⚠️    Contract (id: {}): solution provider (id: {}) not exists",
                    contract_id, sp_id
                );
            }
        }
    }

    debug!(
        "🏁  Smart Contract pallet {:?} checking Contracts storage map END",
        PalletVersion::<T>::get()
    );
}

fn check_node_contract<T: Config>(node_id: u32, contract_id: u64, deployment_hash: HexHash) {
    if let Some(_) = pallet_tfgrid::Nodes::<T>::get(node_id) {
        // ActiveNodeContracts
        let active_node_contracts = ActiveNodeContracts::<T>::get(node_id);
        if !active_node_contracts.contains(&contract_id) {
            debug!(
                " ⚠️    Node Contract (id: {}) on node {}: contract not in active list ({:?})",
                contract_id, node_id, active_node_contracts
            );
        }

        // ContractIDByNodeIDAndHash
        let ctr_id = ContractIDByNodeIDAndHash::<T>::get(node_id, &deployment_hash);
        if ctr_id == 0 {
            debug!(
                " ⚠️    Node Contract (id: {}) on node {}: key not exists in node/deployment map",
                contract_id, node_id
            );
        } else if ctr_id != contract_id {
            debug!(
                " ⚠️    Node Contract (id: {}) on node {}: wrong contract (id: {}) in node/deployment map",
                contract_id, node_id, ctr_id
            );
        }

        // ContractBillingInformationByID
        if !ContractBillingInformationByID::<T>::contains_key(contract_id) {
            debug!(
                " ⚠️    Node Contract (id: {}) on node {}: no related billing information found",
                contract_id, node_id
            );
        }

        // NodeContractResources
        // Nothing to check here 
        // A node contract needs a call to report_contract_resources() to
        // have associated ressources in NodeContractResources storage map
    } else {
        debug!(
            " ⚠️    Node Contract (id: {}) on node {}: node not exists",
            contract_id, node_id
        );
    }

    if deployment_hash == HexHash::default() {
        debug!(
            " ⚠️    Node Contract (id: {}) on node {}: deployment hash is default ({:?})",
            contract_id, node_id, String::from_utf8_lossy(&deployment_hash)
        );
    }
}

fn check_name_contract<T: Config>(contract_id: u64, name: &T::NameContractName) {
    // ContractIDByNameRegistration
    let ctr_id = ContractIDByNameRegistration::<T>::get(name);
    if ctr_id == 0 {
        debug!(
            " ⚠️    Name Contract (id: {}): key (name: {:?}) not exists",
            contract_id, name
        );
    }
    else if ctr_id != contract_id  {
        debug!(
            " ⚠️    Name Contract (id: {}): wrong contract (id: {}) in name registration map",
            contract_id, ctr_id
        );
    }
}

fn check_rent_contract<T: Config>(node_id: u32, contract: &types::Contract<T>) {
    if let Some(_) = pallet_tfgrid::Nodes::<T>::get(node_id) {
        // ActiveRentContractForNode
        let active_rent_contract = ActiveRentContractForNode::<T>::get(node_id);
        if active_rent_contract != Some(contract.contract_id) {
            debug!(
                " ⚠️    Rent Contract (id: {}) on node {}: contract not activated ({:?})",
                contract.contract_id, node_id, active_rent_contract
            );
        }
        // ActiveNodeContracts
        let active_node_contracts = ActiveNodeContracts::<T>::get(node_id);
        for ctr_id in active_node_contracts {
            if let Some(node_contract) = Contracts::<T>::get(ctr_id) {
                if contract.twin_id != node_contract.twin_id {
                    debug!(
                        " ⚠️    Rent Contract (id: {}) on node {}: not matching twin on node contract (id: {})",
                        contract.contract_id, node_id, ctr_id
                    );
                }
            }
        }
    } else {
        debug!(
            " ⚠️    Rent Contract (id: {}) on node {}: node not exists",
            contract.contract_id, node_id
        );
    }
}

// ContractsToBillAt
pub fn check_contracts_to_bill_at<T: Config>() {
    debug!(
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
                    debug!(
                        " ⚠️    Contract (id: {}) should be in billing loop",
                        contract_id
                    );
                }
                1 => {
                    if let types::ContractData::NodeContract(node_contract) = c.contract_type {
                        let contract_resource = NodeContractResources::<T>::get(contract_id as u64);
                        if node_contract.public_ips == 0 && contract_resource.used.is_empty() {
                            debug!(
                                " ⚠️    Node Contract (id: {}) with no ips / no resources should not be in billing loop",
                                contract_id
                            );
                        }
                    }
                }
                _ => {
                    debug!(
                        " ⚠️    Contract (id: {}) duplicated {} times in billing loop",
                        contract_id, count
                    );
                }
            }
        } else if count > &0 {
            debug!(
                " ⚠️    Contract (id: {}) not exists and should not be in billing loop",
                contract_id
            );
        }
    }

    debug!(
        "🏁  Smart Contract pallet {:?} checking ContractsToBillAt storage map END",
        PalletVersion::<T>::get()
    );
}

// ActiveNodeContracts
pub fn check_active_node_contracts<T: Config>() {
    debug!(
        "🔎  Smart Contract pallet {:?} checking ActiveNodeContracts storage map START",
        PalletVersion::<T>::get()
    );

    for (node_id, contract_ids) in ActiveNodeContracts::<T>::iter() {
        if pallet_tfgrid::Nodes::<T>::get(node_id).is_none() {
            debug!(
                " ⚠️    ActiveNodeContracts[node: {}]: node not exists",
                node_id
            );
        }

        for ctr_id in contract_ids {
            if let Some(c) = Contracts::<T>::get(ctr_id) {
                match c.contract_type {
                    types::ContractData::NodeContract(_) => (),
                    _ => {
                        debug!(
                        " ⚠️    ActiveNodeContracts[node: {}, contract: {}]: type is not node contract",
                        node_id, ctr_id
                    );
                    }
                }
            } else {
                debug!(
                    " ⚠️    ActiveNodeContracts[node: {}]: contract {} not exists",
                    node_id, ctr_id
                );
            }
        }
    }

    debug!(
        "🏁  Smart Contract pallet {:?} checking ActiveNodeContracts storage map END",
        PalletVersion::<T>::get()
    );
}

// ActiveRentContractForNode
pub fn check_active_rent_contract_for_node<T: Config>() {
    debug!(
        "🔎  Smart Contract pallet {:?} checking ActiveRentContractForNode storage map START",
        PalletVersion::<T>::get()
    );

    for (node_id, contract_id) in ActiveRentContractForNode::<T>::iter() {
        if pallet_tfgrid::Nodes::<T>::get(node_id).is_none() {
            debug!(
                " ⚠️    ActiveRentContractForNode[node: {}]: node not exists",
                node_id
            );
        }

        if let Some(c) = Contracts::<T>::get(contract_id) {
            match c.contract_type {
                types::ContractData::RentContract(_) => (),
                _ => {
                    debug!(
                        " ⚠️    ActiveRentContractForNode[node: {}]: contract {} is not a rent contract",
                        node_id, contract_id
                    );
                }
            }
        } else {
            debug!(
                " ⚠️    ActiveRentContractForNode[node: {}]: contract {} not exists",
                node_id, contract_id
            );
        }
    }

    debug!(
        "🏁  Smart Contract pallet {:?} checking ActiveRentContractForNode storage map END",
        PalletVersion::<T>::get()
    );
}

// ContractIDByNodeIDAndHash
pub fn check_contract_id_by_node_id_and_hash<T: Config>() {
    debug!(
        "🔎  Smart Contract pallet {:?} checking ContractIDByNodeIDAndHash storage map START",
        PalletVersion::<T>::get()
    );

    for (node_id, hash, contract_id) in ContractIDByNodeIDAndHash::<T>::iter() {
        if pallet_tfgrid::Nodes::<T>::get(node_id).is_none() {
            debug!(
                " ⚠️    ContractIDByNodeIDAndHash[node: {}, hash: {:?}]: node not exists",
                node_id, String::from_utf8_lossy(&hash)
            );
        }

        if let Some(c) = Contracts::<T>::get(contract_id) {
            match c.contract_type {
                types::ContractData::NodeContract(node_contract) => {
                    if node_contract.deployment_hash != hash {
                        debug!(
                            " ⚠️    ContractIDByNodeIDAndHash[node: {}, hash: {:?}]: deployment hash ({:?}) on contract {} is not matching",
                            node_id, String::from_utf8_lossy(&hash), String::from_utf8_lossy(&node_contract.deployment_hash), contract_id, 
                        );
                    }
                }
                _ => {
                    debug!(
                        " ⚠️    ContractIDByNodeIDAndHash[node: {}, hash: {:?}]: contract {} is not a node contract",
                        node_id, String::from_utf8_lossy(&hash), contract_id
                    );
                }
            }
        } else {
            debug!(
                " ⚠️    ContractIDByNodeIDAndHash[node: {}, hash: {:?}]: contract {} not exists",
                node_id, String::from_utf8_lossy(&hash), contract_id
            );
        }
    }

    debug!(
        "🏁  Smart Contract pallet {:?} checking ContractIDByNodeIDAndHash storage map END",
        PalletVersion::<T>::get()
    );
}

// ContractIDByNameRegistration
pub fn check_contract_id_by_name_registration<T: Config>() {
    debug!(
        "🔎  Smart Contract pallet {:?} checking ContractIDByNameRegistration storage map START",
        PalletVersion::<T>::get()
    );

    for (name, contract_id) in ContractIDByNameRegistration::<T>::iter() {
        if let Some(c) = Contracts::<T>::get(contract_id) {
            match c.contract_type {
                types::ContractData::NameContract(name_contract) => {
                    if name_contract.name != name {
                        debug!(
                            " ⚠️    ContractIDByNameRegistration[name: {:?}]: name ({:?}) on contract {} is not matching",
                            String::from_utf8_lossy(&name.into()), String::from_utf8_lossy(&name_contract.name.into()), contract_id, 
                        );
                    }
                }
                _ => {
                    debug!(
                        " ⚠️    ContractIDByNameRegistration[name: {:?}]: contract {} is not a name contract",
                        String::from_utf8_lossy(&name.into()), contract_id
                    );
                }
            }
        } else {
            debug!(
                " ⚠️    ContractIDByNameRegistration[name: {:?}]: contract {} not exists",
                String::from_utf8_lossy(&name.into()), contract_id
            );
        }
    }

    debug!(
        "🏁  Smart Contract pallet {:?} checking ContractIDByNameRegistration storage map END",
        PalletVersion::<T>::get()
    );
}

// ContractLock
pub fn check_contract_lock<T: Config>() {
    debug!(
        "🔎  Smart Contract pallet {:?} checking ContractLock storage map START",
        PalletVersion::<T>::get()
    );

    for (contract_id, _contract_lock) in ContractLock::<T>::iter() {
        if Contracts::<T>::get(contract_id).is_none() {        
            debug!(
                " ⚠️    ContractLock[contract: {}]: contract not exists",
                contract_id
            );
        }
    }

    debug!(
        "🏁  Smart Contract pallet {:?} checking ContractLock storage map END",
        PalletVersion::<T>::get()
    );
}

// SolutionProviders
pub fn check_solution_providers<T: Config>() {
    debug!(
        "🔎  Smart Contract pallet {:?} checking SolutionProviders storage map START",
        PalletVersion::<T>::get()
    );

    let solution_provider_id_range = 1..=SolutionProviderID::<T>::get();

    for (solution_provider_id, _solution_provider) in SolutionProviders::<T>::iter() {
        if !solution_provider_id_range.contains(&solution_provider_id) {
            debug!(
                " ⚠️    SolutionProviders[id: {}]: id not in range {:?}",
                solution_provider_id, solution_provider_id_range
            );
        }
    }

    debug!(
        "🏁  Smart Contract pallet {:?} checking SolutionProviders storage map END",
        PalletVersion::<T>::get()
    );
}

// ContractBillingInformationByID
pub fn check_contract_billing_information_by_id<T: Config>() {
    debug!(
        "🔎  Smart Contract pallet {:?} checking ContractBillingInformationByID storage map START",
        PalletVersion::<T>::get()
    );

    for (contract_id, _contract_billing_information) in ContractBillingInformationByID::<T>::iter() {
        if let Some(c) = Contracts::<T>::get(contract_id) {
            match c.contract_type {
                types::ContractData::NodeContract(_) => (),
                _ => {
                    debug!(
                        " ⚠️    ContractBillingInformationByID[contract: {}]: contract is not a node contract",
                        contract_id
                    );
                }
            }
        } else {
            debug!(
                " ⚠️    ContractBillingInformationByID[contract: {}]: contract not exists",
                contract_id
            );
        }
    }

    debug!(
        "🏁  Smart Contract pallet {:?} checking ContractBillingInformationByID storage map END",
        PalletVersion::<T>::get()
    );
}

// NodeContractResources
pub fn check_node_contract_resources<T: Config>() {
    debug!(
        "🔎  Smart Contract pallet {:?} checking NodeContractResources storage map START",
        PalletVersion::<T>::get()
    );

    for (contract_id, contract_resource) in NodeContractResources::<T>::iter() {
        if contract_resource.contract_id != contract_id {
            debug!(
                " ⚠️    NodeContractResources[contract: {}]: wrong contract id on resource ({})",
               contract_id, contract_resource.contract_id
            );
        }

        if let Some(c) = Contracts::<T>::get(contract_id) {
            match c.contract_type {
                types::ContractData::NodeContract(_) => (),
                _ => {
                    debug!(
                        " ⚠️    NodeContractResources[contract: {}]: contract is not a node contract",
                        contract_id
                    );
                }
            }
        } else {
            debug!(
                " ⚠️    NodeContractResources[contract: {}]: contract not exists",
                contract_id
            );
        }
    }

    debug!(
        "🏁  Smart Contract pallet {:?} checking NodeContractResources storage map END",
        PalletVersion::<T>::get()
    );
}

// Contracts
pub fn clean_contracts<T: Config>() -> frame_support::weights::Weight {
    debug!(
        "🧼  Smart Contract pallet {:?} cleaning Contracts storage map START",
        PalletVersion::<T>::get()
    );

    let mut r: u64 = 0;
    let mut w: u64 = 0;

    let contracts = Contracts::<T>::iter().collect::<Vec<_>>();
    r += contracts.len() as u64;

    for (contract_id, contract) in contracts.into_iter() {
        match contract.contract_type {
            types::ContractData::NodeContract(node_contract) => clean_node_contract::<T>(
                node_contract.node_id,
                contract_id,
                node_contract.deployment_hash,
                &mut r,
                &mut w,
            ),
            types::ContractData::NameContract(_) => clean_name_contract::<T>(),
            types::ContractData::RentContract(_) => clean_rent_contract::<T>(),
        }

        // ContractLock
        if !ContractLock::<T>::contains_key(contract_id) {
            let now = <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;
            r += 1;
            let mut contract_lock = types::ContractLock::default();
            contract_lock.lock_updated = now;
            ContractLock::<T>::insert(contract_id, contract_lock);
            w += 1;
        }
        r += 1;
    }

    debug!(
        "✨  Smart Contract pallet {:?} cleaning Contracts storage map END",
        PalletVersion::<T>::get()
    );

    T::DbWeight::get().reads_writes(r + 2, w)
}

fn clean_node_contract<T: Config>(node_id: u32, contract_id: u64, deployment_hash: HexHash, r: &mut u64, w: &mut u64) {
    if deployment_hash == HexHash::default() {
        Contracts::<T>::remove(contract_id);
        *w += 1;
    }

    // ActiveNodeContracts
    let mut active_node_contracts = ActiveNodeContracts::<T>::get(node_id);
    *r += 1;
    if !active_node_contracts.contains(&contract_id) {
        active_node_contracts.push(contract_id);
        ActiveNodeContracts::<T>::insert(node_id, active_node_contracts);
        *w += 1;
    }

    // ContractIDByNodeIDAndHash
    // Nothing to do here
    // Storage re-built from zero in clean_contract_id_by_node_id_and_hash()
    // if !ContractIDByNodeIDAndHash::<T>::contains_key(node_id, deployment_hash) {
    //     ContractIDByNodeIDAndHash::<T>::insert(node_id, deployment_hash, contract_id);
    //     *w += 1;
    // }
    // *r += 1;
}

fn clean_name_contract<T: Config>() {
    // Nothing to do here
}

fn clean_rent_contract<T: Config>() {
    // Nothing to do here
}

// ContractsToBillAt
pub fn clean_contracts_to_bill_at<T: Config>() -> frame_support::weights::Weight {
    debug!(
        "🧼  Smart Contract pallet {:?} cleaning ContractsToBillAt storage map START",
        PalletVersion::<T>::get()
    );

    let mut r: u64 = 0;
    let mut w: u64 = 0;

    let contracts_to_bill_at = ContractsToBillAt::<T>::iter().collect::<Vec<_>>();
    r += contracts_to_bill_at.len() as u64;
    let mut contract_id_count = vec![0; (ContractID::<T>::get() + 1) as usize];
    r += 1;

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
                        r += 1;
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
            r += 1;
        }
        ContractsToBillAt::<T>::insert(index, new_contract_ids);
        w += 1;
    }

    debug!(
        "✨  Smart Contract pallet {:?} cleaning ContractsToBillAt storage map END",
        PalletVersion::<T>::get()
    );

    T::DbWeight::get().reads_writes(r + 2, w)
}

// ActiveNodeContracts
pub fn clean_active_node_contracts<T: Config>() -> frame_support::weights::Weight {
    debug!(
        "🧼  Smart Contract pallet {:?} cleaning ActiveNodeContracts storage map START",
        PalletVersion::<T>::get()
    );

    let mut r: u64 = 0;
    let mut w: u64 = 0;

    let active_node_contracts = ActiveNodeContracts::<T>::iter().collect::<Vec<_>>();
    r += active_node_contracts.len() as u64;

    for (node_id, mut contract_ids) in active_node_contracts {
        if pallet_tfgrid::Nodes::<T>::get(node_id).is_none() {
            ActiveNodeContracts::<T>::remove(node_id);
            w += 1;
        } else {
            contract_ids.retain(|contract_id| Contracts::<T>::get(contract_id).is_some());
            r += contract_ids.len() as u64;
            ActiveNodeContracts::<T>::insert(node_id, contract_ids);
            w += 1;
        }
        r += 1;
    }

    debug!(
        "✨  Smart Contract pallet {:?} cleaning ActiveNodeContracts storage map END",
        PalletVersion::<T>::get()
    );

    T::DbWeight::get().reads_writes(r + 2, w)
}

// ActiveRentContractForNode
pub fn clean_active_rent_contract_for_node<T: Config>() -> frame_support::weights::Weight {
    debug!(
        "🧼  Smart Contract pallet {:?} cleaning ActiveRentContractForNode storage map START",
        PalletVersion::<T>::get()
    );

    let active_rent_contract_for_node = ActiveRentContractForNode::<T>::iter().collect::<Vec<_>>();

    let to_remove: Vec<_> = active_rent_contract_for_node.clone().into_iter()
        .filter(|(_, contract_id)| Contracts::<T>::get(contract_id).is_none())
        .map(|(node_id, _)| node_id)
        .collect();

    for node_id in to_remove.clone() {
        ActiveRentContractForNode::<T>::remove(node_id);
    }

    debug!(
        "✨  Smart Contract pallet {:?} cleaning ActiveRentContractForNode storage map END",
        PalletVersion::<T>::get()
    );

    let r = 2 * active_rent_contract_for_node.len() as u64 + 2;
    let w = to_remove.len() as u64;

    T::DbWeight::get().reads_writes(r, w)
}

// ContractIDByNodeIDAndHash
pub fn clean_contract_id_by_node_id_and_hash<T: Config>() -> frame_support::weights::Weight {
    debug!(
        "🧼  Smart Contract pallet {:?} cleaning ContractIDByNodeIDAndHash storage map START",
        PalletVersion::<T>::get()
    );

    // !!! This map storage is re-built from zero !!!

    // 1. Remove all items under ContractIDByNodeIDAndHash
    let _ = frame_support::migration::clear_storage_prefix(
        b"SmartContractModule",
        b"ContractIDByNodeIDAndHash",
        b"",
        None,
        None,
    );

    let mut r: u64 = 0;
    let mut w: u64 = 0;

    // 2. Insert items based on existing node contracts
    for (contract_id, contract) in Contracts::<T>::iter() {
        match contract.contract_type {
            types::ContractData::NodeContract(node_contract) => {
                ContractIDByNodeIDAndHash::<T>::insert(
                    node_contract.node_id,
                    node_contract.deployment_hash,
                    contract_id,
                );
                w += 1;
            }
            _ => (),
        }
        r += 1;
    }

    debug!(
        "✨  Smart Contract pallet {:?} cleaning ContractIDByNodeIDAndHash storage map END",
        PalletVersion::<T>::get()
    );

    T::DbWeight::get().reads_writes(r + 2, w)
}

// ContractIDByNameRegistration
pub fn clean_contract_id_by_name_registration<T: Config>() -> frame_support::weights::Weight {
    debug!(
        "🧼  Smart Contract pallet {:?} cleaning ContractIDByNameRegistration storage map START",
        PalletVersion::<T>::get()
    );

    let contract_id_by_name_registration = ContractIDByNameRegistration::<T>::iter().collect::<Vec<_>>();

    let to_remove: Vec<_> = contract_id_by_name_registration.clone().into_iter()
        .filter(|(_, contract_id)| Contracts::<T>::get(contract_id).is_none())
        .map(|(name, _)| name)
        .collect();

    for contract_id in to_remove.clone() {
        ContractIDByNameRegistration::<T>::remove(contract_id);
    }

    debug!(
        "✨  Smart Contract pallet {:?} cleaning ContractIDByNameRegistration storage map END",
        PalletVersion::<T>::get()
    );

    let r = 2 * contract_id_by_name_registration.len() as u64 + 2;
    let w = to_remove.len() as u64;

    T::DbWeight::get().reads_writes(r, w)
}

// ContractLock
pub fn clean_contract_lock<T: Config>() -> frame_support::weights::Weight {
    debug!(
        "🧼  Smart Contract pallet {:?} cleaning ContractLock storage map START",
        PalletVersion::<T>::get()
    );

    let contract_lock = ContractLock::<T>::iter().collect::<Vec<_>>();

    let to_remove: Vec<u64> = contract_lock.clone().into_iter()
        .filter(|(contract_id, _)| Contracts::<T>::get(contract_id).is_none())
        .map(|(id, _)| id)
        .collect();

    for contract_id in to_remove.clone() {
        ContractLock::<T>::remove(contract_id);
    }

    debug!(
        "✨  Smart Contract pallet {:?} cleaning ContractLock storage map END",
        PalletVersion::<T>::get()
    );

    let r = 2 * contract_lock.len() as u64 + 2;
    let w = to_remove.len() as u64;

    T::DbWeight::get().reads_writes(r, w)
}

// SolutionProviders
pub fn clean_solution_providers<T: Config>() -> frame_support::weights::Weight {
    debug!(
        "🧼  Smart Contract pallet {:?} cleaning SolutionProviders storage map START",
        PalletVersion::<T>::get()
    );

    // Nothing to do here

    debug!(
        "✨  Smart Contract pallet {:?} cleaning SolutionProviders storage map END",
        PalletVersion::<T>::get()
    );

    T::DbWeight::get().reads_writes(2, 0)
}

// ContractBillingInformationByID
pub fn clean_contract_billing_information_by_id<T: Config>() -> frame_support::weights::Weight {
    debug!(
        "🧼  Smart Contract pallet {:?} cleaning ContractBillingInformationByID storage map START",
        PalletVersion::<T>::get()
    );

    let contract_billing_information_by_id = ContractBillingInformationByID::<T>::iter().collect::<Vec<_>>();

    let to_remove: Vec<u64> = contract_billing_information_by_id.clone().into_iter()
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

    for contract_id in to_remove.clone() {
        ContractBillingInformationByID::<T>::remove(contract_id); 
    }

    debug!(
        "✨  Smart Contract pallet {:?} cleaning ContractBillingInformationByID storage map END",
        PalletVersion::<T>::get()
    );

    let r = 2 * contract_billing_information_by_id.len() as u64 + 2;
    let w = to_remove.len() as u64;

    T::DbWeight::get().reads_writes(r, w)
}

// NodeContractResources
pub fn clean_node_contract_resources<T: Config>() -> frame_support::weights::Weight {
    debug!(
        "🧼  Smart Contract pallet {:?} cleaning NodeContractResources storage map START",
        PalletVersion::<T>::get()
    );

    let node_contract_resources = NodeContractResources::<T>::iter().collect::<Vec<_>>();

    let to_remove: Vec<u64> = node_contract_resources.clone().into_iter()
        .filter(|(contract_id, _)| Contracts::<T>::get(contract_id).is_none())
        .map(|(id, _)| id)
        .collect();

    for contract_id in to_remove.clone() {
        NodeContractResources::<T>::remove(contract_id);
    }

    debug!(
        "✨  Smart Contract pallet {:?} cleaning NodeContractResources storage map END",
        PalletVersion::<T>::get()
    );

    let r = 2 * node_contract_resources.len() as u64 + 2;
    let w = to_remove.len() as u64;

    T::DbWeight::get().reads_writes(r, w)
}