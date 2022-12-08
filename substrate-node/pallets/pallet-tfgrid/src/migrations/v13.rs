use crate::{
    types::StorageVersion, Config, InterfaceOf, LocationOf, Pallet, PalletVersion, SerialNumberOf,
    TFGRID_NODE_VERSION,
};
use crate::{NodePower, NodeResources};
use frame_support::Blake2_128Concat;
use frame_support::{
    pallet_prelude::{OptionQuery, Weight},
    storage_alias,
    traits::Get,
    traits::OnRuntimeUpgrade,
};
use log::{debug, info};
use pallet_timestamp as timestamp;
use sp_runtime::SaturatedConversion;
use sp_std::marker::PhantomData;
use tfchain_support::resources::Resources;
use tfchain_support::types::{ConsumableResources, Power, PowerState, PowerTarget};

#[cfg(feature = "try-runtime")]
use frame_support::traits::OnRuntimeUpgradeHelpersExt;

// Storage alias from NodeV13 => write to this
#[storage_alias]
pub type Nodes<T: Config> = StorageMap<
    Pallet<T>,
    Blake2_128Concat,
    u32,
    super::types::v13::Node<LocationOf<T>, InterfaceOf<T>, SerialNumberOf<T>>,
    OptionQuery,
>;

// Storage alias from FarmV13 => write to this
#[storage_alias]
pub type Farms<T: Config> = StorageMap<
    Pallet<T>,
    Blake2_128Concat,
    u32,
    super::types::v13::Farm<<T as Config>::FarmName>,
    OptionQuery,
>;

pub struct NodeMigration<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for NodeMigration<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        assert!(PalletVersion::<T>::get() <= StorageVersion::V12Struct);

        info!(
            " --- Current TFGrid pallet version: {:?}",
            PalletVersion::<T>::get()
        );
        let nodes_count: u64 = Nodes::<T>::iter_keys().count() as u64;
        Self::set_temp_storage(nodes_count, "pre_node_count");
        debug!(
            "🔎 NodeMigrationV13 pre migration: Number of existing nodes {:?}",
            nodes_count
        );

        info!("👥  TFGrid pallet to V13 passes PRE migrate checks ✅",);
        Ok(())
    }

    fn on_runtime_upgrade() -> Weight {
        if PalletVersion::<T>::get() == StorageVersion::V12Struct {
            migrate_to_version_13::<T>()
        } else {
            info!(" >>> Unused TFGrid pallet V13 migration");
            0
        }
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        assert!(PalletVersion::<T>::get() >= StorageVersion::V13Struct);
        info!(
            " --- Current TFGrid pallet version: {:?}",
            PalletVersion::<T>::get()
        );
        // Check number of nodes against pre-check result
        let pre_nodes_count = Self::get_temp_storage("pre_node_count").unwrap_or(0u64);
        assert_eq!(
            Nodes::<T>::iter_keys().count() as u64,
            pre_nodes_count,
            "Number of nodes migrated does not match"
        );

        info!(
            "👥  TFGrid pallet to {:?} passes POST migrate checks ✅",
            PalletVersion::<T>::get()
        );

        Ok(())
    }
}

pub fn migrate_to_version_13<T: Config>() -> frame_support::weights::Weight {
    info!(
        " >>> Starting tfgrid pallet migration, pallet version: {:?}",
        PalletVersion::<T>::get()
    );

    let mut migrated_count = 0;

    Nodes::<T>::translate::<
        super::types::v12::Node<LocationOf<T>, InterfaceOf<T>, SerialNumberOf<T>>,
        _,
    >(|k, n| {
        let migrated_node =
            super::types::v13::Node::<LocationOf<T>, InterfaceOf<T>, SerialNumberOf<T>> {
                version: TFGRID_NODE_VERSION,
                id: n.id,
                farm_id: n.farm_id,
                twin_id: n.twin_id,
                location: n.location,
                public_config: n.public_config,
                created: n.created,
                farming_policy_id: n.farming_policy_id,
                interfaces: n.interfaces,
                certification: n.certification,
                secure_boot: n.secure_boot,
                virtualized: n.virtualized,
                serial_number: n.serial_number,
                connection_price: n.connection_price,
            };
        NodeResources::<T>::insert(
            n.id,
            &ConsumableResources {
                total_resources: n.resources,
                used_resources: Resources::empty(),
            },
        );
        NodePower::<T>::insert(
            n.id,
            &Power {
                target: PowerTarget::Up,
                state: PowerState::Up,
                last_uptime: <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000,
            },
        );

        migrated_count += 1;

        debug!("Node: {:?} succesfully migrated", k);
        Some(migrated_node)
    });

    info!(
        " <<< Node storage updated! Migrated {} Nodes ✅",
        migrated_count
    );

    // Update pallet storage version
    PalletVersion::<T>::set(StorageVersion::V13Struct);
    info!(
        " <<< Storage version TFGrid pallet upgraded to {:?}",
        PalletVersion::<T>::get()
    );

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(migrated_count as Weight + 1, migrated_count as Weight + 1)
}
