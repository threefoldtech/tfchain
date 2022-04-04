use super::*;
use frame_support::weights::Weight;

pub mod deprecated {
    use codec::{Decode, Encode};
    use crate::Config;
    use frame_support::{decl_module};
    use sp_std::prelude::*;

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
    pub struct FarmV2 {
        pub version: u32,
        pub id: u32,
        pub name: Vec<u8>,
        pub twin_id: u32,
        pub pricing_policy_id: u32,
        pub certification_type: super::types::CertificationType,
        pub public_ips: Vec<super::types::PublicIP>,
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::Origin { }
    }
}

pub fn migrate_to_version_2<T: Config>() -> frame_support::weights::Weight {
    frame_support::debug::RuntimeLogger::init();

    let farm_1 = Farms::get(1);

    if farm_1.version == 2 {
        frame_support::debug::info!(" >>> Unused migration!");
        return 0
    }

    frame_support::debug::info!(" >>> Starting migration");

    let count = Farms::iter().count();
    frame_support::debug::info!(" >>> Updating Farms storage. Migrating {} farms...", count);

    let mut migrated_count = 0;
    // We transform the storage values from the old into the new format.
    Farms::translate::<deprecated::FarmV2, _>(
        |k, farm| {
            frame_support::debug::info!("     Migrated farm for {:?}...", k);

            let new_farm = super::types::Farm {
                // UPDATE VERSION TO 2
                version: 2,
                id: farm.id,
                name: farm.name,
                twin_id: farm.twin_id,
                pricing_policy_id: farm.pricing_policy_id,
                certification_type: farm.certification_type,
                public_ips: farm.public_ips,
                dedicated_farm: false
            };

            migrated_count+=1;
            Some(new_farm)
        }
    );

    frame_support::debug::info!(" <<< Pallet tfgrid storage updated! Migrated {} farms ✅", migrated_count);

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(migrated_count as Weight + 1, migrated_count as Weight + 1)
}