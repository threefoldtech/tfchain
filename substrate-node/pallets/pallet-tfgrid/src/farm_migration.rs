use super::*;
use frame_support::weights::Weight;
use tfchain_support::types::FarmCertification;

pub mod deprecated {
    use crate::Config;
    use codec::{Decode, Encode};
    use frame_support::decl_module;
    use sp_std::prelude::*;
    use tfchain_support::types::PublicIP;

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
    pub struct FarmV2 {
        pub version: u32,
        pub id: u32,
        pub name: Vec<u8>,
        pub twin_id: u32,
        pub pricing_policy_id: u32,
        pub certification_type: CertificationType,
        pub public_ips: Vec<PublicIP>,
        pub dedicated_farm: bool,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug)]
    pub enum CertificationType {
        Diy,
        Certified,
    }

    impl Default for CertificationType {
        fn default() -> CertificationType {
            CertificationType::Diy
        }
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::Origin { }
    }
}

pub fn rework_farm_certification<T: Config>() -> frame_support::weights::Weight {
    frame_support::debug::RuntimeLogger::init();

    if PalletVersion::get() == types::StorageVersion::V3Struct {
        frame_support::debug::info!(
            " >>> Starting migration, pallet version: {:?}",
            PalletVersion::get()
        );
        let count = Farms::iter().count();
        frame_support::debug::info!(" >>> Updating Farms storage. Migrating {} Farms...", count);

        let mut migrated_count = 0;
        // We transform the storage values from the old into the new format.
        Farms::translate::<deprecated::FarmV2, _>(|k, farm| {
            frame_support::debug::info!("     Migrated farm for {:?}...", k);

            let new_farm = Farm {
                version: 3,
                id: farm.id,
                name: farm.name,
                twin_id: farm.twin_id,
                pricing_policy_id: farm.pricing_policy_id,
                certification: FarmCertification::NotCertified,
                public_ips: farm.public_ips,
                farming_policy_limits: None,
            };

            migrated_count += 1;

            Some(new_farm)
        });
        frame_support::debug::info!(
            " <<< Farm storage updated! Migrated {} farms ✅",
            migrated_count
        );

        // Update pallet storage version
        PalletVersion::set(types::StorageVersion::V4Struct);
        frame_support::debug::info!(" <<< Storage version upgraded");

        // Return the weight consumed by the migration.
        T::DbWeight::get().reads_writes(migrated_count as Weight + 1, migrated_count as Weight + 1)
    } else {
        frame_support::debug::info!(" >>> Unused migration");
        return 0;
    }
}
