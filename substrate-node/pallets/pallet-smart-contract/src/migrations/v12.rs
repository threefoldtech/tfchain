use crate::*;
use frame_support::{
    pallet_prelude::ValueQuery,
    storage_alias,
    traits::{LockableCurrency, OnRuntimeUpgrade},
    weights::Weight,
    Blake2_128Concat,
};
use log::{debug, info};
use sp_core::Get;
use sp_runtime::traits::Zero;
use sp_std::marker::PhantomData;

#[cfg(feature = "try-runtime")]
use frame_support::{dispatch::DispatchError, ensure};
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

// Storage alias from ContractPaymentState v12
#[storage_alias]
pub type ContractPaymentState<T: Config> = StorageMap<
    Pallet<T>,
    Blake2_128Concat,
    u64,
    super::types::v12::ContractPaymentState<BalanceOf<T>>,
    ValueQuery,
>;

pub struct MigrateContractLockToContractPaymentState<T: Config>(pub PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateContractLockToContractPaymentState<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
        validate_pallet_version::<T>(types::StorageVersion::V11)?;

        let count = ContractLock::<T>::iter().count();
        debug!(
            "🏁  Smart Contract pallet {:?} ContractLock length before migration {:?}",
            PalletVersion::<T>::get(),
            count
        );

        Ok(count.to_le_bytes().to_vec())
    }

    fn on_runtime_upgrade() -> Weight {
        if PalletVersion::<T>::get() == types::StorageVersion::V11 {
            migrate_to_version_12::<T>()
        } else {
            info!(" >>> Unused Smart Contract pallet V12 migration");
            Weight::zero()
        }
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(count: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
        validate_pallet_version::<T>(types::StorageVersion::V12)?;

        let new_count = ContractPaymentState::<T>::iter().count();
        let old_count =
            usize::from_le_bytes(count.try_into().expect("slice with incorrect length"));

        if old_count != 0 {
            debug!(
                "🏁  Smart Contract pallet {:?} ContractPaymentState length after migration {:?}",
                PalletVersion::<T>::get(),
                new_count
            );
            ensure!(
                new_count == old_count,
                DispatchError::Other(
                    "Number of ContractPaymentState migrated does not match: {:?}"
                )
            );
        }

        check_contract_lock_v12::<T>()
    }
}

#[cfg(feature = "try-runtime")]
fn validate_pallet_version<T: Config>(
    expected_version: types::StorageVersion,
) -> Result<(), sp_runtime::TryRuntimeError> {
    let current_version = PalletVersion::<T>::get();
    debug!("current pallet version: {:?}", current_version);
    ensure!(
        current_version >= expected_version,
        DispatchError::Other("Unexpected pallet version")
    );
    Ok(())
}

pub fn migrate_to_version_12<T: Config>() -> frame_support::weights::Weight {
    debug!(
        " >>> Starting contract pallet migration, pallet version: {:?}",
        PalletVersion::<T>::get()
    );

    let mut total_reads = 0;
    let mut total_writes = 0;

    for (contract_id, contract) in Contracts::<T>::iter() {
        log::debug!("Contract id: {:?}", contract_id);

        if let Some(src_twin) = pallet_tfgrid::Twins::<T>::get(contract.twin_id) {
            if ContractLock::<T>::contains_key(contract_id) {
                let (r, w) = migrate_contract_lock::<T>(contract_id);
                total_reads += r;
                total_writes += w;

                let (r, w) = remove_all_locks::<T>(&src_twin.account_id);
                total_reads += r;
                total_writes += w;
            } else {
                log::debug!("ContractLock not found for contract {:?}", contract_id);
            }
        } else {
            log::debug!("Twin not found for contract {:?}", contract_id);
        }
    }

    // Set the new storage version
    PalletVersion::<T>::put(types::StorageVersion::V12);
    total_writes += 1;

    T::DbWeight::get().reads_writes(total_reads, total_writes)
}

fn migrate_contract_lock<T: Config>(contract_id: u64) -> (u64, u64) {
    let mut reads = 0;
    let mut writes = 0;

    let old_contract_lock = ContractLock::<T>::take(contract_id);
    reads += 1;
    writes += 1;

    ContractPaymentState::<T>::insert(
        contract_id,
        super::types::v12::ContractPaymentState {
            standard_reserve: BalanceOf::<T>::zero(),
            additional_reserve: BalanceOf::<T>::zero(),
            standard_overdraft: old_contract_lock.amount_locked,
            additional_overdraft: old_contract_lock.extra_amount_locked,
            last_updated_seconds: old_contract_lock.lock_updated,
            cycles: old_contract_lock.cycles,
        },
    );
    writes += 1;

    (reads, writes)
}

fn remove_all_locks<T: Config>(account_id: &T::AccountId) -> (u64, u64) {
    let mut reads = 0;
    let mut writes = 0;

    let locks = pallet_balances::Pallet::<T>::locks(account_id);
    reads += 1;

    for lock in locks {
        log::debug!("Removing lock: {:?} for account: {:?}", lock.id, account_id);
        pallet_balances::Pallet::<T>::remove_lock(lock.id, account_id);
        reads += 1;
        writes += 1;
    }

    (reads, writes)
}

#[cfg(feature = "try-runtime")]
pub fn check_contract_lock_v12<T: Config>() -> Result<(), sp_runtime::TryRuntimeError> {
    debug!(
        "🔎  Smart Contract pallet {:?} checking ContractLock storage map START",
        PalletVersion::<T>::get()
    );

    for (contract_id, _) in Contracts::<T>::iter() {
        if !ContractPaymentState::<T>::contains_key(contract_id) {
            debug!(
                " ⚠️    Contract (id: {}): no contract lock found",
                contract_id
            );
        }
    }

    debug!(
        "🏁  Smart Contract pallet {:?} checking ContractLock storage map END",
        PalletVersion::<T>::get()
    );

    debug!(
        "👥  Smart Contract pallet to {:?} passes POST migrate checks ✅",
        PalletVersion::<T>::get()
    );

    Ok(())
}
