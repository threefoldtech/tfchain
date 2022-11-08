use super::*;
use frame_support::weights::Weight;
use tfchain_support::types::ConsumableResources;

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

pub mod v6 {
    use super::*;
    use crate::Config;

    use frame_support::{pallet_prelude::Weight, traits::OnRuntimeUpgrade};
    use sp_std::marker::PhantomData;
    pub struct ContractMigrationV6<T: Config>(PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for ContractMigrationV6<T> {
        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<(), &'static str> {
            info!("current pallet version: {:?}", PalletVersion::<T>::get());
            assert!(PalletVersion::<T>::get() == types::StorageVersion::V5);

            info!("👥  Smart Contract pallet to V6 passes PRE migrate checks ✅",);
            Ok(())
        }

        fn on_runtime_upgrade() -> Weight {
            migrate_to_version_6::<T>();
            migrate_to_version_7::<T>()
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade() -> Result<(), &'static str> {
            info!("current pallet version: {:?}", PalletVersion::<T>::get());
            assert!(PalletVersion::<T>::get() == types::StorageVersion::V7);

            info!(
                "👥  Smart Contract pallet to {:?} passes POST migrate checks ✅",
                PalletVersion::<T>::get()
            );

            Ok(())
        }
    }
}

pub fn migrate_to_version_6<T: Config>() -> frame_support::weights::Weight {
    if PalletVersion::<T>::get() == types::StorageVersion::V5 {
        info!(
            " >>> Starting contract pallet migration, pallet version: {:?}",
            PalletVersion::<T>::get()
        );

        let mut migrated_count = 0;

        // Collect ContractsToBillAt storage in memory
        let contracts_to_bill_at = ContractsToBillAt::<T>::iter().collect::<Vec<_>>();

        // Remove all items under ContractsToBillAt
        frame_support::migration::remove_storage_prefix(
            b"SmartContractModule",
            b"ContractsToBillAt",
            b"",
        );

        let billing_freq = 600;
        BillingFrequency::<T>::put(billing_freq);

        for (block_number, contract_ids) in contracts_to_bill_at {
            migrated_count += 1;
            // Construct new index
            let index = (block_number - 1) % billing_freq;
            // Reinsert items under the new key
            info!(
                "inserted contracts:{:?} at index: {:?}",
                contract_ids.clone(),
                index
            );
            ContractsToBillAt::<T>::insert(index, contract_ids);
        }

        info!(
            " <<< Contracts storage updated! Migrated {} Contracts ✅",
            migrated_count
        );

        // Update pallet storage version
        PalletVersion::<T>::set(types::StorageVersion::V6);
        info!(" <<< Storage version upgraded");

        T::DbWeight::get().reads_writes(migrated_count as Weight + 1, migrated_count as Weight + 1)
    } else {
        info!(" >>> Unused migration");
        return 0;
    }
}

pub fn migrate_to_version_7<T: Config>() -> frame_support::weights::Weight {
    if PalletVersion::<T>::get() == types::StorageVersion::V6 {
        info!(
            " >>> Starting contract pallet migration, pallet version: {:?}",
            PalletVersion::<T>::get()
        );

        let translated_count = translate_contract_objects::<T>();
        let (mut total_reads, mut total_writes) =
            remove_deployment_contracts_from_contracts_to_bill::<T>();
        let (reads, writes) = add_used_resources_and_active_node_contracts::<T>();
        total_reads += reads;
        total_writes += writes;

        NodeContractResources::<T>::drain();
        ActiveRentContractForNode::<T>::drain();

        info!(
            " <<< Contracts storage updated! Migrated {} Contracts ✅",
            translated_count
        );

        // Update pallet storage version
        PalletVersion::<T>::set(types::StorageVersion::V7);
        info!(" <<< Storage version upgraded");

        // Return the weight consumed by the migration.
        T::DbWeight::get().reads_writes(
            (translated_count + total_reads) as Weight + 1,
            (translated_count + total_writes) as Weight + 1,
        )
    } else {
        info!(" >>> Unused migration");
        return 0;
    }
}

