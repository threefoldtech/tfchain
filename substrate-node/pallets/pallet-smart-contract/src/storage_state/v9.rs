use crate::pallet_tfgrid;
use crate::*;
use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};
use log::{debug, info};
use sp_std::marker::PhantomData;
use scale_info::prelude::string::String;

pub struct CheckStorageStateV9<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for CheckStorageStateV9<T> {
    fn on_runtime_upgrade() -> Weight {
        if PalletVersion::<T>::get() == types::StorageVersion::V9 {
            info!(
                " >>> Starting Smart Contract pallet {:?} storage check",
                PalletVersion::<T>::get()
            );
            check_contracts_to_bill_at::<T>()
                + check_contracts::<T>()
                + check_active_node_contracts::<T>()
                + check_active_rent_contract_for_node::<T>()
                + check_contract_id_by_node_id_and_hash::<T>()
                + check_contract_id_by_name_registration::<T>()
                + check_contract_lock::<T>()
        } else {
            info!(" >>> Unused Smart Contract pallet V9 storage check");
            Weight::zero()
        }

        // ✅ Contracts
        // ContractBillingInformationByID
        // NodeContractResources
        // ✅ ContractIDByNodeIDAndHash
        // ✅ ActiveNodeContracts
        // ✅ ContractsToBillAt
        // ContractLock
        // ✅ ContractIDByNameRegistration
        // ✅ ActiveRentContractForNode
        // SolutionProviders
    }
}

