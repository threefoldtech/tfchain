use super::*;
use frame_support::weights::Weight;

pub mod deprecated {
    use codec::{Decode, Encode};
    use crate::Config;
    use frame_support::{decl_module};
    use sp_std::prelude::*;

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
    pub struct NodeV2 {
        pub version: u32,
        pub id: u32,
        pub farm_id: u32,
        pub twin_id: u32,
        pub resources: super::types::Resources,
        pub location: super::types::Location,
        pub country: Vec<u8>,
        pub city: Vec<u8>,
        // optional public config
        pub public_config: Option<super::types::PublicConfig>,
        pub created: u64,
        pub farming_policy_id: u32,
        pub interfaces: Vec<super::types::Interface>,
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::Origin { }
    }
}

pub fn migrate_to_v3<T: Config>() -> frame_support::weights::Weight {
    frame_support::debug::RuntimeLogger::init();

    let version = PalletVersion::get();
    frame_support::debug::info!(" >>> Starting migration, pallet version: {:?}", version);

    // Storage migrations should use storage versions for safety.
    if PalletVersion::get() == super::types::StorageVersion::V2Struct {
        let count = Nodes::iter().count();
        frame_support::debug::info!(" >>> Updating Nodes storage. Migrating {} nodes...", count);

        let mut migrated_count = 0;
        // We transform the storage values from the old into the new format.
        Nodes::translate::<deprecated::NodeV2, _>(
            |k, node| {
                frame_support::debug::info!("     Migrated node for {:?}...", k);

                let new_node = super::types::Node {
                    version: node.version,
                    id: node.id,
                    farm_id: node.farm_id,
                    twin_id: node.twin_id,
                    resources: node.resources,
                    location: node.location,
                    country: node.country,
                    city: node.city,
                    public_config: node.public_config,
                    created: node.created,
                    farming_policy_id: node.farming_policy_id,
                    interfaces: node.interfaces,
                    certification_type: super::types::CertificationType::Diy,
                    secure_boot: false,
                    virtualized: false,
                    serial_number: Vec::new(),
                };

                migrated_count+=1;
                Some(new_node)
            }
        );

        // Update storage version.
        PalletVersion::put(super::types::StorageVersion::V3Struct);
        frame_support::debug::info!(" <<< Pallet tfgrid storage updated! Migrated {} nodes ✅", migrated_count);

        // Return the weight consumed by the migration.
        T::DbWeight::get().reads_writes(migrated_count as Weight + 1, migrated_count as Weight + 1)
    } else {
        frame_support::debug::info!(" >>> Unused migration!");
        0
    }
}