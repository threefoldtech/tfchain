use crate::*;
use frame_support::{
    pallet_prelude::ValueQuery, storage_alias, traits::OnRuntimeUpgrade, weights::Weight,
    Blake2_128Concat,
};
use log::{debug, info};
use sp_core::Get;
use sp_runtime::traits::Zero;
use sp_std::marker::PhantomData;

#[cfg(feature = "try-runtime")]
use sp_std::{vec, vec::Vec};

// Storage alias from ContractLock v11
#[storage_alias]
pub type ContractLock<T: Config> = StorageMap<
    Pallet<T>,
    Blake2_128Concat,
    u64,
    super::types::v11::ContractLock<BalanceOf<T>>,
    ValueQuery,
>;
pub struct ExtendContractLock<T: Config>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for ExtendContractLock<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        debug!("current pallet version: {:?}", PalletVersion::<T>::get());
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V10);

        debug!("👥  Smart Contract pallet to V11 passes PRE migrate checks ✅",);
        Ok(vec![])
    }

    fn on_runtime_upgrade() -> Weight {
        if PalletVersion::<T>::get() >= types::StorageVersion::V10 {
            migrate_to_version_11::<T>()
        } else {
            info!(" >>> Unused Smart Contract pallet V11 migration");
            Weight::zero()
        }
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_: Vec<u8>) -> Result<(), &'static str> {
        debug!("current pallet version: {:?}", PalletVersion::<T>::get());
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V11);

        check_contract_lock_v11::<T>();

        debug!(
            "👥  Smart Contract pallet to {:?} passes POST migrate checks ✅",
            PalletVersion::<T>::get()
        );

        Ok(())
    }
}

pub fn migrate_to_version_11<T: Config>() -> frame_support::weights::Weight {
    debug!(
        " >>> Starting contract pallet migration, pallet version: {:?}",
        PalletVersion::<T>::get()
    );

    let mut r = 0;
    let mut w = 0;

    // migrate contract locks
    ContractLock::<T>::translate::<super::types::v10::ContractLock<BalanceOf<T>>, _>(|k, fp| {
        r += 1;
        w += 1;
        debug!("Migrating contract lock {:?}", k);
        Some(super::types::v11::ContractLock::<BalanceOf<T>> {
            amount_locked: fp.amount_locked,
            // Default to 0
            extra_amount_locked: BalanceOf::<T>::zero(),
            lock_updated: fp.lock_updated,
            cycles: fp.cycles,
        })
    });

    // Set the new storage version
    PalletVersion::<T>::put(types::StorageVersion::V11);
    w += 1;

    T::DbWeight::get().reads_writes(r, w)
}

pub fn check_contract_lock_v11<T: Config>() {
    debug!(
        "🔎  Smart Contract pallet {:?} checking ContractLock storage map START",
        PalletVersion::<T>::get()
    );

    // Check each contract has an associated contract lock
    for (contract_id, _) in Contracts::<T>::iter() {
        // ContractLock
        if !ContractLock::<T>::contains_key(contract_id) {
            debug!(
                " ⚠️    Contract (id: {}): no contract lock found",
                contract_id
            );
        }
    }

    // Check each contract lock has a valid contract
    for (contract_id, contract_lock) in ContractLock::<T>::iter() {
        if Contracts::<T>::get(contract_id).is_none() {
            debug!(
                " ⚠️    ContractLock[contract: {}]: contract not exists",
                contract_id
            );
        } else {
            // Ensure new field is set to zero
            assert_eq!(contract_lock.extra_amount_locked, BalanceOf::<T>::zero());
        }
    }

    debug!(
        "🏁  Smart Contract pallet {:?} checking ContractLock storage map END",
        PalletVersion::<T>::get()
    );
}
