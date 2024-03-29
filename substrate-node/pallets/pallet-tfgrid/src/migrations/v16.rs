use crate::*;
use frame_support::{
    traits::{OnRuntimeUpgrade, PalletInfoAccess},
    weights::Weight,
};
use log::info;
use sp_core::Get;
use sp_std::marker::PhantomData;

pub struct KillNodeGpuStatus<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for KillNodeGpuStatus<T> {
    fn on_runtime_upgrade() -> Weight {
        if PalletVersion::<T>::get() == types::StorageVersion::V15Struct {
            info!(" >>> Kill NodeGpuStatus storage...");

            let res = frame_support::migration::clear_storage_prefix(
                <Pallet<T>>::name().as_bytes(),
                b"NodeGpuStatus",
                b"",
                None,
                None,
            );

            log::info!(
                "Cleared '{}' entries from 'NodeGpuStatus' storage prefix",
                res.unique
            );

            if res.maybe_cursor.is_some() {
                log::error!("Storage prefix 'NodeGpuStatus' is not completely cleared.");
            }

            // Update pallet storage version
            PalletVersion::<T>::set(types::StorageVersion::V16Struct);
            info!(" <<< NodeGpuStatus killing success, storage version upgraded");

            // Return the weight consumed by the migration.
            T::DbWeight::get().reads_writes(0, res.unique as u64 + 1)
        } else {
            info!(" >>> Unused TFGrid pallet V16 migration");
            Weight::zero()
        }
    }
}
