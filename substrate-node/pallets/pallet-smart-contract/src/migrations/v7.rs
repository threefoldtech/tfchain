use crate::{
    types::{
        CapacityReservationContract, Contract, ContractBillingInformation, ContractData,
        ContractState, Deployment, NameContract, StorageVersion,
    },
    ActiveNodeContracts, ActiveRentContractForNode, BalanceOf, BillingFrequency, Config,
    ContractBillingInformationByID, ContractID, ContractLock, Contracts as ContractsV7,
    ContractsToBillAt, NodeContractResources, Pallet, PalletVersion, CONTRACT_VERSION,
};
use frame_support::{
    pallet_prelude::OptionQuery, pallet_prelude::Weight, storage_alias, traits::Get,
    traits::OnRuntimeUpgrade, Blake2_128Concat,
};
use log::info;
use sp_std::{cmp::max, collections::btree_map::BTreeMap, marker::PhantomData, vec, vec::Vec};
use tfchain_support::{resources::Resources, types::ConsumableResources};

#[cfg(feature = "try-runtime")]
use frame_support::traits::OnRuntimeUpgradeHelpersExt;
#[cfg(feature = "try-runtime")]
use sp_runtime::SaturatedConversion;

pub mod deprecated {
    use crate::pallet::{
        ContractPublicIP, DeploymentHash, MaxDeploymentDataLength, MaxNodeContractPublicIPs,
    };
    use crate::types;
    use crate::Config;
    use codec::{Decode, Encode, MaxEncodedLen};
    use frame_support::decl_module;
    use frame_support::{BoundedVec, RuntimeDebugNoBound};

    use scale_info::TypeInfo;
    use sp_std::prelude::*;

    #[derive(
        Clone, Eq, PartialEq, RuntimeDebugNoBound, Encode, Decode, TypeInfo, MaxEncodedLen,
    )]
    #[scale_info(skip_type_params(T))]
    #[codec(mel_bound())]
    pub struct ContractV6<T: Config> {
        pub version: u32,
        pub state: types::ContractState,
        pub contract_id: u64,
        pub twin_id: u32,
        pub contract_type: ContractData<T>,
        pub solution_provider_id: Option<u64>,
    }

    #[derive(
        Clone, Eq, PartialEq, RuntimeDebugNoBound, Encode, Decode, TypeInfo, MaxEncodedLen,
    )]
    #[scale_info(skip_type_params(T))]
    #[codec(mel_bound())]
    pub struct NodeContract<T: Config> {
        pub node_id: u32,
        // Hash of the deployment, set by the user
        // Max 32 bytes
        pub deployment_hash: DeploymentHash,
        pub deployment_data: BoundedVec<u8, MaxDeploymentDataLength<T>>,
        pub public_ips: u32,
        pub public_ips_list: BoundedVec<ContractPublicIP<T>, MaxNodeContractPublicIPs<T>>,
    }

    #[derive(
        Clone, Eq, PartialEq, RuntimeDebugNoBound, Encode, Decode, TypeInfo, MaxEncodedLen,
    )]
    #[scale_info(skip_type_params(T))]
    #[codec(mel_bound())]
    pub struct NameContract<T: Config> {
        pub name: T::NameContractName,
    }

    #[derive(
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Clone,
        Encode,
        Decode,
        Default,
        RuntimeDebugNoBound,
        TypeInfo,
        MaxEncodedLen,
    )]
    pub struct RentContract {
        pub node_id: u32,
    }

    #[derive(
        Clone, Eq, PartialEq, RuntimeDebugNoBound, Encode, Decode, TypeInfo, MaxEncodedLen,
    )]
    #[scale_info(skip_type_params(T))]
    #[codec(mel_bound())]
    pub enum ContractData<T: Config> {
        NodeContract(NodeContract<T>),
        NameContract(NameContract<T>),
        RentContract(RentContract),
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::Origin { }
    }
}

#[storage_alias]
type Contracts<T: Config> =
    StorageMap<Pallet<T>, Blake2_128Concat, u64, deprecated::ContractV6<T>, OptionQuery>;

pub struct ContractMigrationV6<T: Config>(PhantomData<T>);

pub struct NodeChanges {
    pub used_resources: Resources,
    pub active_contracts: Vec<u64>,
}

