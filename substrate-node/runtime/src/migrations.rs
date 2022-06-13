use super::*;

use crate::sp_api_hidden_includes_construct_runtime::hidden_include::StorageMap;
use frame_system as system;
use pallet_tfgrid;
use tfchain_support::types::{FarmCertification, NodeCertification};

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

pub struct InsertNewFarmingPolicies;
impl frame_support::traits::OnRuntimeUpgrade for InsertNewFarmingPolicies {
    fn on_runtime_upgrade() -> Weight {
        // Create Policy 1 for
        // Non certified nodes / non certified farm
        let farming_policy_1: pallet_tfgrid::types::FarmingPolicy<BlockNumber> =
            pallet_tfgrid::types::FarmingPolicy {
                version: 1,
                id: 1,
                name: "farming_policy_non_certified_default".as_bytes().to_vec(),
                cu: 2400,
                su: 1000,
                nu: 30,
                ipv4: 5,
                minimal_uptime: 95,
                policy_created: system::Pallet::<Runtime>::block_number(),
                policy_end: 0,
                immutable: false,
                default: true,
                node_certification: NodeCertification::Diy,
                farm_certification: FarmCertification::NotCertified,
            };
        pallet_tfgrid::FarmingPoliciesMap::<Runtime>::insert(1, farming_policy_1);

        // Create policy 2 for
        // certified nodes / non certified farm
        let farming_policy_2: pallet_tfgrid::types::FarmingPolicy<BlockNumber> =
            pallet_tfgrid::types::FarmingPolicy {
                version: 1,
                id: 1,
                name: "farming_policy_certified_nodes_default".as_bytes().to_vec(),
                cu: 3000,
                su: 1250,
                nu: 38,
                ipv4: 6,
                minimal_uptime: 95,
                policy_created: system::Pallet::<Runtime>::block_number(),
                policy_end: 0,
                immutable: false,
                default: true,
                node_certification: NodeCertification::Certified,
                farm_certification: FarmCertification::NotCertified,
            };
        pallet_tfgrid::FarmingPoliciesMap::<Runtime>::insert(2, farming_policy_2);

        // Update internal ID
        pallet_tfgrid::FarmingPolicyID::put(2);

        <Runtime as frame_system::Config>::DbWeight::get().writes(3)
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

        // 4. InsertNewFarmingPolicies
        frame_support::debug::info!("🔍️ InsertNewFarmingPolicies start");
        weight += <InsertNewFarmingPolicies as OnRuntimeUpgrade>::on_runtime_upgrade();
        frame_support::debug::info!("🚀 InsertNewFarmingPolicies end");

        weight
    }
}
