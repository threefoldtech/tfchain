use crate::{types::FarmingPolicy, *};
use frame_support::{traits::Get, traits::OnRuntimeUpgrade, weights::Weight};
use log::{debug, info};
use sp_std::marker::PhantomData;

#[cfg(feature = "try-runtime")]
use codec::Decode;
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

pub struct FixFarmingPoliciesMap<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for FixFarmingPoliciesMap<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V13Struct);

        let policies_count: u64 = FarmingPoliciesMap::<T>::iter().count() as u64;
        info!(
            "🔎 FixFarmingPoliciesMap pre migration: Number of existing farming policies {:?}",
            policies_count
        );

        info!("👥  TFGrid pallet to V14 passes PRE migrate checks ✅",);
        Ok(policies_count.encode())
    }

    fn on_runtime_upgrade() -> Weight {
        if PalletVersion::<T>::get() == types::StorageVersion::V13Struct {
            fix_farming_policies_map_migration_::<T>()
        } else {
            info!(" >>> Unused TFGrid pallet V14 migration");
            Weight::zero()
        }
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(pre_policies_count: Vec<u8>) -> Result<(), &'static str> {
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V14Struct);

        // Check number of farming policies against pre-check result
        let pre_policies_count: u64 = Decode::decode(&mut pre_policies_count.as_slice())
            .expect("the state parameter should be something that was generated by pre_upgrade");
        assert_eq!(
            FarmingPoliciesMap::<T>::iter().count() as u64,
            pre_policies_count,
            "Number of farming policies migrated does not match"
        );

        info!(
            "👥  TFGrid pallet migration to {:?} passes POST migrate checks ✅",
            Pallet::<T>::pallet_version()
        );

        Ok(())
    }
}

pub fn fix_farming_policies_map_migration_<T: Config>() -> frame_support::weights::Weight {
    info!(" >>> Migrating farming policies storage...");

    let mut read_writes = 0;
    FarmingPoliciesMap::<T>::translate::<FarmingPolicy<T::BlockNumber>, _>(|k, fp| {
        debug!("Farming policy #{:?}: start migrating", k);
        debug!("  id was: {:?}", fp.id);
        let mut new_farming_policy = fp.clone();
        new_farming_policy.id = k;
        debug!("  id is now: {:?}", new_farming_policy.id);
        debug!("Farming policy #{:?} succesfully migrated", k);

        read_writes += 1;
        Some(new_farming_policy)
    });

    info!(
        " <<< Farming policies storage updated! Migrated {} Farming policies ✅",
        read_writes
    );

    // Update pallet storage version
    PalletVersion::<T>::set(types::StorageVersion::V14Struct);
    info!(" <<< Storage version upgraded");

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(read_writes, read_writes + 1)
}