pub struct ContractChanges<T: Config> {
    pub used_resources: Resources,
    pub public_ips: u32,
    pub deployment_contracts: Vec<u64>,
    pub contract_lock: crate::types::ContractLock<BalanceOf<T>>,
    pub billing_info: ContractBillingInformation,
}

impl<T: Config> OnRuntimeUpgrade for ContractMigrationV6<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        let contract_counts: u64 = Contracts::<T>::iter_keys().count() as u64;
        Self::set_temp_storage(contract_counts, "pre_contract_count");
        info!(
            "👥  Smart Contract pallet to V6 passes PRE migrate checks ✅: {:?} contracts",
            contract_counts
        );
        Ok(())
    }

    fn on_runtime_upgrade() -> Weight {
        if PalletVersion::<T>::get() == StorageVersion::V6 {
            migrate_to_version_7::<T>()
        } else {
            info!(" >>> Unused migration");
            0
        }
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        let pre_contract_counts = Self::get_temp_storage("pre_contract_count").unwrap_or(0u64);
        let post_contract_counts = ContractsV7::<T>::iter().count().saturated_into::<u64>();
        assert!(
            post_contract_counts >= pre_contract_counts,
            "This migration should result in the same amount of contracts or more!"
        );
        post_migration_checks::<T>();
        info!(
            "👥  Smart Contract pallet to {:?} passes POST migrate checks ✅: {:?} contracts",
            PalletVersion::<T>::get(),
            post_contract_counts,
        );

        Ok(())
    }
}

pub fn post_migration_checks<T: Config>() {
    // lets check the contracts
    let mut count = 0;
    for (_, contract) in ContractsV7::<T>::iter() {
        match contract.contract_type {
            ContractData::NameContract(_) => {
                assert!(
                    is_in_contracts_to_bill::<T>(contract.contract_id), 
                    "Name Contract with id {:?} should be in ContractsToBill!", 
                    contract.contract_id
                );
            }
            ContractData::Deployment(dc) => {
                assert!(
                    !is_in_contracts_to_bill::<T>(contract.contract_id), 
                    "Deployment contract with id {:?} should not be in ContractsToBill!", 
                    contract.contract_id
                );
                assert!(
                    ContractsV7::<T>::contains_key(dc.capacity_reservation_id),
                    "Migration failure! Capacity Reservation Contract with id {:?} not found!",
                    dc.capacity_reservation_id
                );
            }
            ContractData::CapacityReservationContract(crc) => {
                assert!(
                    is_in_contracts_to_bill::<T>(contract.contract_id), 
                    "Capacity Reservation Contract with id {:?} should be in ContractsToBill!", 
                    contract.contract_id
                );
                let mut resources_check = Resources::empty();
                let mut pub_ips = 0;
                for dc_id in crc.deployment_contracts {
                    let ctr = ContractsV7::<T>::get(dc_id).unwrap();
                    let dc = match ctr.contract_type {
                        ContractData::Deployment(dc) => Some(dc),
                        _ => None,
                    }
                    .unwrap();
                    resources_check.add(&dc.resources);
                    pub_ips += dc.public_ips;
                }
                assert_eq!(crc.resources.used_resources, resources_check, 
                    "Migration failure! The used resources of capacity reservation contract with id {:?} are incorrect!",
                    contract.contract_id
                );
                assert_eq!(
                    crc.public_ips, pub_ips,
                    "Migration failure! The amount of public ips for contract {:?} is incorrect!",
                    contract.contract_id
                );
            }
        }
        count += 1; 
    }
    info!("Checked the migration of {:?} contracts. All good ✅", count);

    // check node resources
    count = 0;
    for (node_id, crc_ids) in ActiveNodeContracts::<T>::iter() {
        let node = pallet_tfgrid::Nodes::<T>::get(node_id).unwrap();
        let mut resources_check = Resources::empty();
        for crc_id in crc_ids {
            let ctr = ContractsV7::<T>::get(crc_id).unwrap();
            let crc = match ctr.contract_type {
                ContractData::CapacityReservationContract(dc) => Some(dc),
                _ => None,
            }
            .unwrap();
            resources_check.add(&crc.resources.total_resources);
        }
        assert_eq!(
            node.resources.used_resources, resources_check,
            "Migration failure! The used resources of the node with id {:?} are incorrect!",
            node.id,
        );
        count += 1;
    }
    info!("Checked the migration of resources in {:?} nodes. All good ✅", count);

}

