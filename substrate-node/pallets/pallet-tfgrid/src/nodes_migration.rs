use super::Config;
use super::*;
use frame_support::{traits::Get, weights::Weight};
use log::info;
use sp_std::collections::btree_map::BTreeMap;

#[cfg(feature = "try-runtime")]
use frame_support::traits::OnRuntimeUpgradeHelpersExt;
#[cfg(feature = "try-runtime")]
use sp_runtime::SaturatedConversion;

pub mod v9 {
    use super::*;
    use crate::Config;

    use frame_support::{pallet_prelude::Weight, traits::OnRuntimeUpgrade};
    use sp_std::marker::PhantomData;
    pub struct FixFarmNodeIndexMap<T: Config>(PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for FixFarmNodeIndexMap<T> {
        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<(), &'static str> {
            assert!(PalletVersion::<T>::get() == types::StorageVersion::V9Struct);

            // Store number of nodes in temp storage
            let nodes_count: u64 = Nodes::<T>::iter_keys().count().saturated_into();
            Self::set_temp_storage(nodes_count, "pre_nodes_count");
            log::info!(
                "🔎 FixFarmingPolicy pre migration: Number of existing nodes {:?}",
                nodes_count
            );

            info!("👥  TFGrid pallet to V10 passes PRE migrate checks ✅",);
            Ok(())
        }

        fn on_runtime_upgrade() -> Weight {
            if PalletVersion::<T>::get() == types::StorageVersion::V9Struct {
                add_farm_nodes_index::<T>()
            } else {
                info!(" >>> Unused migration");
                return 0;
            }
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade() -> Result<(), &'static str> {
            assert!(PalletVersion::<T>::get() >= types::StorageVersion::V10Struct);

            // Check number of nodes against pre-check result
            let pre_nodes_count = Self::get_temp_storage("pre_nodes_count").unwrap_or(0u64);
            assert_eq!(
                Nodes::<T>::iter().count().saturated_into::<u64>(),
                pre_nodes_count,
                "Number of nodes migrated does not match"
            );

            info!(
                "👥  TFGrid pallet migration to {:?} passes POST migrate checks ✅",
                Pallet::<T>::pallet_version()
            );

            Ok(())
        }
    }
}

pub fn add_farm_nodes_index<T: Config>() -> frame_support::weights::Weight {
    info!(" >>> Migrating nodes storage...");

    NodesByFarmID::<T>::remove_all(None);

    let mut reads = 0;
    let mut writes = 0;

    let mut farms_with_nodes: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for (_, node) in Nodes::<T>::iter() {
        // Add index of farm - list (nodes)
        farms_with_nodes
            .entry(node.farm_id)
            .or_insert(vec![])
            .push(node.id);

        reads += 1;
    }

    for (farm_id, nodes) in farms_with_nodes.iter() {
        info!(
            "inserting nodes: {:?} with farm id: {:?}",
            nodes.clone(),
            farm_id
        );
        NodesByFarmID::<T>::insert(farm_id, nodes);
        writes += 1;
    }

    // Update pallet storage version
    PalletVersion::<T>::set(types::StorageVersion::V10Struct);
    info!(" <<< Storage version upgraded");

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(reads as Weight, writes as Weight)
}