pub fn remove_deployment_contracts_from_contracts_to_bill<T: Config>() -> (u32, u32) {
    let mut reads = 0;
    let mut writes = 0;
    // we only bill capacity reservation contracts and name contracts
    for index in 0..BillingFrequency::<T>::get() {
        let mut contract_ids = ContractsToBillAt::<T>::get(index);
        reads += 1;
        let mut modified = false;
        contract_ids.retain(|&id| {
            let contract = Contracts::<T>::get(id);
            if let Some(c) = contract {
                let res = !matches!(c.contract_type, types::ContractData::DeploymentContract(_));
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
    (reads, writes)
}

pub fn add_used_resources_and_active_node_contracts<T: Config>() -> (u32, u32) {
    // active node contracts contains only capacity reservation contracts
    let mut reads = 0;
    let mut writes = 0;
    ActiveNodeContracts::<T>::drain();
    for (_contract_id, contract) in Contracts::<T>::iter() {
        match contract.contract_type {
            types::ContractData::NameContract(_) => {}
            types::ContractData::DeploymentContract(dc) => {
                // fix the used resources, the public ips and the deployment_contracts
                // of the corresponding CapacityReservationContract
                let mut crc_contract = Contracts::<T>::get(dc.capacity_reservation_id).unwrap();
                reads += 1;
                let mut crc = match crc_contract.contract_type {
                    types::ContractData::CapacityReservationContract(c) => Some(c),
                    _ => None,
                }
                .unwrap();
                crc.resources.used_resources = crc.resources.used_resources.add(&dc.resources);
                crc.public_ips += dc.public_ips;
                let mut deployment_contracts = crc.deployment_contracts;
                deployment_contracts.push(contract.contract_id);
                crc.deployment_contracts = deployment_contracts;
                // todo check for billing
                crc_contract.contract_type = types::ContractData::CapacityReservationContract(crc);
                Contracts::<T>::insert(crc_contract.contract_id, &crc_contract);
                writes += 1;
            }
            types::ContractData::CapacityReservationContract(crc) => {
                // update the active node contracts
                let mut contracts = ActiveNodeContracts::<T>::get(crc.node_id);
                reads += 1;
                contracts.push(contract.contract_id);
                ActiveNodeContracts::<T>::insert(crc.node_id, &contracts);
                writes += 1;
                // update the used resources of the node
                let mut node = pallet_tfgrid::Nodes::<T>::get(crc.node_id).unwrap();
                reads += 1;
                node.resources.used_resources = node
                    .resources
                    .used_resources
                    .add(&crc.resources.total_resources);
                pallet_tfgrid::Nodes::<T>::insert(node.id, &node);
                writes += 1;
            }
        }
        reads += 1;
    }
    (reads, writes)
}

pub fn translate_contract_objects<T: Config>() -> u32 {
    let mut count = 0;
    Contracts::<T>::translate::<deprecated::ContractV6<T>, _>(|k, ctr| {
        let contract_type = match ctr.contract_type {
            deprecated::ContractData::NodeContract(nc) => {
                let used_resources = NodeContractResources::<T>::get(ctr.contract_id).used;
                let mut crc_id = ActiveRentContractForNode::<T>::get(nc.node_id).unwrap_or(0);
                // if there is no rent contract for it create a capacity reservation contract that consumes the required resources
                // else use the rent contract id as capacity reservation contract id
                if crc_id == 0 {
                    let billing_index = find_bill_index::<T>(ctr.contract_id).unwrap_or(0);
                    crc_id = create_capacity_reservation::<T>(
                        nc.node_id,
                        ctr.twin_id,
                        ctr.state.clone(),
                        used_resources,
                        ctr.solution_provider_id,
                        billing_index,
                    );
                }
                // create the deployment contract
                let deployment_contract = types::DeploymentContract {
                    capacity_reservation_id: crc_id,
                    deployment_hash: nc.deployment_hash,
                    deployment_data: nc.deployment_data,
                    public_ips: nc.public_ips,
                    public_ips_list: nc.public_ips_list,
                    resources: used_resources,
                };
                types::ContractData::DeploymentContract(deployment_contract)
            }
            deprecated::ContractData::NameContract(nc) => {
                types::ContractData::NameContract(types::NameContract { name: nc.name })
            }
            deprecated::ContractData::RentContract(rc) => {
                let node = pallet_tfgrid::Nodes::<T>::get(rc.node_id).unwrap();
                let capacity_reservation_contract = types::CapacityReservationContract {
                    node_id: rc.node_id,
                    deployment_contracts: vec![],
                    public_ips: 0,
                    resources: ConsumableResources {
                        total_resources: node.resources.total_resources,
                        used_resources: Resources::empty(),
                    },
                    group_id: None,
                };
                types::ContractData::CapacityReservationContract(capacity_reservation_contract)
            }
        };
        let new_contract = types::Contract {
            version: CONTRACT_VERSION,
            state: ctr.state,
            contract_id: ctr.contract_id,
            twin_id: ctr.twin_id,
            contract_type: contract_type,
            solution_provider_id: ctr.solution_provider_id,
        };
        info!("Contract: {:?} succesfully migrated", k);
        count += 1;
        Some(new_contract)
    });
    count
}

pub fn find_bill_index<T: Config>(contract_id: u64) -> Option<u64> {
    for index in 0..BillingFrequency::<T>::get() {
        for ctr_id in ContractsToBillAt::<T>::get(index) {
            if ctr_id == contract_id {
                return Some(index);
            }
        }
    }
    None
}

pub fn create_capacity_reservation<T: Config>(
    node_id: u32,
    twin_id: u32,
    state: types::ContractState,
    resources: Resources,
    solution_provider_id: Option<u64>,
    billing_index: u64,
) -> u64 {
    let mut ctr_id = ContractID::<T>::get();
    ctr_id = ctr_id + 1;

    let capacity_reservation_contract = types::CapacityReservationContract {
        node_id: node_id,
        deployment_contracts: vec![],
        public_ips: 0,
        resources: ConsumableResources {
            total_resources: resources.clone(),
            used_resources: resources.clone(),
        },
        group_id: None,
    };

    let contract = types::Contract::<T> {
        version: CONTRACT_VERSION,
        state: state,
        contract_id: ctr_id,
        twin_id: twin_id,
        contract_type: types::ContractData::CapacityReservationContract(
            capacity_reservation_contract,
        ),
        solution_provider_id: solution_provider_id,
    };

    Contracts::<T>::insert(ctr_id, &contract);

    let now = <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;
    let mut contract_lock = types::ContractLock::default();
    contract_lock.lock_updated = now;
    ContractLock::<T>::insert(ctr_id, contract_lock);

    ContractID::<T>::put(ctr_id);

    // insert billing information
    let now = <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;
    let contract_billing_information = types::ContractBillingInformation {
        last_updated: now,
        amount_unbilled: 0,
        previous_nu_reported: 0,
    };

    // insert in contracts to bill
    ContractBillingInformationByID::<T>::insert(contract.contract_id, contract_billing_information);
    let mut contracts_to_bill_at = ContractsToBillAt::<T>::get(billing_index);
    contracts_to_bill_at.push(ctr_id);
    ContractsToBillAt::<T>::insert(billing_index, &contracts_to_bill_at);

    ctr_id
}
