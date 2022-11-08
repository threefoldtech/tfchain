use super::Config;
use super::*;
use frame_support::{traits::Get, weights::Weight};
use log::info;

#[cfg(feature = "try-runtime")]
use frame_support::traits::OnRuntimeUpgradeHelpersExt;
#[cfg(feature = "try-runtime")]
use sp_runtime::SaturatedConversion;

pub mod v10 {
    use super::*;
    use crate::Config;

    use frame_support::{pallet_prelude::Weight, traits::OnRuntimeUpgrade};
    use sp_std::marker::PhantomData;
    pub struct FixFarmingPolicy<T: Config>(PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for FixFarmingPolicy<T> {
        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<(), &'static str> {
            assert!(PalletVersion::<T>::get() == types::StorageVersion::V10Struct);

            // Store number of farms in temp storage
            let farms_count: u64 = Farms::<T>::iter_keys().count().saturated_into();
            Self::set_temp_storage(farms_count, "pre_farms_count");
            log::info!(
                "🔎 FixFarmingPolicy pre migration: Number of existing farms {:?}",
                farms_count
            );

            info!("👥  TFGrid pallet to V10 passes PRE migrate checks ✅",);
            Ok(())
        }

        fn on_runtime_upgrade() -> Weight {
            if PalletVersion::<T>::get() == types::StorageVersion::V10Struct {
                fix_farming_policy_migration_::<T>()
            } else {
                info!(" >>> Unused migration");
                return 0;
            }
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade() -> Result<(), &'static str> {
            assert!(PalletVersion::<T>::get() == types::StorageVersion::V11Struct);

            // Check number of farms against pre-check result
            let pre_farms_count = Self::get_temp_storage("pre_farms_count").unwrap_or(0u64);
            assert_eq!(
                Farms::<T>::iter().count().saturated_into::<u64>(),
                pre_farms_count,
                "Number of farms migrated does not match"
            );

            info!(
                "👥  TFGrid pallet migration to {:?} passes POST migrate checks ✅",
                Pallet::<T>::pallet_version()
            );

            Ok(())
        }
    }
}

pub fn fix_farming_policy_migration_<T: Config>() -> frame_support::weights::Weight {
    info!(" >>> Migrating farm storage...");

    let mut read_writes = 0;

    Farms::<T>::translate::<super::FarmInfoOf<T>, _>(|k, f| {
        let mut new_farm = f;

        new_farm.pricing_policy_id = 1;
        info!("migrated farm: {:?}", k);

        read_writes += 1;
        Some(new_farm)
    });

    // Update pallet storage version
    PalletVersion::<T>::set(types::StorageVersion::V11Struct);
    info!(" <<< Storage version upgraded");

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(read_writes as Weight, read_writes as Weight)
}
