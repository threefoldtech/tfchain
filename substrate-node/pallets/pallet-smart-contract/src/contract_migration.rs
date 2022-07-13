use super::*;
use frame_support::weights::Weight;
use sp_core::H256;

pub mod deprecated {
    use crate::Config;
    use codec::{Decode, Encode};
    use frame_support::decl_module;
    use sp_std::prelude::*;
    use scale_info::TypeInfo;
    use super::types;
    use sp_std::vec::Vec;
    use tfchain_support::types::{PublicIP};

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
    pub struct ContractV3 {
        pub version: u32,
        pub state: types::ContractState,
        pub contract_id: u64,
        pub twin_id: u32,
        pub contract_type: ContractData,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
    pub struct NodeContract {
        pub node_id: u32,
        // deployment_data is the encrypted deployment body. This encrypted the deployment with the **USER** public key.
        // So only the user can read this data later on (or any other key that he keeps safe).
        // this data part is read only by the user and can actually hold any information to help him reconstruct his deployment or can be left empty.
        pub deployment_data: Vec<u8>,
        // Hash of the deployment, set by the user
        // Max 32 bytes
        pub deployment_hash: Vec<u8>,
        pub public_ips: u32,
        pub public_ips_list: Vec<PublicIP>,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
    pub struct NameContract {
        pub name: Vec<u8>,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
    pub struct RentContract {
        pub node_id: u32,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug, TypeInfo)]
    pub enum ContractData {
        NodeContract(NodeContract),
        NameContract(NameContract),
        RentContract(RentContract),
    }

    impl Default for ContractData {
        fn default() -> ContractData {
            ContractData::NodeContract(NodeContract::default())
        }
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::Origin { }
    }
}

pub fn migrate_to_version_4<T: Config>() -> frame_support::weights::Weight {
    if PalletVersion::<T>::get() == types::StorageVersion::V3 {
        frame_support::log::info!(
            " >>> Starting migration, pallet version: {:?}",
            PalletVersion::<T>::get()
        );
        let count = Contracts::<T>::iter().count();
        frame_support::log::info!(
            " >>> Updating Contracts storage. Migrating {} Contracts...",
            count
        );

        let mut migrated_count = 0;
        // We transform the storage values from the old into the new format.
        Contracts::<T>::translate::<deprecated::ContractV3, _>(|k, ctr| {
            frame_support::log::info!("     Migrated contract for {:?}...", k);

            let mut new_contract = types::Contract {
                version: 4,
                state: ctr.state,
                contract_id: ctr.contract_id,
                twin_id: ctr.twin_id,
                contract_type: types::ContractData::default(),
            };

            match ctr.contract_type {
                deprecated::ContractData::NodeContract(node_contract) => {
                    let mut new_node_contract = types::NodeContract {
                        node_id: node_contract.node_id,
                        deployment_data: node_contract.deployment_data,
                        deployment_hash: H256::zero(),
                        public_ips: node_contract.public_ips,
                        public_ips_list: node_contract.public_ips_list,
                    };

                    // If it's a valid 32 byte hash, transform it as a H256 and save it on the node contract
                    if node_contract.deployment_hash.len() == 32 {
                        new_node_contract.deployment_hash =
                            sp_core::H256::from_slice(&node_contract.deployment_hash);
                    };

                    new_contract.contract_type =
                        types::ContractData::NodeContract(new_node_contract);
                }
                deprecated::ContractData::NameContract(nc) => {
                    let name_c = types::NameContract {
                        name: nc.name
                    };
                    new_contract.contract_type = types::ContractData::NameContract(name_c);
                },
                deprecated::ContractData::RentContract(rc) => {
                    let rent_c = types::RentContract {
                        node_id: rc.node_id
                    };
                    new_contract.contract_type = types::ContractData::RentContract(rent_c);
                }
            };

            migrated_count += 1;

            Some(new_contract)
        });
        frame_support::log::info!(
            " <<< Contracts storage updated! Migrated {} Contracts ✅",
            migrated_count
        );

        // Update pallet storage version
        PalletVersion::<T>::set(types::StorageVersion::V4);
        frame_support::log::info!(" <<< Storage version upgraded");

        // Return the weight consumed by the migration.
        T::DbWeight::get().reads_writes(migrated_count as Weight + 1, migrated_count as Weight + 1)
    } else {
        frame_support::log::info!(" >>> Unused migration");
        return 0;
    }
}