pub fn is_in_contracts_to_bill<T: Config>(contract_id: u64) -> bool {
    for index in 0..BillingFrequency::<T>::get() {
        if ContractsToBillAt::<T>::get(index).contains(&contract_id) { 
            return true;
        }
    }
    false
}

pub fn migrate_to_version_7<T: Config>() -> frame_support::weights::Weight {
    info!(
        " >>> Starting smart contract pallet migration, pallet version: {:?}",
        PalletVersion::<T>::get()
    );

    let mut total_reads = 0;
    let mut total_writes = 0;
    let mut contracts: BTreeMap<u64, Contract<T>> = BTreeMap::new();
    let (mut bill_index_per_contract_id, reads) = get_bill_index_per_contract_id::<T>();
    total_reads += reads;

    let (reads, writes) =
        translate_contract_objects::<T>(&mut contracts, &mut bill_index_per_contract_id);
    total_reads += reads;
    total_writes += writes;
    info!("translated {:?} contracts", contracts.len());

    let (reads, writes) = write_contracts_to_bill_at::<T>(&bill_index_per_contract_id);
    total_reads += reads;
    total_writes += writes;

    let writes = write_contracts_to_storage::<T>(&contracts);
    total_writes += writes;

    info!(" <<< Contracts storage updated! Migrated all Contracts ✅");

    // Update pallet storage version
    PalletVersion::<T>::set(StorageVersion::V7);
    info!(" <<< Storage version upgraded");

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(total_reads as Weight + 1, total_writes as Weight + 1)
}

pub fn write_contracts_to_storage<T: Config>(contracts: &BTreeMap<u64, Contract<T>>) -> u32 {
    for (contract_id, contract) in contracts {
        ContractsV7::<T>::insert(contract_id, contract);
    }
    contracts.len() as u32
}

pub fn write_contracts_to_bill_at<T: Config>(
    bill_index_per_contract_id: &BTreeMap<u64, u64>,
) -> (u32, u32) {
    let bill_frequency = BillingFrequency::<T>::get();
    let mut contract_ids_per_billing_index: Vec<Vec<u64>> = vec![vec![]; bill_frequency as usize];
    let mut writes = 0;

    // invert bill_index_per_contract_id
    for (contract_id, bill_index) in bill_index_per_contract_id {
        contract_ids_per_billing_index[*bill_index as usize].push(*contract_id);
    }

    // write to storage
    for index in 0..bill_frequency {
        if contract_ids_per_billing_index[index as usize].len() > 0 {
            ContractsToBillAt::<T>::insert(index, &contract_ids_per_billing_index[index as usize]);
            writes += 1;
        }
    }

    (1, writes)
}

pub fn remove_deployment_contracts_from_contracts_to_bill<T: Config>(
    contracts: &BTreeMap<u64, Contract<T>>,
) -> (u32, u32) {
    let mut writes = 0;
    // we only bill capacity reservation contracts and name contracts
    for index in 0..BillingFrequency::<T>::get() {
        let mut contract_ids = ContractsToBillAt::<T>::get(index);
        let mut modified = false;
        contract_ids.retain(|id| {
            if let Some(c) = contracts.get(id) {
                let res = !matches!(c.contract_type, ContractData::Deployment(_));
                modified |= res;
                res
            } else {
                // some contracts are still in contracts to bill but have been removed already
                // keep them as the chain will remove them and log stuff
                true
            }
        });
        if modified {
            ContractsToBillAt::<T>::insert(index, &contract_ids);
            writes += 1;
        }
    }
    (1, writes)
}

