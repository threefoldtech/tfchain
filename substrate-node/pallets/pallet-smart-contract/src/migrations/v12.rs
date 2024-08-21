use crate::*;
use frame_support::{
    pallet_prelude::ValueQuery, storage_alias, traits::{LockableCurrency, OnRuntimeUpgrade}, weights::Weight,
    Blake2_128Concat,
};
use log::{debug, info};
use sp_core::Get;
use sp_runtime::traits::Zero;
use sp_std::marker::PhantomData;

#[cfg(feature = "try-runtime")]
use frame_support::{dispatch::DispatchError, ensure};
#[cfg(feature = "try-runtime")]
use sp_std::{vec::Vec};

// Storage alias from ContractPaymentState v12
#[storage_alias]
pub type ContractPaymentState<T: Config> = StorageMap<
    Pallet<T>,
    Blake2_128Concat,
    u64,
    super::types::v12::ContractPaymentState<BalanceOf<T>>,
    ValueQuery,
>;
pub struct MigrateContractLockToContractPaymentState<T: Config>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for MigrateContractLockToContractPaymentState<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
        debug!("current pallet version: {:?}", PalletVersion::<T>::get());
        ensure!(
            PalletVersion::<T>::get() >= types::StorageVersion::V11,
            DispatchError::Other("Unexpected pallet version")
        );
        debug!("👥  Smart Contract pallet to V12 passes PRE migrate checks ✅",);
        let count = ContractLock::<T>::iter().count();
        debug!(
            "🏁  Smart Contract pallet {:?} ContractLock length before migration {:?}",
            PalletVersion::<T>::get(),
            count
        );
        // convert usize to vec of u8
        Ok(ContractLock::<T>::iter().count().to_le_bytes().to_vec())
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
        debug!("current pallet version: {:?}", PalletVersion::<T>::get());
        ensure!(
            PalletVersion::<T>::get() >= types::StorageVersion::V12,
            DispatchError::Other("Unexpected pallet version")
        );
        let new_count =  ContractPaymentState::<T>::iter().count();
        debug!("🏁  Smart Contract pallet {:?} ContractPaymentState length after migration {:?}", PalletVersion::<T>::get(), new_count);
        ensure!(
            new_count == usize::from_le_bytes(count.try_into().expect("slice with incorrect length")),
            DispatchError::Other("Number of ContractPaymentState migrated does not match: {:?}")
        );
        // print len of ContractPaymentState
        debug!(
            "🏁  Smart Contract pallet {:?} ContractPaymentState length after migration {:?}",
            PalletVersion::<T>::get(),
            new_count
        );
        check_contract_lock_v12::<T>()
    }
}

pub fn migrate_to_version_12<T: Config>() -> frame_support::weights::Weight {
    debug!(
        " >>> Starting contract pallet migration, pallet version: {:?}",
        PalletVersion::<T>::get()
    );

    let mut r = 0;
    let mut w = 0;

    for (contract_id, contract) in Contracts::<T>::iter() {
        log::debug!("Contract id: {:?}", contract_id);
        let src_twin = pallet_tfgrid::Twins::<T>::get(contract.twin_id);
        // continue if src_twin doesn't exist
        if src_twin.is_none() {
            log::debug!("Twin not found for contract {:?}", contract_id);
            continue;
        }
        let old_contract_lock = ContractLock::<T>::take(contract_id);

        ContractPaymentState::<T>::insert(contract_id, super::types::v12::ContractPaymentState {
            standard_reserve: BalanceOf::<T>::zero(),
            additional_reserve: BalanceOf::<T>::zero(),
            standard_overdraft: old_contract_lock.amount_locked,
            additional_overdraft:old_contract_lock.extra_amount_locked,
            last_updated_seconds: old_contract_lock.lock_updated,
            cycles: old_contract_lock.cycles,
        });

        let src_twin = src_twin.unwrap();
        let account_id = &src_twin.account_id;
        let locks = pallet_balances::Pallet::<T>::locks(account_id);
        r += 3;
        w += 2;

        // Remove all locks on the user account
        for lock in locks {
            log::debug!("Removing lock: {:?} for account: {:?}", lock.id, account_id,);
            pallet_balances::Pallet::<T>::remove_lock(lock.id, account_id);
            r += 1;
            w += 1;
        }
    }
    // Set the new storage version
    PalletVersion::<T>::put(types::StorageVersion::V12);
    w += 1;

    let weight = T::DbWeight::get().reads_writes(r, w);

    // log weights
    debug!(" >>> consumed weight: {:?} ", weight);
    weight

}

#[cfg(feature = "try-runtime")]
pub fn check_contract_lock_v12<T: Config>() -> Result<(), sp_runtime::TryRuntimeError> {
    debug!(
        "🔎  Smart Contract pallet {:?} checking ContractLock storage map START",
        PalletVersion::<T>::get()
    );

    // Check each contract has an associated contract lock
    for (contract_id, _) in Contracts::<T>::iter() {
        // ContractLock
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
