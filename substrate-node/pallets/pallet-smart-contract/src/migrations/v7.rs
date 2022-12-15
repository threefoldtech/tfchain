use crate::{
    types::{
        CapacityReservationContract, Contract, ContractBillingInformation, ContractData,
        Deployment, NameContract, StorageVersion, ContractResources
    },
    
    ActiveNodeContracts, BalanceOf, BillingFrequency, Config,
    ContractBillingInformationByID, ContractID, ContractLock, Contracts as ContractsV7,
    ContractsToBillAt, DeploymentID, Deployments, Pallet, 
    PalletVersion, CONTRACT_VERSION,
};
use frame_support::{
    pallet_prelude::{OptionQuery, ValueQuery}, pallet_prelude::Weight, storage_alias, traits::Get,
    traits::OnRuntimeUpgrade, Blake2_128Concat,
};
use log::{info, debug};
use sp_std::{cmp::max, collections::btree_map::BTreeMap, marker::PhantomData, vec, vec::Vec};
use tfchain_support::{resources::Resources, types::{ConsumableResources}};

#[cfg(feature = "try-runtime")]
use frame_support::traits::OnRuntimeUpgradeHelpersExt;
#[cfg(feature = "try-runtime")]
use sp_runtime::SaturatedConversion;

pub mod deprecated {
    use crate::pallet::{
        MaxDeploymentDataLength, MaxNodeContractPublicIPs,
    };
    use crate::types;
    use crate::Config;
    use codec::{Decode, Encode, MaxEncodedLen};
    use frame_support::decl_module;
    use frame_support::{BoundedVec, RuntimeDebugNoBound};
    use tfchain_support::types::PublicIP;

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
        pub public_ips_list: BoundedVec<PublicIP, MaxNodeContractPublicIPs<T>>,
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
pub type ActiveRentContractForNode<T: Config> =
    StorageMap<Pallet<T>, Blake2_128Concat, u32, u64, OptionQuery>;

#[storage_alias]
type NodeContractResources<T: Config> =
    StorageMap<Pallet<T>, Blake2_128Concat, u64, ContractResources, ValueQuery>;

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
    pub deployments: Vec<u64>,
    pub contract_lock: crate::types::ContractLock<BalanceOf<T>>,
    pub billing_info: ContractBillingInformation,
}

impl<T: Config> OnRuntimeUpgrade for ContractMigrationV6<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        let contract_counts: u64 = Contracts::<T>::iter_keys().count() as u64;
        let mut expected_contract_count_after_migration = contract_counts;
        // all node contracts that were part of a rent contract will not be transformed into a capacity reservation contract
        for (node_id, _) in ActiveRentContractForNode::<T>::iter() {
            expected_contract_count_after_migration -= ActiveNodeContracts::<T>::get(node_id).len() as u64;
        }
        Self::set_temp_storage(expected_contract_count_after_migration, "expected_contract_count_after_migration");
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
        let expected_contract_count_after_migration = Self::get_temp_storage("expected_contract_count_after_migration").unwrap_or(0u64);
        let post_contract_counts = ContractsV7::<T>::iter().count().saturated_into::<u64>();
        assert_eq!(post_contract_counts,
            expected_contract_count_after_migration, 
            "This migration failed: expectation did not equal the actual contract count!"
        );
        post_migration_checks::<T>();
        info!(
            "👥  Smart Contract pallet to {:?} passes POST migrate checks ✅",
            PalletVersion::<T>::get(),
        );

        Ok(())
    }
}

