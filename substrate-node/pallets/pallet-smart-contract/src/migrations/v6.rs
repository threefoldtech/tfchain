use crate::*;
use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};
use log::debug;
use sp_std::marker::PhantomData;

#[cfg(feature = "try-runtime")]
use frame_support::traits::OnRuntimeUpgradeHelpersExt;
#[cfg(feature = "try-runtime")]
use sp_runtime::SaturatedConversion;

pub struct ContractMigrationV5<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for ContractMigrationV5<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        info!(
            " --- Current Smart Contract pallet version: {:?}",
            PalletVersion::<T>::get()
        );
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V5);

        // Store number of contracts in temp storage
        let contracts_count: u64 = ContractsToBillAt::<T>::iter_keys().count().saturated_into();
        Self::set_temp_storage(contracts_count, "pre_contracts_count");
        info!(
            "👥  Smart Contract pallet to V6 passes PRE migrate checks ✅: {:?} contracts",
            contracts_count
        );
        Ok(())
    }

    fn on_runtime_upgrade() -> Weight {
        migrate_to_version_6::<T>()
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        info!(
            " --- Current Smart Contract pallet version: {:?}",
            PalletVersion::<T>::get()
        );
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V6);

        // Check number of Contracts against pre-check result
        let pre_contracts_count = Self::get_temp_storage("pre_contracts_count").unwrap_or(0u64);
        assert_eq!(
            ContractsToBillAt::<T>::iter_keys()
                .count()
                .saturated_into::<u64>(),
            pre_contracts_count,
            "Number of Contracts migrated does not match"
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
        frame_support::migration::remove_storage_prefix(
            b"SmartContractModule",
            b"ContractsToBillAt",
            b"",
        );

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
        info!(
            " <<< Storage version Smart Contract pallet upgraded to {:?}",
            PalletVersion::<T>::get()
        );

        // Return the weight consumed by the migration.
        T::DbWeight::get().reads_writes(migrated_count as Weight + 1, migrated_count as Weight + 1)
    } else {
        info!(" >>> Unused Smart Contract pallet V6 migration");
        return 0;
    }
}
