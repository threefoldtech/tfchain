use super::*;
use frame_support::{weights::Weight, BoundedVec};
use log::info;
use pallet_tfgrid;
use sp_core::H256;
use sp_std::convert::{TryFrom, TryInto};
use tfchain_support::types::PublicIP;

pub mod deprecated {
    use crate::types;
    use crate::Config;
    use codec::{Decode, Encode};
    use frame_support::decl_module;
    use scale_info::TypeInfo;
    use sp_std::prelude::*;
    use sp_std::vec::Vec;

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
    pub struct ContractV3 {
        pub version: u32,
        pub state: types::ContractState,
        pub contract_id: u64,
        pub twin_id: u32,
        pub contract_type: ContractData,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
    pub struct PublicIP {
        pub ip: Vec<u8>,
        pub gateway: Vec<u8>,
        pub contract_id: u64,
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

pub mod v4 {
    use super::*;
    use crate::Config;

    use frame_support::{pallet_prelude::Weight, traits::OnRuntimeUpgrade};
    use sp_std::marker::PhantomData;
    pub struct ContractMigrationV4<T: Config>(PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for ContractMigrationV4<T> {
        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<(), &'static str> {
            info!("current pallet version: {:?}", PalletVersion::<T>::get());
            assert!(PalletVersion::<T>::get() == types::StorageVersion::V3);

            info!("👥  Smart Contract pallet to v4 passes PRE migrate checks ✅",);
            Ok(())
        }

        fn on_runtime_upgrade() -> Weight {
            migrate_to_version_4::<T>()
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade() -> Result<(), &'static str> {
            info!("current pallet version: {:?}", PalletVersion::<T>::get());
            assert!(PalletVersion::<T>::get() == types::StorageVersion::V4);

            info!(
                "👥  Smart Contract pallet to {:?} passes POST migrate checks ✅",
                PalletVersion::<T>::get()
            );

            Ok(())
        }
    }
}

pub fn migrate_to_version_4<T: Config>() -> frame_support::weights::Weight {
    if PalletVersion::<T>::get() == types::StorageVersion::V3 {
        info!(
            " >>> Starting contract pallet migration, pallet version: {:?}",
            PalletVersion::<T>::get()
        );

        let mut migrated_count = 0;
        // We transform the storage values from the old into the new format.
        Contracts::<T>::translate::<deprecated::ContractV3, _>(|k, ctr| {
            // info!("     Migrated contract for {:?}...", k);

            // dummy default
            let rc = types::RentContract { node_id: 0 };

            let mut new_contract = types::Contract {
                version: 4,
                state: ctr.state,
                contract_id: ctr.contract_id,
                twin_id: ctr.twin_id,
                contract_type: types::ContractData::RentContract(rc),
                solution_provider_id: None,
            };

            match ctr.contract_type {
                deprecated::ContractData::NodeContract(node_contract) => {
                    info!("found node contract");
                    let mut public_ips_list: BoundedVec<
                        PublicIP<
                            <T as pallet_tfgrid::Config>::PublicIP,
                            <T as pallet_tfgrid::Config>::GatewayIP,
                        >,
                        pallet::MaxNodeContractPublicIPs,
                    > = vec![].try_into().unwrap();

                    let mut should_free_ip = false;
                    if node_contract.public_ips_list.len() > 0 {
                        for pub_ip in node_contract.public_ips_list {
                            let ip = match <T as pallet_tfgrid::Config>::PublicIP::try_from(
                                pub_ip.ip.clone(),
                            ) {
                                Ok(x) => x,
                                Err(err) => {
                                    info!("error while parsing ip: {:?}", err);
                                    should_free_ip = true;
                                    continue;
                                }
                            };

                            let gateway = match <T as pallet_tfgrid::Config>::GatewayIP::try_from(
                                pub_ip.gateway.clone(),
                            ) {
                                Ok(x) => x,
                                Err(err) => {
                                    info!("error while parsing gateway: {:?}", err);
                                    should_free_ip = true;
                                    continue;
                                }
                            };

                            let new_ip = PublicIP {
                                ip,
                                gateway,
                                contract_id: pub_ip.contract_id,
                            };

                            match public_ips_list.try_push(new_ip) {
                                Ok(()) => (),
                                Err(err) => {
                                    info!("error while pushing ip to contract ip list: {:?}", err);
                                    should_free_ip = true;
                                    continue;
                                }
                            }
                        }
                    }

                    let mut new_node_contract = types::NodeContract {
                        node_id: node_contract.node_id,
                        deployment_hash: H256::zero(),
                        public_ips: node_contract.public_ips,
                        public_ips_list,
                    };

                    if should_free_ip {
                        match pallet::Pallet::<T>::_free_ip(k, &mut new_node_contract) {
                            Ok(_) => info!("successfully freed ips for contract: {:?}", k),
                            Err(err) => info!("error occurred while freeing ip {:?}", err),
                        };
                    };

                    // If it's a valid 32 byte hash, transform it as a H256 and save it on the node contract
                    if node_contract.deployment_hash.len() == 32 {
                        new_node_contract.deployment_hash =
                            sp_core::H256::from_slice(&node_contract.deployment_hash);
                    }

                    // Save the contract data
                    new_contract.contract_type =
                        types::ContractData::NodeContract(new_node_contract);
                }
                deprecated::ContractData::NameContract(nc) => {
                    match super::NameContractNameOf::<T>::try_from(nc.name) {
                        Ok(ctr_name) => {
                            let name_c = types::NameContract { name: ctr_name };
                            new_contract.contract_type = types::ContractData::NameContract(name_c);
                        }
                        Err(err) => {
                            info!("error while parsing contract name: {:?}", err);
                            // If it's not a valid contract name, it's probably garbage. Cancel the contract
                            return None;
                        }
                    };
                }
                deprecated::ContractData::RentContract(rc) => {
                    let rent_c = types::RentContract {
                        node_id: rc.node_id,
                    };
                    new_contract.contract_type = types::ContractData::RentContract(rent_c);
                }
            };

            migrated_count += 1;

            info!("Contract: {:?} succesfully migrated", k);

            Some(new_contract)
        });
        info!(
            " <<< Contracts storage updated! Migrated {} Contracts ✅",
            migrated_count
        );

        // Update pallet storage version
        PalletVersion::<T>::set(types::StorageVersion::V4);
        info!(" <<< Storage version upgraded");

        // Return the weight consumed by the migration.
        T::DbWeight::get().reads_writes(migrated_count as Weight + 1, migrated_count as Weight + 1)
    } else {
        info!(" >>> Unused migration");
        return 0;
    }
}
