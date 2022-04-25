use super::*;

pub struct RemoveCollectiveFlip;
impl frame_support::traits::OnRuntimeUpgrade for RemoveCollectiveFlip {
    fn on_runtime_upgrade() -> Weight {
        use frame_support::storage::migration;
        // Remove the storage value `RandomMaterial` from removed pallet `RandomnessCollectiveFlip`
        migration::remove_storage_prefix(b"RandomnessCollectiveFlip", b"RandomMaterial", b"");
        <Runtime as frame_system::Config>::DbWeight::get().writes(1)
    }
}

/// Migrate from `PalletVersion` to the new `StorageVersion`
pub struct MigratePalletVersionToStorageVersion;
impl frame_support::traits::OnRuntimeUpgrade for MigratePalletVersionToStorageVersion {
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        frame_support::migrations::migrate_from_pallet_version_to_storage_version::<
            AllPalletsWithSystem,
        >(&RocksDbWeight::get())
    }
}

/// 1. SystemToDualRefCount: from `unique`  to `dual` reference counting.
/// 2. frame_system::Pallet<System>(automatically): from `dual` to dual `triple` reference counting.
pub struct SystemToDualRefCount;
impl frame_support::traits::OnRuntimeUpgrade for SystemToDualRefCount {
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        frame_system::migrations::migrate_to_dual_ref_count::<Runtime>()
    }
}

pub struct SystemToTripleRefCount;
impl frame_support::traits::OnRuntimeUpgrade for SystemToTripleRefCount {
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        frame_system::migrations::migrate_to_triple_ref_count::<Runtime>()
    }
}

pub struct GrandpaStoragePrefixMigration;
impl frame_support::traits::OnRuntimeUpgrade for GrandpaStoragePrefixMigration {
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        use frame_support::traits::PalletInfo;
        let name = <Runtime as frame_system::Config>::PalletInfo::name::<Grandpa>()
            .expect("grandpa is part of pallets in construct_runtime, so it has a name; qed");
        pallet_grandpa::migrations::v4::migrate::<Runtime, &str>(name)
    }
}

const COUNCIL_OLD_PREFIX: &str = "Instance1Collective";
/// Migrate from `Instance1Collective` to the new pallet prefix `Council`
pub struct CouncilStoragePrefixMigration;
impl frame_support::traits::OnRuntimeUpgrade for CouncilStoragePrefixMigration {
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        pallet_collective::migrations::v4::migrate::<Runtime, Council, _>(COUNCIL_OLD_PREFIX)
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        pallet_collective::migrations::v4::pre_migrate::<Council, _>(COUNCIL_OLD_PREFIX);
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        pallet_collective::migrations::v4::post_migrate::<Council, _>(COUNCIL_OLD_PREFIX);
        Ok(())
    }
}

const COUNCIL_MEMBERSHIP_OLD_PREFIX: &str = "Instance1Membership";
/// Migrate from `Instance1Membership` to the new pallet prefix `TechnicalMembership`
pub struct CouncilMembershipStoragePrefixMigration;
impl frame_support::traits::OnRuntimeUpgrade for CouncilMembershipStoragePrefixMigration {
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        use frame_support::traits::PalletInfo;
        let name = <Runtime as frame_system::Config>::PalletInfo::name::<CouncilMembership>()
            .expect("CouncilMembership is part of runtime, so it has a name; qed");
        pallet_membership::migrations::v4::migrate::<Runtime, CouncilMembership, _>(
            COUNCIL_MEMBERSHIP_OLD_PREFIX,
            name,
        )
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        use frame_support::traits::PalletInfo;
        let name = <Runtime as frame_system::Config>::PalletInfo::name::<CouncilMembership>()
            .expect("CouncilMembership is part of runtime, so it has a name; qed");
        pallet_membership::migrations::v4::pre_migrate::<CouncilMembership, _>(
            COUNCIL_MEMBERSHIP_OLD_PREFIX,
            name,
        );
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        use frame_support::traits::PalletInfo;
        let name = <Runtime as frame_system::Config>::PalletInfo::name::<CouncilMembership>()
            .expect("CouncilMembership is part of runtime, so it has a name; qed");
        pallet_membership::migrations::v4::post_migrate::<CouncilMembership, _>(
            COUNCIL_MEMBERSHIP_OLD_PREFIX,
            name,
        );
        Ok(())
    }
}

use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};
pub struct CustomOnRuntimeUpgrades;
impl OnRuntimeUpgrade for CustomOnRuntimeUpgrades {
    fn on_runtime_upgrade() -> Weight {
        let mut weight = 0;

        // 1. RemoveCollectiveFlip
        frame_support::log::info!("🔍️ RemoveCollectiveFlip start");
        weight += <RemoveCollectiveFlip as OnRuntimeUpgrade>::on_runtime_upgrade();
        frame_support::log::info!("🚀 RemoveCollectiveFlip end");

        // 2. MigratePalletVersionToStorageVersion
        frame_support::log::info!("🔍️ MigratePalletVersionToStorageVersion start");
        weight += <MigratePalletVersionToStorageVersion as OnRuntimeUpgrade>::on_runtime_upgrade();
        frame_support::log::info!("🚀 MigratePalletVersionToStorageVersion end");

        // 3. SystemToDualRefCount
        frame_support::log::info!("🔍️ SystemToDualRefCount start");
        weight += <SystemToDualRefCount as OnRuntimeUpgrade>::on_runtime_upgrade();
        frame_support::log::info!("🚀 SystemToDualRefCount end");

        // 4. SystemToTripleRefCount
        frame_support::log::info!("🔍️ SystemToTripleRefCount start");
        weight += <SystemToTripleRefCount as OnRuntimeUpgrade>::on_runtime_upgrade();
        frame_support::log::info!("🚀 SystemToTripleRefCount end");

        // 5. GrandpaStoragePrefixMigration
        frame_support::log::info!("🔍️ GrandpaStoragePrefixMigration start");
        frame_support::traits::StorageVersion::new(0).put::<Grandpa>();
        weight += <GrandpaStoragePrefixMigration as OnRuntimeUpgrade>::on_runtime_upgrade();
        frame_support::log::info!("🚀 GrandpaStoragePrefixMigration end");

        // 6. CouncilStoragePrefixMigration
        frame_support::log::info!("🔍️ CouncilStoragePrefixMigration start");
        frame_support::traits::StorageVersion::new(0).put::<Council>();
        weight += <CouncilStoragePrefixMigration as OnRuntimeUpgrade>::on_runtime_upgrade();
        frame_support::log::info!("🚀 CouncilStoragePrefixMigration end");

        // 7. CouncilMembershipStoragePrefixMigration
        frame_support::log::info!("🔍️ CouncilMembershipStoragePrefixMigration start");
        frame_support::traits::StorageVersion::new(0).put::<CouncilMembership>();
        weight +=
            <CouncilMembershipStoragePrefixMigration as OnRuntimeUpgrade>::on_runtime_upgrade();
        frame_support::log::info!("🚀 CouncilMembershipStoragePrefixMigration end");

        weight
    }
}