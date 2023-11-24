use crate::*;
use frame_support::{traits::Get, traits::OnRuntimeUpgrade, weights::Weight};
use log::{debug, info};
use sp_std::marker::PhantomData;

#[cfg(feature = "try-runtime")]
use parity_scale_codec::Decode;
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

pub struct MigrateTwinsV15<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateTwinsV15<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V14Struct);

        let twins_count: u64 = Twins::<T>::iter().count() as u64;
        log::info!(
            "🔎 MigrateTwinsV15 pre migration: Number of existing twins {:?}",
            twins_count
        );

        info!("👥  TFGrid pallet to v14 passes PRE migrate checks ✅",);
        Ok(twins_count.encode())
    }

    fn on_runtime_upgrade() -> Weight {
        if PalletVersion::<T>::get() == types::StorageVersion::V14Struct {
            migrate_twins::<T>()
        } else {
            info!(" >>> Unused TFGrid pallet V15 migration");
            Weight::zero()
        }
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(pre_twins_count: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V15Struct);

        // Check number of twins against pre-check result
        let pre_twins_count: u64 = Decode::decode(&mut pre_twins_count.as_slice())
            .expect("the state parameter should be something that was generated by pre_upgrade");
        assert_eq!(
            Twins::<T>::iter().count() as u64,
            pre_twins_count,
            "Number of twins migrated does not match"
        );

        info!(
            "👥  TFGrid pallet migration to {:?} passes POST migrate checks ✅",
            Pallet::<T>::pallet_version()
        );

        Ok(())
    }
}

pub fn migrate_twins<T: Config>() -> frame_support::weights::Weight {
    info!(" >>> Migrating twin storage...");

    let mut read_writes = 0;

    Twins::<T>::translate::<super::types::v14::Twin<Vec<u8>, AccountIdOf<T>>, _>(|k, twin| {
        debug!("migrated twin: {:?}", k);

        let new_twin = types::Twin {
            id: twin.id,
            account_id: twin.account_id,
            relay: None,
            entities: twin.entities,
            pk: None,
        };

        read_writes += 1;
        Some(new_twin)
    });

    // Update pallet storage version
    PalletVersion::<T>::set(types::StorageVersion::V15Struct);
    info!(" <<< Twin migration success, storage version upgraded");

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(read_writes, read_writes + 1)
}
