use super::*;
use frame_support::weights::Weight;

pub mod deprecated {
    use crate::Config;
    use frame_support::decl_module;
    use sp_std::prelude::*;

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
