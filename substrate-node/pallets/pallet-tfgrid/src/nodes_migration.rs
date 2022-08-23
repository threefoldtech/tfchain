use super::Config;
use super::*;
use frame_support::{traits::Get, weights::Weight};
use log::info;

pub mod v7patch {
    use super::*;
    use crate::Config;

    use frame_support::{pallet_prelude::Weight, traits::OnRuntimeUpgrade};
    use sp_std::marker::PhantomData;
    pub struct NodesMigration<T: Config>(PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for NodesMigration<T> {
        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<(), &'static str> {
            assert!(PalletVersion::<T>::get() == types::StorageVersion::V6Struct);

            info!("👥  TFGrid pallet to v4 passes PRE migrate checks ✅",);
            Ok(())
        }

        fn on_runtime_upgrade() -> Weight {
            if PalletVersion::<T>::get() == types::StorageVersion::V6Struct {
                add_farm_nodes_index::<T>()
            } else {
                info!(" >>> Unused migration");
                return 0;
            }
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade() -> Result<(), &'static str> {
            assert!(PalletVersion::<T>::get() == types::StorageVersion::V7Struct);

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

    let mut reads = 0;

    for (_, node) in Nodes::<T>::iter() {
        // Add index of farm - list (nodes)
        let mut nodes_by_farm_id = NodesByFarmID::<T>::get(node.farm_id);
        nodes_by_farm_id.push(node.id);
        NodesByFarmID::<T>::insert(node.farm_id, nodes_by_farm_id);

        reads += 1;
    }

    // Update pallet storage version
    PalletVersion::<T>::set(types::StorageVersion::V7Struct);
    info!(" <<< Storage version upgraded");

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(reads as Weight + 1, 0)
}
