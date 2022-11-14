use crate::{
    types::{
        CapacityReservationContract, Contract, ContractBillingInformation, ContractData,
        ContractState, DeploymentContract, NameContract, StorageVersion,
    },
    ActiveNodeContracts, ActiveRentContractForNode, BalanceOf, BillingFrequency, Config,
    ContractBillingInformationByID, ContractID, ContractLock, Contracts as ContractsV7,
    ContractsToBillAt, NodeContractResources, PalletVersion, CONTRACT_VERSION,
};
use frame_support::{
    pallet_prelude::OptionQuery, pallet_prelude::Weight, storage_alias, traits::Get,
    traits::OnRuntimeUpgrade, Blake2_128Concat,
};
use frame_system::Pallet;
use log::info;
use sp_std::{cmp::max, collections::btree_map::BTreeMap, marker::PhantomData, vec, vec::Vec};
use tfchain_support::types::{ConsumableResources, Resources};

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

        info!("👥  Smart Contract pallet to V6 passes PRE migrate checks ✅",);
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
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        
        info!(
            "👥  Smart Contract pallet to {:?} passes POST migrate checks ✅",
            PalletVersion::<T>::get()
        );

        Ok(())
    }
}

pub fn migrate_to_version_7<T: Config>() -> frame_support::weights::Weight {
    info!(
        " >>> Starting contract pallet migration, pallet version: {:?}",
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
                let res = !matches!(c.contract_type, ContractData::DeploymentContract(_));
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
                }

                // remove the contract id from the billing as we don't bill deployment contracts
                bill_index_per_contract_id.remove(&ctr.contract_id);

                // create the deployment contract
                let dc = DeploymentContract {
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

                ContractData::DeploymentContract(dc)
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