pub fn translate_contract_objects<T: Config>(
    contracts: &mut BTreeMap<u64, Contract<T>>,
    bill_index_per_contract_id: &mut BTreeMap<u64, u64>,
) -> (u32, u32) {
    let mut reads = 0;
    let mut writes = 0;
    let mut crc_changes: BTreeMap<u64, ContractChanges<T>> = BTreeMap::new();
    let mut node_changes: BTreeMap<u32, NodeChanges> = BTreeMap::new();

    for (k, ctr) in Contracts::<T>::drain() {
        reads += 1;
        let contract_type = match ctr.contract_type {
            deprecated::ContractData::NodeContract(ref nc) => {
                let used_resources = NodeContractResources::<T>::get(ctr.contract_id).used;
                let mut crc_id = ActiveRentContractForNode::<T>::get(nc.node_id).unwrap_or(0);
                reads += 2;
                // if there is no rent contract for it create a capacity reservation contract that consumes the required resources
                // else use the rent contract id as capacity reservation contract id
                if crc_id == 0 {
                    let billing_index = bill_index_per_contract_id
                        .get(&ctr.contract_id)
                        .unwrap_or(&0);
                    let (id, crc) = create_capacity_reservation::<T>(
                        nc.node_id,
                        ctr.twin_id,
                        ctr.state.clone(),
                        used_resources,
                        ctr.solution_provider_id,
                    );
                    crc_id = id;
                    contracts.insert(crc_id, crc);
                    bill_index_per_contract_id.insert(crc_id, *billing_index);
                    node_changes.entry(nc.node_id).and_modify(|changes| {
                        changes.active_contracts.push(crc_id);
                        changes.used_resources.add(&used_resources);
                    })
                    .or_insert(NodeChanges {
                        used_resources: used_resources,
                        active_contracts: vec![crc_id],
                    });
                }

                // remove the contract id from the billing as we don't bill deployment contracts
                bill_index_per_contract_id.remove(&ctr.contract_id);

                // create the deployment contract
                let dc = Deployment {
                    capacity_reservation_id: crc_id,
                    deployment_hash: nc.deployment_hash,
                    deployment_data: nc.deployment_data.clone(),
                    public_ips: nc.public_ips,
                    public_ips_list: nc.public_ips_list.clone(),
                    resources: used_resources,
                };

                // gather the capacity reservation contract changes for later so that we can modify them later
                let billing_info_dc = ContractBillingInformationByID::<T>::get(ctr.contract_id);
                let contract_lock_dc = ContractLock::<T>::get(ctr.contract_id);
                reads += 2;
                crc_changes
                    .entry(dc.capacity_reservation_id)
                    .and_modify(|changes| {
                        changes.used_resources.add(&dc.resources);
                        changes.public_ips += dc.public_ips;
                        changes.deployment_contracts.push(ctr.contract_id);
                        changes.billing_info.previous_nu_reported +=
                            billing_info_dc.previous_nu_reported;
                        changes.billing_info.last_updated = max(
                            changes.billing_info.last_updated,
                            billing_info_dc.last_updated,
                        );
                        changes.billing_info.amount_unbilled += billing_info_dc.amount_unbilled;
                        changes.contract_lock.amount_locked += contract_lock_dc.amount_locked;
                        changes.contract_lock.lock_updated = max(
                            changes.contract_lock.lock_updated,
                            contract_lock_dc.lock_updated,
                        );
                        changes.contract_lock.cycles =
                            max(changes.contract_lock.cycles, contract_lock_dc.cycles);
                    })
                    .or_insert(ContractChanges {
                        used_resources: dc.resources,
                        public_ips: dc.public_ips,
                        deployment_contracts: vec![ctr.contract_id],
                        billing_info: billing_info_dc,
                        contract_lock: contract_lock_dc,
                    });

                ContractData::Deployment(dc)
            }
            deprecated::ContractData::NameContract(nc) => {
                ContractData::NameContract(NameContract { name: nc.name })
            }
            deprecated::ContractData::RentContract(ref rc) => {
                let node = pallet_tfgrid::Nodes::<T>::get(rc.node_id).unwrap();
                let crc = CapacityReservationContract {
                    node_id: rc.node_id,
                    deployment_contracts: vec![],
                    public_ips: 0,
                    resources: ConsumableResources {
                        total_resources: node.resources.total_resources,
                        used_resources: Resources::empty(),
                    },
                    group_id: None,
                };
                // gather the node changes
                node_changes
                    .entry(crc.node_id)
                    .and_modify(|changes| {
                        changes.active_contracts.push(ctr.contract_id);
                        changes.used_resources.add(&crc.resources.total_resources);
                    })
                    .or_insert(NodeChanges {
                        used_resources: crc.resources.total_resources,
                        active_contracts: vec![ctr.contract_id],
                    });

                ContractData::CapacityReservationContract(crc)
            }
        };
        let new_contract = Contract {
            version: CONTRACT_VERSION,
            state: ctr.state,
            contract_id: ctr.contract_id,
            twin_id: ctr.twin_id,
            contract_type: contract_type,
            solution_provider_id: ctr.solution_provider_id,
        };
        info!("Contract: {:?} succesfully migrated", k);
        contracts.insert(ctr.contract_id, new_contract);
    }

    frame_support::migration::remove_storage_prefix(
        b"SmartContractModule",
        b"NodeContractResources",
        b"",
    );
    frame_support::migration::remove_storage_prefix(
        b"SmartContractModule",
        b"ActiveRentContractForNode",
        b"",
    );

    // apply the changes to the capacity reservations contracts that we gathered previously
    for (contract_id, contract_changes) in crc_changes {
        let crc_contract = contracts.get_mut(&contract_id).unwrap();
        let mut crc = match &crc_contract.contract_type {
            ContractData::CapacityReservationContract(c) => Some(c.clone()),
            _ => None,
        }
        .unwrap();
        crc.resources.used_resources = contract_changes.used_resources;
        crc.public_ips = contract_changes.public_ips;
        crc.deployment_contracts = contract_changes.deployment_contracts;
        crc_contract.contract_type = ContractData::CapacityReservationContract(crc);

        ContractBillingInformationByID::<T>::insert(contract_id, contract_changes.billing_info);
        ContractLock::<T>::insert(contract_id, contract_changes.contract_lock);
        writes += 2;
    }

    // fix the active node contracts storage and modif the node objects
    frame_support::migration::remove_storage_prefix(
        b"SmartContractModule",
        b"ActiveNodeContracts",
        b"",
    );
    for (node_id, nc) in node_changes.iter() {
        // modify storage
        ActiveNodeContracts::<T>::insert(node_id, &nc.active_contracts);
        // modify used resources of node object
        let mut node = pallet_tfgrid::Nodes::<T>::get(node_id).unwrap();
        node.resources.used_resources = nc.used_resources;
        pallet_tfgrid::Nodes::<T>::insert(node_id, &node);
        reads += 1;
        writes += 2;
    }

    (reads, writes)
}

