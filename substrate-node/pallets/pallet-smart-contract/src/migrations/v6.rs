use crate::*;
use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};
use log::{debug, info};
use sp_core::Get;
use sp_std::{marker::PhantomData, vec::Vec};

#[cfg(feature = "try-runtime")]
use frame_support::ensure;
#[cfg(feature = "try-runtime")]
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "try-runtime")]
use sp_runtime::DispatchError;

pub struct ContractMigrationV5<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for ContractMigrationV5<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        ensure!(
            PalletVersion::<T>::get() >= types::StorageVersion::V5,
            DispatchError::Other("Unexpected pallet version")
        );

        let contracts_count: u64 = ContractsToBillAt::<T>::iter().count() as u64;
        log::info!(
            "🔎 ContractMigrationV5 pre migration: Number of existing contracts {:?}",
            contracts_count
        );

        info!("👥  Smart Contract pallet to V6 passes PRE migrate checks ✅",);
        Ok(contracts_count.encode())
    }

    fn on_runtime_upgrade() -> Weight {
        migrate_to_version_6::<T>()
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(pre_contracts_count: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        ensure!(
            PalletVersion::<T>::get() >= types::StorageVersion::V6,
            DispatchError::Other("Unexpected pallet version")
        );

        // Check number of Contracts against pre-check result
        let pre_contracts_count: u64 = Decode::decode(&mut pre_contracts_count.as_slice())
            .expect("the state parameter should be something that was generated by pre_upgrade");
        ensure!(
            ContractsToBillAt::<T>::iter().count() as u64 == pre_contracts_count,
            DispatchError::Other("Number of Contracts migrated does not match")
        );

        info!(
            "👥  Smart Contract pallet to {:?} passes POST migrate checks ✅",
            PalletVersion::<T>::get()
        );

        Ok(())
    }
}

pub fn migrate_to_version_6<T: Config>() -> frame_support::weights::Weight {
    if PalletVersion::<T>::get() == types::StorageVersion::V5 {
        info!(
            " >>> Starting contract pallet migration, pallet version: {:?}",
            PalletVersion::<T>::get()
        );

        let mut migrated_count = 0;

        // Collect ContractsToBillAt storage in memory
        let contracts_to_bill_at = ContractsToBillAt::<T>::iter().collect::<Vec<_>>();

        // Remove all items under ContractsToBillAt
        let _ = frame_support::migration::clear_storage_prefix(
            b"SmartContractModule",
            b"ContractsToBillAt",
            b"",
            None,
            None,
        ); // TODO check parameters

        let billing_freq = 600;
        BillingFrequency::<T>::put(billing_freq);

        for (block_number, contract_ids) in contracts_to_bill_at {
            migrated_count += 1;
            // Construct new index
            let index = (block_number - 1) % billing_freq;
            // Reinsert items under the new key
            debug!(
                "inserted contracts:{:?} at index: {:?}",
                contract_ids.clone(),
                index
            );
            ContractsToBillAt::<T>::insert(index, contract_ids);
        }

        info!(
            " <<< Contracts storage updated! Migrated {} Contracts ✅",
            migrated_count
        );

        // Update pallet storage version
        PalletVersion::<T>::set(types::StorageVersion::V6);
        info!(" <<< Storage version upgraded");

        // Return the weight consumed by the migration.
        T::DbWeight::get().reads_writes(migrated_count + 1, migrated_count + 1)
    } else {
        info!(" >>> Unused Smart Contract pallet V6 migration");
        Weight::zero()
    }
}
