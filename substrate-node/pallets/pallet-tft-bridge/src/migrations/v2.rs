use crate::*;
use frame_support::{
    log::{debug, info},
    traits::Get,
    traits::OnRuntimeUpgrade,
    weights::Weight,
};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_std::marker::PhantomData;

#[cfg(feature = "try-runtime")]
use frame_support::{dispatch::DispatchError, ensure};
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

pub struct MigrateBurnTransactionsV2<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateBurnTransactionsV2<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        ensure!(
            PalletVersion::<T>::get() == types::StorageVersion::V1,
            DispatchError::Other("Unexpected pallet version")
        );

        let burn_transactions_count: u64 =
            migrations::types::v1::BurnTransactions::<T>::iter().count() as u64;
        info!(
            "🔎 MigrateBurnTransactionsV2 pre migration: Number of existing burn transactions {:?}",
            burn_transactions_count
        );

        let executed_burn_transactions_count: u64 =
            migrations::types::v1::ExecutedBurnTransactions::<T>::iter().count() as u64;
        info!(
            "🔎 MigrateBurnTransactionsV2 pre migration: Number of existing executed burn transactions {:?}",
            executed_burn_transactions_count
        );

        info!("👥  TFT-BRIDGE pallet to V1 passes PRE migrate checks ✅",);
        return Ok(Vec::<u8>::new());
    }

    fn on_runtime_upgrade() -> Weight {
        if PalletVersion::<T>::get() == types::StorageVersion::V1 {
            migrate_burn_transactions::<T>()
        } else {
            info!(" >>> Unused TFT-BRIDGE pallet V2 migration");
            Weight::zero()
        }
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(
        _pre_burn_transactions_count: Vec<u8>,
    ) -> Result<(), sp_runtime::TryRuntimeError> {
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        ensure!(
            PalletVersion::<T>::get() == types::StorageVersion::V2,
            DispatchError::Other("Unexpected pallet version")
        );

        let burn_transactions_count: u64 =
            migrations::types::v2::BurnTransactions::<T>::iter().count() as u64;
        info!(
            "🔎 MigrateBurnTransactionsV2 post migration: Number of existing burn transactions {:?}",
            burn_transactions_count
        );

        let executed_burn_transactions_count: u64 =
            migrations::types::v2::ExecutedBurnTransactions::<T>::iter().count() as u64;
        info!(
            "🔎 MigrateBurnTransactionsV2 post migration: Number of existing executed burn transactions {:?}",
            executed_burn_transactions_count
        );

        Ok(())
    }
}

pub fn migrate_burn_transactions<T: Config>() -> frame_support::weights::Weight {
    info!(" >>> Migrating burn transactions storage...");

    let mut read_writes = 0;

    migrations::types::v2::BurnTransactions::<T>::translate::<
        super::types::v1::BurnTransaction<BlockNumberFor<T>>,
        _,
    >(|k, burn_transaction| {
        debug!("migrated burn transaction: {:?}", k);

        let new_burn_transaction =
            migrations::types::v2::BurnTransaction::<T::AccountId, BlockNumberFor<T>> {
                block: burn_transaction.block,
                amount: burn_transaction.amount,
                source: None,
                target: burn_transaction.target,
                signatures: burn_transaction.signatures,
                sequence_number: burn_transaction.sequence_number,
            };

        read_writes += 1;
        Some(new_burn_transaction)
    });

    migrations::types::v2::ExecutedBurnTransactions::<T>::translate::<
        super::types::v1::BurnTransaction<BlockNumberFor<T>>,
        _,
    >(|k, executed_burn_transaction| {
        debug!("migrated executed burn transaction: {:?}", k);

        let new_executed_burn_transaction =
            migrations::types::v2::BurnTransaction::<T::AccountId, BlockNumberFor<T>> {
                block: executed_burn_transaction.block,
                amount: executed_burn_transaction.amount,
                source: None,
                target: executed_burn_transaction.target,
                signatures: executed_burn_transaction.signatures,
                sequence_number: executed_burn_transaction.sequence_number,
            };

        read_writes += 1;
        Some(new_executed_burn_transaction)
    });

    // Update pallet storage version
    PalletVersion::<T>::set(types::StorageVersion::V2);
    info!(" <<< burnTransactions migration success, storage version upgraded");

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(read_writes, read_writes + 1)
}
