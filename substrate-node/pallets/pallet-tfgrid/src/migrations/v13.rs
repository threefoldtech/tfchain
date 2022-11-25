use crate::{
    types::StorageVersion, Config, InterfaceOf, LocationOf, Nodes, PalletVersion, PubConfigOf,
    SerialNumberOf, TFGRID_NODE_VERSION,
};
use frame_support::{pallet_prelude::Weight, traits::Get, traits::OnRuntimeUpgrade};
use log::info;
use pallet_timestamp as timestamp;
use sp_runtime::SaturatedConversion;
use sp_std::marker::PhantomData;
use tfchain_support::resources::Resources;
use tfchain_support::types::{ConsumableResources, Node, Power, PowerState, PowerTarget};

#[cfg(feature = "try-runtime")]
use frame_support::traits::OnRuntimeUpgradeHelpersExt;

pub mod deprecated {
    use crate::Config;
    use codec::{Decode, Encode};
    use frame_support::decl_module;

    use scale_info::TypeInfo;
    use sp_std::prelude::*;

    use tfchain_support::{resources::Resources, types::NodeCertification};

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
    pub struct NodeV12Struct<Location, PubConfig, If, SerialNumber> {
        pub version: u32,
        pub id: u32,
        pub farm_id: u32,
        pub twin_id: u32,
        pub resources: Resources,
        pub location: Location,
        // optional public config
        pub public_config: Option<PubConfig>,
        pub created: u64,
        pub farming_policy_id: u32,
        pub interfaces: Vec<If>,
        pub certification: NodeCertification,
        pub secure_boot: bool,
        pub virtualized: bool,
        pub serial_number: Option<SerialNumber>,
        pub connection_price: u32,
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::Origin { }
    }
}
pub struct NodeMigration<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for NodeMigration<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        info!(
            " --- Current TFGrid pallet version: {:?}",
            PalletVersion::<T>::get()
        );
        let nodes_count: u64 = Nodes::<T>::iter_keys().count() as u64;
        Self::set_temp_storage(nodes_count, "pre_node_count");
        log::info!(
            "🔎 NodeMigrationV13 pre migration: Number of existing nodes {:?}",
            nodes_count
        );
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
        deprecated::NodeV12Struct<LocationOf<T>, PubConfigOf<T>, InterfaceOf<T>, SerialNumberOf<T>>,
        _,
    >(|k, n| {
        migrated_count += 1;
        let migrated_contract = Node {
            version: TFGRID_NODE_VERSION,
            id: n.id,
            farm_id: n.farm_id,
            twin_id: n.twin_id,
            resources: ConsumableResources {
                total_resources: n.resources,
                used_resources: Resources::empty(),
            },
            location: n.location,
            power: Power {
                target: PowerTarget::Up,
                state: PowerState::Up,
                last_uptime: <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000,
            },
            // optional public config
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
        info!("Node: {:?} succesfully migrated", k);
        Some(migrated_contract)
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