pub fn get_bill_index_per_contract_id<T: Config>() -> (BTreeMap<u64, u64>, u32) {
    let mut bill_index_per_contract_id: BTreeMap<u64, u64> = BTreeMap::new();
    let mut reads = 1;
    for index in 0..BillingFrequency::<T>::get() {
        for ctr_id in ContractsToBillAt::<T>::get(index) {
            reads += 1;
            bill_index_per_contract_id.insert(ctr_id, index);
        }
    }
    info!(
        "bill_index_per_contract is {:?}",
        bill_index_per_contract_id.len()
    );

    (bill_index_per_contract_id, reads)
}

pub fn find_bill_index<T: Config>(contract_id: u64) -> (Option<u64>, u32) {
    let mut reads = 1;
    for index in 0..BillingFrequency::<T>::get() {
        for ctr_id in ContractsToBillAt::<T>::get(index) {
            reads += 1;
            if ctr_id == contract_id {
                return (Some(index), reads);
            }
        }
    }
    info!("Reads for finding bill index {:?}", reads);
    (None, reads)
}

pub fn create_capacity_reservation<T: Config>(
    node_id: u32,
    twin_id: u32,
    state: ContractState,
    resources: Resources,
    solution_provider_id: Option<u64>,
) -> (u64, Contract<T>) {
    let mut ctr_id = ContractID::<T>::get();
    ctr_id = ctr_id + 1;

    let capacity_reservation_contract = CapacityReservationContract {
        node_id: node_id,
        deployment_contracts: vec![],
        public_ips: 0,
        resources: ConsumableResources {
            total_resources: resources.clone(),
            used_resources: resources.clone(),
        },
        group_id: None,
    };

    let contract = Contract::<T> {
        version: CONTRACT_VERSION,
        state: state,
        contract_id: ctr_id,
        twin_id: twin_id,
        contract_type: ContractData::CapacityReservationContract(capacity_reservation_contract),
        solution_provider_id: solution_provider_id,
    };

    ContractID::<T>::put(ctr_id);

    (ctr_id, contract)
}
