use super::Config;
use super::InterfaceOf;
use super::PubConfigOf;
use super::*;
use frame_support::{traits::Get, weights::Weight, BoundedVec};
use log::info;
use sp_std::collections::btree_map::BTreeMap;
use tfchain_support::types::Node;

pub mod deprecated {
    use codec::{Decode, Encode};
    use core::cmp::{Ord, PartialOrd};
    use scale_info::TypeInfo;
    use sp_std::prelude::*;
    use sp_std::vec::Vec;
    use tfchain_support::{resources::Resources, types::NodeCertification};

    #[derive(Encode, Decode, Debug, Default, PartialEq, Eq, Clone, TypeInfo)]
    pub struct EntityV9<AccountId> {
        pub version: u32,
        pub id: u32,
        pub name: Vec<u8>,
        pub account_id: AccountId,
        pub country: Vec<u8>,
        pub city: Vec<u8>,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
    pub struct NodeV9<PubConfig, If> {
        pub version: u32,
        pub id: u32,
        pub farm_id: u32,
        pub twin_id: u32,
        pub resources: Resources,
        pub location: Location,
        pub country: Vec<u8>,
        pub city: Vec<u8>,
        // optional public config
        pub public_config: Option<PubConfig>,
        pub created: u64,
        pub farming_policy_id: u32,
        pub interfaces: Vec<If>,
        pub certification: NodeCertification,
        pub secure_boot: bool,
        pub virtualized: bool,
        pub serial_number: Vec<u8>,
        pub connection_price: u32,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
    pub struct Location {
        pub longitude: Vec<u8>,
        pub latitude: Vec<u8>,
    }
}

pub mod v10 {
    use super::*;
    use crate::Config;

    use frame_support::{pallet_prelude::Weight, traits::OnRuntimeUpgrade};
    use sp_std::marker::PhantomData;
    pub struct GridMigration<T: Config>(PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for GridMigration<T> {
        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<(), &'static str> {
            assert!(PalletVersion::<T>::get() == types::StorageVersion::V9Struct);

            info!("👥  TFGrid pallet to v10 passes PRE migrate checks ✅",);
            Ok(())
        }

        fn on_runtime_upgrade() -> Weight {
            migrate_entities::<T>() + migrate_nodes::<T>()
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade() -> Result<(), &'static str> {
            assert!(PalletVersion::<T>::get() == types::StorageVersion::V10Struct);

            info!(
                "👥  TFGrid pallet migration to {:?} passes POST migrate checks ✅",
                Pallet::<T>::pallet_version()
            );

            Ok(())
        }
    }
}

pub fn migrate<T: Config>() -> frame_support::weights::Weight {
    if PalletVersion::<T>::get() == types::StorageVersion::V9Struct {
        migrate_entities::<T>() + migrate_nodes::<T>()
    } else {
        info!(" >>> Unused migration");
        0
    }
}

pub fn migrate_nodes<T: Config>() -> frame_support::weights::Weight {
    info!(" >>> Migrating nodes storage...");

    let mut migrated_count = 0;
    let mut writes = 0;
    let mut farms_with_nodes: BTreeMap<u32, Vec<u32>> = BTreeMap::new();

    // We transform the storage values from the old into the new format.
    Nodes::<T>::translate::<deprecated::NodeV9<PubConfigOf<T>, InterfaceOf<T>>, _>(|k, node| {
        info!("     Migrated node for {:?}...", k);

        // TODO: handle migration errors
        let location_input = LocationInput {
            city: BoundedVec::try_from(node.city).unwrap(),
            country: BoundedVec::try_from(node.country).unwrap(),
            latitude: BoundedVec::try_from(node.location.latitude).unwrap(),
            longitude: BoundedVec::try_from(node.location.longitude).unwrap(),
        };
        let location = <T as Config>::Location::try_from(location_input).unwrap();

        // TODO: handle migration errors
        let serial_number_input: SerialNumberInput =
            BoundedVec::try_from(node.serial_number).unwrap();
        let serial_number = <T as Config>::SerialNumber::try_from(serial_number_input).unwrap();

        let new_node = Node {
            version: 5,
            id: node.id,
            farm_id: node.farm_id,
            twin_id: node.twin_id,
            resources: node.resources,
            location,
            public_config: node.public_config,
            created: node.created,
            farming_policy_id: node.farming_policy_id,
            interfaces: node.interfaces,
            certification: node.certification,
            secure_boot: node.secure_boot,
            virtualized: node.virtualized,
            serial_number,
            connection_price: 80,
        };

        // Add index of farm - list (nodes)
        farms_with_nodes
            .entry(node.farm_id)
            .or_insert(vec![])
            .push(node.id);

        migrated_count += 1;

        Some(new_node)
    });
    info!(
        " <<< Node storage updated! Migrated {} nodes ✅",
        migrated_count
    );

    for (farm_id, nodes) in farms_with_nodes.iter() {
        info!(
            "inserting nodes: {:?} with farm id: {:?}",
            nodes.clone(),
            farm_id
        );
        NodesByFarmID::<T>::insert(farm_id, nodes);
        writes += 1;
    }

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(
        migrated_count as Weight + 1,
        migrated_count + writes as Weight + 1,
    )
}

pub fn migrate_entities<T: Config>() -> frame_support::weights::Weight {
    info!(" >>> Migrating entities storage...");
    0
}
