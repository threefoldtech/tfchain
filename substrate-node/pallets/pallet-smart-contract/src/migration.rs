use super::*;
use frame_support::weights::Weight;

pub mod v5 {
    use super::*;
    use crate::Config;

    use frame_support::{pallet_prelude::Weight, traits::OnRuntimeUpgrade};
    use sp_std::marker::PhantomData;
    pub struct ContractMigrationV5<T: Config>(PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for ContractMigrationV5<T> {
        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<(), &'static str> {
            info!("current pallet version: {:?}", PalletVersion::<T>::get());
            assert!(PalletVersion::<T>::get() == types::StorageVersion::V5);

            info!("👥  Smart Contract pallet to v6 passes PRE migrate checks ✅",);
            Ok(())
        }

        fn on_runtime_upgrade() -> Weight {
            migrate_to_version_6::<T>()
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade() -> Result<(), &'static str> {
            info!("current pallet version: {:?}", PalletVersion::<T>::get());
            assert!(PalletVersion::<T>::get() == types::StorageVersion::V6);

            info!(
                "👥  Smart Contract pallet to {:?} passes POST migrate checks ✅",
                PalletVersion::<T>::get()
            );

            Ok(())
        }
    }
}

pub fn migrate_to_version_6<T: Config>() -> frame_support::weights::Weight {
    if PalletVersion::<T>::get() == types::StorageVersion::V6 {
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

        let billing_frequency = BillingFrequency::<T>::get();
        for (block_number, contract_ids) in contracts_to_bill_at {
            migrated_count += 1;
            // Construct new index
            let index = block_number - 1 % billing_frequency;
            // Reinsert items under the new key
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
        T::DbWeight::get().reads_writes(migrated_count as Weight + 1, migrated_count as Weight + 1)
    } else {
        info!(" >>> Unused migration");
        return 0;
    }
}