pub fn post_migration_checks<T: Config>() {
    // lets check the contracts
    let mut count = 0;
    let mut deployments_count = 0;
    for (_, contract) in ContractsV7::<T>::iter() {
        assert!(
            is_in_contracts_to_bill::<T>(contract.contract_id), 
            "Contract with id {:?} should be in ContractsToBill!", 
            contract.contract_id
        );
        match contract.contract_type {
            ContractData::NameContract(_) => { }
            ContractData::CapacityReservationContract(crc) => {
                let mut resources_check = Resources::empty();
                let mut pub_ips = 0;
                deployments_count += crc.deployments.len();
                for d_id in crc.deployments {
                    let deployment = Deployments::<T>::get(d_id).unwrap();
                    resources_check.add(&deployment.resources);
                    pub_ips += deployment.public_ips;
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

    // lets check the deployments
    assert_eq!(deployments_count, Deployments::<T>::iter().count(), "Migration failure! The amount of deployments doesn't equal the deployment ids found in the capacity reservations!");
    for (_, deployment) in Deployments::<T>::iter() {
        assert!(
            ContractsV7::<T>::contains_key(deployment.capacity_reservation_id),
            "Migration failure! Capacity Reservation Contract with id {:?} not found!",
            deployment.capacity_reservation_id
        );
    }

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

pub fn translate_contract_objects<T: Config>(
    contracts: &mut BTreeMap<u64, Contract<T>>,
    bill_index_per_contract_id: &mut BTreeMap<u64, u64>,
) -> (u32, u32) {
    let mut reads = 0;
    let mut writes = 0;
    let mut crc_changes: BTreeMap<u64, ContractChanges<T>> = BTreeMap::new();
    let mut node_changes: BTreeMap<u32, NodeChanges> = BTreeMap::new();

    // DeploymentID gets value of ContractID as we will convert the existing Node Contracts to Deployments and they will take over their id
    DeploymentID::<T>::put(ContractID::<T>::get());
    reads += 1;
    writes +=1;

    for (k, ctr) in Contracts::<T>::drain() {
        reads += 1;
        let contract_type = match ctr.contract_type {
            deprecated::ContractData::NodeContract(ref nc) => {
                let used_resources = NodeContractResources::<T>::get(ctr.contract_id).used;
                // the capacity reservation id is either the id of the existing rent contract (which will become a capacity reservation contract) 
                // or we have to transform this contract into a capacity reservation contract and take over its id
                let crc_id = ActiveRentContractForNode::<T>::get(nc.node_id).unwrap_or(ctr.contract_id);
                reads += 2;

                // create the deployment it gets the node contracts' id as an id
                // see https://github.com/threefoldtech/tfchain/discussions/509
                let dc = Deployment {
                    id: ctr.contract_id,
                    twin_id: ctr.twin_id,
                    capacity_reservation_id: crc_id,
                    deployment_hash: nc.deployment_hash,
                    deployment_data: nc.deployment_data.clone(),
                    public_ips: nc.public_ips,
                    public_ips_list: nc.public_ips_list.clone(),
                    resources: used_resources,
                };
                Deployments::<T>::insert(ctr.contract_id, &dc);
                writes += 1;

                if crc_id == ctr.contract_id {
                    // There is no rent contract for it here so create a capacity reservation contract that consumes the
                    // required resource. The only deployment is the one created above. Also take the public ips from it.
                    let capacity_reservation_contract = CapacityReservationContract {
                            node_id: nc.node_id,
                            deployments: vec![ctr.contract_id],
                            public_ips: nc.public_ips,
                            resources: ConsumableResources {
                                total_resources: used_resources.clone(),
                                used_resources: used_resources.clone(),
                            },
                            group_id: None,
                        };
                    // gather the node changes
                    node_changes.entry(nc.node_id).and_modify(|changes| {
                        changes.active_contracts.push(crc_id);
                        changes.used_resources.add(&used_resources);
                    })
                    .or_insert(NodeChanges {
                        used_resources: used_resources,
                        active_contracts: vec![crc_id],
                    });
                    Some(ContractData::CapacityReservationContract(capacity_reservation_contract))
                } else {
                    // There was already a rent contract which will be converted into a capacity reservation contract so
                    // no need to transform this one in a capacity reservation contract. Gather the capacity reservation
                    // contract changes for later. 
                    // We have to merge the billing info and contract lock into the rent contracts' billing info and 
                    // contract lock 
                    let billing_info_nc = ContractBillingInformationByID::<T>::take(ctr.contract_id);
                    let contract_lock_nc = ContractLock::<T>::take(ctr.contract_id);
                    reads += 2;
                    writes += 2;
                    crc_changes
                    .entry(dc.capacity_reservation_id)
                    .and_modify(|changes| {
                        changes.used_resources.add(&dc.resources);
                        changes.public_ips += dc.public_ips;
                        changes.deployments.push(ctr.contract_id);
                        changes.billing_info.previous_nu_reported +=
                        billing_info_nc.previous_nu_reported;
                        changes.billing_info.last_updated = max(
                            changes.billing_info.last_updated,
                            billing_info_nc.last_updated,
                        );
                        changes.billing_info.amount_unbilled += billing_info_nc.amount_unbilled;
                        changes.contract_lock.amount_locked += contract_lock_nc.amount_locked;
                        changes.contract_lock.lock_updated = max(
                            changes.contract_lock.lock_updated,
                            contract_lock_nc.lock_updated,
                        );
                        changes.contract_lock.cycles =
                            max(changes.contract_lock.cycles, contract_lock_nc.cycles);
                    })
                    .or_insert(ContractChanges {
                        used_resources: dc.resources,
                        public_ips: dc.public_ips,
                        deployments: vec![ctr.contract_id],
                        billing_info: billing_info_nc,
                        contract_lock: contract_lock_nc,
                    });
                    // the rent contract should be remove from billing
                    bill_index_per_contract_id.remove(&ctr.contract_id);
                    None
                }
            }
            deprecated::ContractData::NameContract(nc) => {
                Some(ContractData::NameContract(NameContract { name: nc.name }))
            }
            deprecated::ContractData::RentContract(ref rc) => {
                if let Some(node) = pallet_tfgrid::Nodes::<T>::get(rc.node_id) {
                    reads += 2;
                    let crc = CapacityReservationContract {
                        node_id: rc.node_id,
                        deployments: vec![], // will be filled in later
                        public_ips: 0,       // will be modified later
                        resources: ConsumableResources {
                            total_resources: node.resources.total_resources,
                            used_resources: Resources::empty(), // will be modified later
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
                    Some(ContractData::CapacityReservationContract(crc))
                } else {
                    log::error!("Rencontract ({:?}) on a node ({:?}) that no longer exist.", ctr.contract_id, rc.node_id);
                    None
                }
            }
        };
        if let Some(contract_type) = contract_type {
            let new_contract = Contract {
                version: CONTRACT_VERSION,
                state: ctr.state,
                contract_id: ctr.contract_id,
                twin_id: ctr.twin_id,
                contract_type: contract_type,
                solution_provider_id: ctr.solution_provider_id,
            };
            debug!("Contract: {:?} succesfully migrated", k);
            contracts.insert(ctr.contract_id, new_contract);
        }
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
        if let Some(crc_contract) = contracts.get_mut(&contract_id) {
            let crc = match &crc_contract.contract_type {
                ContractData::CapacityReservationContract(c) => Some(c.clone()),
                _ => None,
            };
            if let Some(mut crc) = crc {
                crc.resources.used_resources = contract_changes.used_resources;
                crc.public_ips = contract_changes.public_ips;
                crc.deployments = contract_changes.deployments;
                crc_contract.contract_type = ContractData::CapacityReservationContract(crc);

                ContractBillingInformationByID::<T>::insert(contract_id, contract_changes.billing_info);
                ContractLock::<T>::insert(contract_id, contract_changes.contract_lock);
                writes += 2;
            } else {
                log::error!(
                    "Contract {:?} is not a capacity reservation contract! This should not happen here!", 
                    contract_id
                );
            }
        } else {
            log::error!(
                "Failed to unwrap contract! Capacity Reservation Contract with id {:?} might have invalid data (used_resources, public_ips, deployments, billing information and contract lock. Please recalculate!", 
                contract_id
            );
        }
    }

    // fix the active node contracts storage and modify the node objects
    frame_support::migration::remove_storage_prefix(
        b"SmartContractModule",
        b"ActiveNodeContracts",
        b"",
    );
    for (node_id, nc) in node_changes.iter() {
        ActiveNodeContracts::<T>::insert(node_id, &nc.active_contracts);

        if let Some(mut node) = pallet_tfgrid::Nodes::<T>::get(node_id) {
            node.resources.used_resources = nc.used_resources;
            pallet_tfgrid::Nodes::<T>::insert(node_id, &node);
            reads += 1;
            writes += 2;
        } else {
            log::error!(
                "Node {:?} no longer exist! This should not happen here!", 
                node_id
            );
        }
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