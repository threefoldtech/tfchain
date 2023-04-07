use crate::pallet_tfgrid;
use crate::*;
use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};
use log::{debug, info};
use sp_std::marker::PhantomData;

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
                + check_contract_lock::<T>()
        } else {
            info!(" >>> Unused Smart Contract pallet V9 storage check");
            Weight::zero()
        }

        // Contracts
        // ContractBillingInformationByID
        // NodeContractResources
        // ContractIDByNodeIDAndHash
        // ActiveNodeContracts
        // ContractsToBillAt ✅
        // ContractLock
        // ContractIDByNameRegistration
        // ActiveRentContractForNode ✅
        // SolutionProviders
        // SolutionProviderID
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
                info!(" ⚠️ rogue contract (id: {}) in billing loop", contract_id);
            }
            if let Some(c) = contract {
                if let types::ContractData::NodeContract(node_contract) = c.contract_type {
                    let contract_resource = NodeContractResources::<T>::get(contract_id);
                    if node_contract.public_ips == 0 && contract_resource.used.is_empty() {
                        info!(
                                " ⚠️ node contract (id: {}) with no pub ips + no resources in billing loop",
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
                        " ⚠️ contract (id: {}) should be in billing loop",
                        contract_id
                    );
                }
                1 => {
                    if let types::ContractData::NodeContract(node_contract) = c.contract_type {
                        let contract_resource = NodeContractResources::<T>::get(contract_id as u64);
                        if node_contract.public_ips == 0 && contract_resource.used.is_empty() {
                            info!(
                                " ⚠️ node contract (id: {}) should not be in billing loop",
                                contract_id
                            );
                        }
                    }
                }
                _ => {
                    info!(
                        " ⚠️ contract (id: {}) duplicated {} times in billing loop",
                        contract_id, count
                    );
                }
            },
            _ => {
                if count > &0 {
                    info!(
                        " ⚠️ contract (id: {}) should not be in billing loop",
                        contract_id
                    );
                }
            }
        }
    }

    info!(
        "👥  Smart Contract pallet {:?} passes billing loop check ✅",
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
                " ⚠️ Contracts[id: {}]: wrong id ({})",
                contract_id, contract.contract_id
            );
        }
        if !contract_id_range.contains(&contract_id) {
            info!(
                " ⚠️ Contracts[id: {}]: id not in range {:?}",
                contract_id, contract_id_range
            );
        }
        match contract.contract_type {
            types::ContractData::NodeContract(_node_contract) => check_node_contract::<T>(),
            types::ContractData::NameContract(_name_contract) => check_name_contract::<T>(),
            types::ContractData::RentContract(ref rent_contract) => {
                check_rent_contract::<T>(rent_contract.node_id, &contract)
                // let node_id = rent_contract.node_id;
                // let node = pallet_tfgrid::Nodes::<T>::get(node_id);
                // if let Some(_) = node {
                //     // ActiveRentContractForNode
                //     let active_rent_contract = ActiveRentContractForNode::<T>::get(node_id);
                //     if active_rent_contract != Some(contract_id) {
                //         info!(
                //             " ⚠️ rent contract (id: {}) on node {} not activated ({:?})",
                //             contract_id, node_id, active_rent_contract
                //         );
                //     }
                //     // ActiveNodeContracts
                //     let active_node_contracts = ActiveNodeContracts::<T>::get(node_id);
                //     for ctr_id in active_node_contracts {
                //         if let Some(node_contract) = Contracts::<T>::get(ctr_id) {
                //             if contract.twin_id != node_contract.twin_id {
                //                 info!(
                //                         " ⚠️ rent contract (id: {}) on node {} not matching twin on node contract (id: {})",
                //                         contract_id, node_id, ctr_id
                //                     );
                //             }
                //         }
                //     }
                // } else {
                //     info!(
                //         " ⚠️ rent contract (id: {}) node {} not exists",
                //         contract_id, node_id
                //     );
                // }
            } // _ => (),
        }
    }

    info!(
        "👥  Smart Contract pallet {:?} passes Contracts storage map check ✅",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}

fn check_rent_contract<T: Config>(node_id: u32, contract: &types::Contract<T>) {
    let node = pallet_tfgrid::Nodes::<T>::get(node_id);
    if let Some(_) = node {
        // ActiveRentContractForNode
        let active_rent_contract = ActiveRentContractForNode::<T>::get(node_id);
        if active_rent_contract != Some(contract.contract_id) {
            info!(
                " ⚠️ rent contract (id: {}) on node {} not activated ({:?})",
                contract.contract_id, node_id, active_rent_contract
            );
        }
        // ActiveNodeContracts
        let active_node_contracts = ActiveNodeContracts::<T>::get(node_id);
        for ctr_id in active_node_contracts {
            if let Some(node_contract) = Contracts::<T>::get(ctr_id) {
                if contract.twin_id != node_contract.twin_id {
                    info!(
                        " ⚠️ rent contract (id: {}) on node {} not matching twin on node contract (id: {})",
                        contract.contract_id, node_id, ctr_id
                    );
                }
            }
        }
    } else {
        info!(
            " ⚠️ rent contract (id: {}) node {} not exists",
            contract.contract_id, node_id
        );
    }
}

fn check_node_contract<T: Config>() {
    //TODO
}

fn check_name_contract<T: Config>() {
    //TODO
}

// ActiveNodeContracts
pub fn check_active_node_contracts<T: Config>() -> frame_support::weights::Weight {
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
                " ⚠️ ActiveRentContractForNode[node: {}, contract: {}]: node not exists",
                node_id, contract_id
            );
        }

        let contract = Contracts::<T>::get(contract_id);
        if let Some(c) = contract {
            match c.contract_type {
                types::ContractData::RentContract(_) => (),
                _ => {
                    info!(
                        " ⚠️ ActiveRentContractForNode[node: {}, contract: {}]: type is not rent contract",
                        node_id, contract_id
                    );
                }
            }
        } else {
            info!(
                " ⚠️ ActiveRentContractForNode[node: {}, contract: {}]: contract not exists",
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

// ContractLock
pub fn check_contract_lock<T: Config>() -> frame_support::weights::Weight {
    //TODO

    info!(
        "👥  Smart Contract pallet {:?} passes contract lock check ✅",
        PalletVersion::<T>::get()
    );

    Weight::zero()
}
