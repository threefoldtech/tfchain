use crate::*;
use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};
use log::{debug, info};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::marker::PhantomData;

#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

pub struct FixTwinLockedBalances<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for FixTwinLockedBalances<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        debug!("current pallet version: {:?}", PalletVersion::<T>::get());
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V6);

        debug!("👥  Smart Contract pallet to V7 passes PRE migrate checks ✅",);
        Ok(vec![])
    }

    fn on_runtime_upgrade() -> Weight {
        if PalletVersion::<T>::get() != types::StorageVersion::V6 {
            migrate_to_version_8::<T>()
        } else {
            info!(" >>> Unused Smart Contract pallet V7 migration");
            Weight::zero()
        }
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_: Vec<u8>) -> Result<(), &'static str> {
        debug!("current pallet version: {:?}", PalletVersion::<T>::get());
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V7);

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

    let mut read_writes = 0;

    let mut twin_contract_locked_balances: BTreeMap<u32, BalanceOf<T>> = BTreeMap::new();
    // Fetch all locked balances based on the contract locks in storage and accumulate them by twin id
    for (ctr_id, l) in ContractLock::<T>::iter() {
        let ctr = Contracts::<T>::get(ctr_id);
        read_writes += 1;
        match ctr {
            Some(contract) => {
                twin_contract_locked_balances
                    .entry(contract.twin_id)
                    .and_modify(|v| *v += l.amount_locked)
                    .or_insert(BalanceOf::<T>::saturated_from(0 as u128));
            }
            None => (),
        }
    }

    for (twin_id, t) in pallet_tfgrid::Twins::<T>::iter() {
        read_writes += 1;

        // If the twin needs to have some locked balance on his account because of running contracts
        // Check how much we can actually locked based on his current total balance
        // this will make sure the locked balance will not exceed the total balance on the twin's account
        let contract_locked_b = twin_contract_locked_balances.get(&twin_id);
        let should_lock = match contract_locked_b {
            Some(b) => {
                // get the total balance of the twin - minimum existence requirement
                Some(
                    <T as Config>::Currency::total_balance(&t.account_id)
                        - <T as Config>::Currency::minimum_balance().min(*b),
                )
            }
            None => None,
        };

        debug!(
            "contract locked balance on twin {} account: {:?}",
            t.id, contract_locked_b
        );
        // Unlock all balance for the twin
        <T as Config>::Currency::remove_lock(GRID_LOCK_ID, &t.account_id);
        read_writes += 1;

        if let Some(should_lock) = should_lock {
            debug!("we should lock: {:?}", should_lock);
            // Only do a set lock if we actually have to lock
            <T as Config>::Currency::set_lock(
                GRID_LOCK_ID,
                &t.account_id,
                should_lock,
                WithdrawReasons::all(),
            );
            read_writes += 1;
        }
    }

    // Update pallet storage version
    PalletVersion::<T>::set(types::StorageVersion::V8);
    debug!(" <<< Storage version upgraded");

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads(read_writes + 1)
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
    match locked_balance {
        Some(balance) => balance,
        None => BalanceOf::<T>::saturated_from(0 as u128),
    }
}
