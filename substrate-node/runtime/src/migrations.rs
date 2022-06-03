use super::*;

use pallet_tfgrid;

pub struct FarmingMigration;
impl frame_support::traits::OnRuntimeUpgrade for FarmingMigration {
    fn on_runtime_upgrade() -> Weight {
        pallet_tfgrid::farm_migration::rework_farm_certification::<Runtime>()
    }
}

pub struct NodeMigration;
impl frame_support::traits::OnRuntimeUpgrade for NodeMigration {
    fn on_runtime_upgrade() -> Weight {
        pallet_tfgrid::node_migration::add_connection_price_to_nodes::<Runtime>()
    }
}

pub struct FarmingPolicyMigration;
impl frame_support::traits::OnRuntimeUpgrade for FarmingPolicyMigration {
    fn on_runtime_upgrade() -> Weight {
        pallet_tfgrid::farming_policy_migration::rework_farm_certification::<Runtime>()
    }
}

use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};
pub struct CustomOnRuntimeUpgrades;
impl OnRuntimeUpgrade for CustomOnRuntimeUpgrades {
    fn on_runtime_upgrade() -> Weight {
        let mut weight = 0;

        // 1. FarmingMigration
        frame_support::debug::info!("🔍️ FarmingMigration start");
        weight += <FarmingMigration as OnRuntimeUpgrade>::on_runtime_upgrade();
        frame_support::debug::info!("🚀 FarmingMigration end");

        // 2. NodeMigration
        frame_support::debug::info!("🔍️ NodeMigration start");
        weight += <NodeMigration as OnRuntimeUpgrade>::on_runtime_upgrade();
        frame_support::debug::info!("🚀 NodeMigration end");

        // 3. FarmingPolicyMigration
        frame_support::debug::info!("🔍️ FarmingPolicyMigration start");
        weight += <FarmingPolicyMigration as OnRuntimeUpgrade>::on_runtime_upgrade();
        frame_support::debug::info!("🚀 FarmingPolicyMigration end");

        weight
    }
}
