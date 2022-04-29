use super::*;
use frame_support::weights::Weight;

pub mod deprecated {
    use codec::{Decode, Encode};
    use crate::Config;
    use frame_support::{decl_module};
    use sp_std::prelude::*;

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
    pub struct NodeV3 {
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
        pub certification_type: super::types::CertificationType,
        pub secure_boot: bool,
        pub virtualized: bool,
        pub serial_number: Vec<u8>,
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::Origin { }
    }
}

pub fn add_node_ids_to_farm_id_map<T: Config>() -> frame_support::weights::Weight {
    frame_support::debug::RuntimeLogger::init();

    let version = PalletVersion::get();
    if version == types::StorageVersion::V4Struct {
        frame_support::debug::info!(" >>> Unused migration");
        return 0;
    }

    frame_support::debug::info!(" >>> Starting migration");

    // save number of read writes
    let mut read_writes = 0;

    let current_node_id = NodeID::get();

    for i in 1..current_node_id {
        let node = Nodes::get(i);
        let mut nodes = NodeIdsByFarmID::get(node.farm_id);
        nodes.push(i);
        NodeIdsByFarmID::insert(node.farm_id, nodes);
        frame_support::debug::info!(" >>> Insert node: {:?} in farm {:?} map", i, node.farm_id);
        read_writes += 3;
    };

    frame_support::debug::info!(" >>> Migration done");

    // Update storage version to v4
    PalletVersion::set(types::StorageVersion::V4Struct);

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(read_writes as Weight + 1, read_writes as Weight + 1)
}

pub fn add_connection_price_to_nodes<T: Config>() -> frame_support::weights::Weight {
    frame_support::debug::RuntimeLogger::init();

    let version = PalletVersion::get();
    if version == types::StorageVersion::V5Struct {
        frame_support::debug::info!(" >>> Unused migration");
        return 0;
    }

    frame_support::debug::info!(" >>> Starting migration");
    let count = Nodes::iter().count();
    frame_support::debug::info!(" >>> Updating Nodes storage. Migrating {} nodes...", count);

    let mut migrated_count = 0;
    // We transform the storage values from the old into the new format.
    Nodes::translate::<deprecated::NodeV3, _>(
        |k, node| {
            frame_support::debug::info!("     Migrated node for {:?}...", k);

            let new_node = super::types::Node {
                version: 4,
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
                secure_boot: node.secure_boot,
                virtualized: node.virtualized,
                serial_number: node.serial_number,
                connection_price: 800
            };

            migrated_count+=1;
            Some(new_node)
        }
    );

    frame_support::debug::info!(" <<< Node storage updated! Migrated {} nodes ✅", migrated_count);

    // Update pallet storage version
    PalletVersion::set(types::StorageVersion::V5Struct);
    frame_support::debug::info!(" <<< Storage version upgraded");

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(migrated_count as Weight + 1, migrated_count as Weight + 1)
}