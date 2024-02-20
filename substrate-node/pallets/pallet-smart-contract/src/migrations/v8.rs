use crate::*;
use frame_support::{
    traits::{Currency, LockableCurrency, OnRuntimeUpgrade, WithdrawReasons},
    weights::Weight,
};
use log::{debug, info};
use sp_core::Get;
use sp_runtime::traits::{CheckedSub, SaturatedConversion};
use sp_std::{collections::btree_map::BTreeMap, marker::PhantomData};

#[cfg(feature = "try-runtime")]
use frame_support::{dispatch::DispatchError, ensure};
#[cfg(feature = "try-runtime")]
use sp_std::{vec, vec::Vec};

pub struct FixTwinLockedBalances<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for FixTwinLockedBalances<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
        debug!("current pallet version: {:?}", PalletVersion::<T>::get());
        ensure!(
            PalletVersion::<T>::get() >= types::StorageVersion::V6,
            DispatchError::Other("Unexpected pallet version")
        );

        debug!("👥  Smart Contract pallet to V8 passes PRE migrate checks ✅",);
        Ok(vec![])
    }

    fn on_runtime_upgrade() -> Weight {
        if PalletVersion::<T>::get() >= types::StorageVersion::V6 {
            migrate_to_version_8::<T>()
        } else {
            info!(" >>> Unused Smart Contract pallet V8 migration");
            Weight::zero()
        }
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
        debug!("current pallet version: {:?}", PalletVersion::<T>::get());
        ensure!(
            PalletVersion::<T>::get() >= types::StorageVersion::V8,
            DispatchError::Other("Unexpected pallet version")
        );

        debug!(
            "👥  Smart Contract pallet to {:?} passes POST migrate checks ✅",
            PalletVersion::<T>::get()
        );

        Ok(())
    }
}

pub fn migrate_to_version_8<T: Config>() -> frame_support::weights::Weight {
    debug!(
        " >>> Starting contract pallet migration, pallet version: {:?}",
        PalletVersion::<T>::get()
    );

    let mut reads = 0;
    let mut writes = 0;

    let mut twin_contract_locked_balances: BTreeMap<u32, BalanceOf<T>> = BTreeMap::new();
    // Fetch all locked balances based on the contract locks in storage and accumulate them by twin id
    for (ctr_id, l) in ContractLock::<T>::iter() {
        let ctr = Contracts::<T>::get(ctr_id);
        reads += 1;
        match ctr {
            Some(contract) => {
                reads += 1;
                if !pallet_tfgrid::Twins::<T>::contains_key(contract.twin_id) {
                    debug!(
                        "twins: {} does not exist, removing contract and lock",
                        contract.twin_id
                    );
                    Contracts::<T>::remove(ctr_id);
                    ContractLock::<T>::remove(ctr_id);
                    writes += 2;
                } else {
                    *twin_contract_locked_balances
                        .entry(contract.twin_id)
                        .or_default() += l.amount_locked;
                }
            }
            None => {
                debug!(
                    "no contract found for contract lock {}, cleaning up lock..",
                    ctr_id
                );
                writes += 1;
                ContractLock::<T>::remove(ctr_id);
            }
        }
    }

    for (twin_id, t) in pallet_tfgrid::Twins::<T>::iter() {
        reads += 1;

        // If the twin needs to have some locked balance on his account because of running contracts
        // Check how much we can actually lock based on his current total balance
        // this will make sure the locked balance will not exceed the total balance on the twin's account
        let should_lock = twin_contract_locked_balances.get(&twin_id).map(|b| {
            debug!(
                "contract locked balance on twin {} account: {:?}",
                twin_id, b
            );
            (<T as Config>::Currency::total_balance(&t.account_id)
                - <T as Config>::Currency::minimum_balance())
            .min(*b)
        });

        // Unlock all balance for the twin
        <T as Config>::Currency::remove_lock(GRID_LOCK_ID, &t.account_id);
        writes += 1;

        if let Some(should_lock) = should_lock {
            debug!("we should lock: {:?}", should_lock);
            // Only do a set lock if we actually have to lock
            <T as Config>::Currency::set_lock(
                GRID_LOCK_ID,
                &t.account_id,
                should_lock,
                WithdrawReasons::all(),
            );
            writes += 1;
        }
    }

    // Update pallet storage version
    PalletVersion::<T>::set(types::StorageVersion::V8);
    debug!(" <<< Storage version upgraded");

    info!("👥  Smart Contract pallet to V8 succeeded");
    // Return the weight consumed by the migration.
    T::DbWeight::get().reads(reads) + T::DbWeight::get().writes(writes + 1)
}

fn get_usable_balance<T: Config>(account_id: &T::AccountId) -> BalanceOf<T> {
    let balance = pallet_balances::pallet::Pallet::<T>::usable_balance(account_id);
    let b = balance.saturated_into::<u128>();
    BalanceOf::<T>::saturated_from(b)
}

pub fn get_locked_balance<T: Config>(account_id: &T::AccountId) -> BalanceOf<T> {
    let usable_balance = get_usable_balance::<T>(account_id);

    let free_balance = <T as Config>::Currency::free_balance(account_id);

    let locked_balance = free_balance.checked_sub(&usable_balance);
    locked_balance.unwrap_or_default()
}