// ContractsToBillAt
pub fn check_contracts_to_bill_at<T: Config>() -> frame_support::weights::Weight {
    let mut contract_id_count = vec![0; (ContractID::<T>::get() + 1) as usize];

    for (index, contract_ids) in ContractsToBillAt::<T>::iter() {
        debug!("index: {}, contracts: {:?}", index, contract_ids);
        for contract_id in contract_ids {
            contract_id_count[contract_id as usize] += 1;
            let contract = Contracts::<T>::get(contract_id);
            if contract.is_none() {
                info!(" ⚠️  Rogue Contract (id: {}) in billing loop", contract_id);
            }
            if let Some(c) = contract {
                if let types::ContractData::NodeContract(node_contract) = c.contract_type {
                    let contract_resource = NodeContractResources::<T>::get(contract_id);
                    if node_contract.public_ips == 0 && contract_resource.used.is_empty() {
                        info!(
                                " ⚠️  Node Contract (id: {}) with no pub ips + no resources in billing loop",
                                contract_id
                            );
                    }
                }
            }
        }
    }

    // A contract id should be in billing loop only if contract still exists
    // In this case it should exactly be stored once unless it has no pub ips and no resources
    for (contract_id, count) in contract_id_count.iter().enumerate() {
        let contract = Contracts::<T>::get(contract_id as u64);
        match contract {
            Some(c) => match count {
                0 => {
                    if let types::ContractData::NodeContract(node_contract) = c.contract_type {
                        let contract_resource = NodeContractResources::<T>::get(contract_id as u64);
                        if node_contract.public_ips == 0 && contract_resource.used.is_empty() {
                            continue;
                        }
                    }
                    info!(
                        " ⚠️  Contract (id: {}) should be in billing loop",
                        contract_id
                    );
                }
                1 => {
                    if let types::ContractData::NodeContract(node_contract) = c.contract_type {
                        let contract_resource = NodeContractResources::<T>::get(contract_id as u64);
                        if node_contract.public_ips == 0 && contract_resource.used.is_empty() {
                            info!(
                                " ⚠️  Node Contract (id: {}) should not be in billing loop",
                                contract_id
                            );
                        }
                    }
                }
                _ => {
                    info!(
                        " ⚠️  Contract (id: {}) duplicated {} times in billing loop",
                        contract_id, count
                    );
                }
            },
            _ => {
                if count > &0 {
                    info!(
                        " ⚠️  Contract (id: {}) should not be in billing loop",
                        contract_id
                    );
                }
            }
        }
    }

    info!(
        "👥  Smart Contract pallet {:?} passes ContractsToBillAt check ✅",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

// Contracts
pub fn check_contracts<T: Config>() -> frame_support::weights::Weight {
    let contract_id_range = 1..=ContractID::<T>::get();
    for (contract_id, contract) in Contracts::<T>::iter() {
        if contract_id != contract.contract_id {
            info!(
                " ⚠️  Contracts[id: {}]: wrong id ({})",
                contract_id, contract.contract_id
            );
        }
        if !contract_id_range.contains(&contract_id) {
            info!(
                " ⚠️  Contracts[id: {}]: id not in range {:?}",
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
    }

    info!(
        "👥  Smart Contract pallet {:?} passes Contracts storage map check ✅",
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
                " ⚠️  Node Contract (id: {}) on node {}: contract not in active list ({:?})",
                contract_id, node_id, active_node_contracts
            );
        }

        // ContractIDByNodeIDAndHash
        let ctr_id = ContractIDByNodeIDAndHash::<T>::get(node_id, &deployment_hash);
        if ctr_id == 0 {
            info!(
                " ⚠️  Node Contract (id: {}) on node {}: key not exists in node/deployment map",
                contract_id, node_id
            );
        } else if ctr_id != contract_id {
            info!(
                " ⚠️  Node Contract (id: {}) on node {}: wrong contract ({}) in node/deployment map",
                contract_id, node_id, ctr_id
            );
        }
    } else {
        info!(
            " ⚠️  Node Contract (id: {}) on node {}: node not exists",
            contract_id, node_id
        );
    }
}

fn check_name_contract<T: Config>(contract_id: u64, name: &T::NameContractName) {
    // ContractIDByNameRegistration
    let ctr_id = ContractIDByNameRegistration::<T>::get(name);
    if ctr_id == 0 {
        info!(
            " ⚠️  Name Contract (id: {}): key ({:?}) not exists",
            contract_id, name
        );
    }
    else if ctr_id != contract_id  {
        info!(
            " ⚠️  Name Contract (id: {}): wrong contract ({}) in name registration map",
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
                " ⚠️  Rent Contract (id: {}) on node {}: contract not activated ({:?})",
                contract.contract_id, node_id, active_rent_contract
            );
        }
        // ActiveNodeContracts
        let active_node_contracts = ActiveNodeContracts::<T>::get(node_id);
        for ctr_id in active_node_contracts {
            if let Some(node_contract) = Contracts::<T>::get(ctr_id) {
                if contract.twin_id != node_contract.twin_id {
                    info!(
                        " ⚠️  Rent Contract (id: {}) on node {}: not matching twin on node contract (id: {})",
                        contract.contract_id, node_id, ctr_id
                    );
                }
            }
        }
    } else {
        info!(
            " ⚠️  Rent Contract (id: {}) on node {}: node not exists",
            contract.contract_id, node_id
        );
    }
}

// ActiveNodeContracts
pub fn check_active_node_contracts<T: Config>() -> frame_support::weights::Weight {
    for (node_id, contract_ids) in ActiveNodeContracts::<T>::iter() {
        let node = pallet_tfgrid::Nodes::<T>::get(node_id);
        if node.is_none() {
            info!(
                " ⚠️  ActiveNodeContracts[node: {}]: node not exists",
                node_id
            );
        }

        for ctr_id in contract_ids {
            let contract = Contracts::<T>::get(ctr_id);
            if let Some(c) = contract {
                match c.contract_type {
                    types::ContractData::NodeContract(_) => (),
                    _ => {
                        info!(
                        " ⚠️  ActiveNodeContracts[node: {}, contract: {}]: type is not node contract",
                        node_id, ctr_id
                    );
                    }
                }
            } else {
                info!(
                    " ⚠️  ActiveNodeContracts[node: {}]: contract {} not exists",
                    node_id, ctr_id
                );
            }
        }
    }

    info!(
        "👥  Smart Contract pallet {:?} passes ActiveNodeContracts storage map check ✅",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

// ActiveRentContractForNode
pub fn check_active_rent_contract_for_node<T: Config>() -> frame_support::weights::Weight {
    for (node_id, contract_id) in ActiveRentContractForNode::<T>::iter() {
        let node = pallet_tfgrid::Nodes::<T>::get(node_id);
        if node.is_none() {
            info!(
                " ⚠️  ActiveRentContractForNode[node: {}]: node not exists",
                node_id
            );
        }

        let contract = Contracts::<T>::get(contract_id);
        if let Some(c) = contract {
            match c.contract_type {
                types::ContractData::RentContract(_) => (),
                _ => {
                    info!(
                        " ⚠️  ActiveRentContractForNode[node: {}]: contract {} is not a rent contract",
                        node_id, contract_id
                    );
                }
            }
        } else {
            info!(
                " ⚠️  ActiveRentContractForNode[node: {}]: contract {} not exists",
                node_id, contract_id
            );
        }
    }

    info!(
        "👥  Smart Contract pallet {:?} passes ActiveRentContractForNode storage map check ✅",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

// ContractIDByNodeIDAndHash
pub fn check_contract_id_by_node_id_and_hash<T: Config>() -> frame_support::weights::Weight {
    for (node_id, hash, contract_id) in ContractIDByNodeIDAndHash::<T>::iter() {
        let node = pallet_tfgrid::Nodes::<T>::get(node_id);
        if node.is_none() {
            info!(
                " ⚠️  ContractIDByNodeIDAndHash[node: {}, hash: {:?}]: node not exists",
                node_id, String::from_utf8_lossy(&hash)
            );
        }

        let contract = Contracts::<T>::get(contract_id);
        if let Some(c) = contract {
            match c.contract_type {
                types::ContractData::NodeContract(node_contract) => {
                    if node_contract.deployment_hash != hash {
                        info!(
                            " ⚠️  ContractIDByNodeIDAndHash[node: {}, hash: {:?}]: deployment hash ({:?}) on contract {} is not matching",
                            node_id, String::from_utf8_lossy(&hash), String::from_utf8_lossy(&node_contract.deployment_hash), contract_id, 
                        );
                    }
                }
                _ => {
                    info!(
                        " ⚠️  ContractIDByNodeIDAndHash[node: {}, hash: {:?}]: contract {} is not a node contract",
                        node_id, String::from_utf8_lossy(&hash), contract_id
                    );
                }
            }
        } else {
            info!(
                " ⚠️  ContractIDByNodeIDAndHash[node: {}, hash: {:?}]: contract {} not exists",
                node_id, String::from_utf8_lossy(&hash), contract_id
            );
        }
    }

    info!(
        "👥  Smart Contract pallet {:?} passes ContractIDByNodeIDAndHash storage map check ✅",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

// ContractIDByNameRegistration
pub fn check_contract_id_by_name_registration<T: Config>() -> frame_support::weights::Weight {
    for (name, contract_id) in ContractIDByNameRegistration::<T>::iter() {
        let contract = Contracts::<T>::get(contract_id);
        if let Some(c) = contract {
            match c.contract_type {
                types::ContractData::NameContract(name_contract) => {
                    if name_contract.name != name {
                        info!(
                            " ⚠️  ContractIDByNameRegistration[name: {:?}]: name ({:?}) on contract {} is not matching",
                            name, name_contract.name, contract_id, 
                        );
                    }
                }
                _ => {
                    info!(
                        " ⚠️  ContractIDByNameRegistration[name: {:?}]: contract {} is not a name contract",
                        name, contract_id
                    );
                }
            }
        } else {
            info!(
                " ⚠️  ContractIDByNameRegistration[name: {:?}]: contract {} not exists",
                name, contract_id
            );
        }
    }

    info!(
        "👥  Smart Contract pallet {:?} passes ContractIDByNameRegistration storage map check ✅",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

// ContractLock
pub fn check_contract_lock<T: Config>() -> frame_support::weights::Weight {
    //TODO

    info!(
        "👥  Smart Contract pallet {:?} passes ContractLock storage map check ✅",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}
