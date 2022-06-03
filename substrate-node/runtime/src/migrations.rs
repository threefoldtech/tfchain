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

pub struct OldFarmingPolicyRemoval;
impl frame_support::traits::OnRuntimeUpgrade for OldFarmingPolicyRemoval {
    fn on_runtime_upgrade() -> Weight {
        use frame_support::storage::migration;
        // Remove the storage value `FarmingPolicies` from removed pallet `TfgridModule`
        migration::remove_storage_prefix(b"TfgridModule", b"FarmingPolicies", b"");
        // Remove unused FarmingPolicyIDsByCertificationType
        migration::remove_storage_prefix(
            b"TfgridModule",
            b"FarmingPolicyIDsByCertificationType",
            b"",
        );
        // Remove unused CertificationCodes
        migration::remove_storage_prefix(b"TfgridModule", b"CertificationCodes", b"");
        // Remove unused CertificationCodeIdByName
        migration::remove_storage_prefix(b"TfgridModule", b"CertificationCodeIdByName", b"");
        // Remove unused CertificationCodeID
        migration::remove_storage_prefix(b"TfgridModule", b"CertificationCodeID", b"");
        <Runtime as frame_system::Config>::DbWeight::get().writes(1)
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

        // 3. OldFarmingPolicyRemoval
        frame_support::debug::info!("🔍️ OldFarmingPolicyRemoval start");
        weight += <OldFarmingPolicyRemoval as OnRuntimeUpgrade>::on_runtime_upgrade();
        frame_support::debug::info!("🚀 OldFarmingPolicyRemoval end");

        weight
    }
}
