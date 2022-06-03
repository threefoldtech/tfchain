use crate::types::FarmingPolicy;

use super::*;
use frame_support::weights::Weight;
use tfchain_support::types::Certification;

pub mod deprecated {
    use crate::Config;
    use codec::{Decode, Encode};
    use frame_support::decl_module;
    use sp_std::prelude::*;
    use tfchain_support::types::CertificationType;

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
    pub struct FarmingPolicyV1 {
        pub version: u32,
        pub id: u32,
        pub name: Vec<u8>,
        pub cu: u32,
        pub su: u32,
        pub nu: u32,
        pub ipv4: u32,
        pub timestamp: u64,
        pub certification_type: CertificationType,
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::Origin { }
    }
}

pub fn rework_farm_certification<T: Config>() -> frame_support::weights::Weight {
    frame_support::debug::RuntimeLogger::init();

    if PalletVersion::get() == types::StorageVersion::V5Struct {
        frame_support::debug::info!(
            " >>> Starting migration, pallet version: {:?}",
            PalletVersion::get()
        );
        let count = FarmingPolicies::<T>::iter().count();
        frame_support::debug::info!(
            " >>> Updating Farming Policies storage. Migrating {} Farming Policies...",
            count
        );

        let mut migrated_count = 0;
        // We transform the storage values from the old into the new format.
        FarmingPolicies::<T>::translate::<deprecated::FarmingPolicyV1, _>(|k, policy| {
            frame_support::debug::info!("     Migrated policy for {:?}...", k);

            let new_farming_policy: FarmingPolicy<T::BlockNumber> = FarmingPolicy {
                version: 3,
                id: policy.id,
                name: policy.name,
                cu: policy.cu,
                su: policy.su,
                nu: policy.nu,
                ipv4: policy.ipv4,
                minimal_uptime: 0,
                policy_created: system::Pallet::<T>::block_number(),
                policy_end: system::Pallet::<T>::block_number(),
                immutable: false,
                default: false,
                node_certification: CertificationType::Diy,
                farm_certification: Certification::Gold,
            };

            migrated_count += 1;

            Some(new_farming_policy)
        });
        frame_support::debug::info!(
            " <<< Farming Policy storage updated! Migrated {} policies ✅",
            migrated_count
        );

        // Update pallet storage version
        PalletVersion::set(types::StorageVersion::V6Struct);
        frame_support::debug::info!(" <<< Storage version upgraded");

        // Return the weight consumed by the migration.
        T::DbWeight::get().reads_writes(migrated_count as Weight + 1, migrated_count as Weight + 1)
    } else {
        frame_support::debug::info!(" >>> Unused migration");
        return 0;
    }
}
