use crate::*;
#[cfg(feature = "try-runtime")]
use codec::{Decode, Encode};
use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};
use log::debug;
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
        if PalletVersion::<T>::get() == types::StorageVersion::V6 {
            migrate_to_version_7::<T>()
        } else {
            debug!(" >>> Unused Smart Contract pallet V7 migration");
            Weight::zero()
        }
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(pre_contracts_count: Vec<u8>) -> Result<(), &'static str> {
        debug!("current pallet version: {:?}", PalletVersion::<T>::get());
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V7);

        debug!(
            "👥  Smart Contract pallet to {:?} passes POST migrate checks ✅",
            PalletVersion::<T>::get()
        );

        Ok(())
    }
}

pub fn migrate_to_version_7<T: Config>() -> frame_support::weights::Weight {
    debug!(
        " >>> Starting contract pallet migration, pallet version: {:?}",
        PalletVersion::<T>::get()
    );

    let mut read_writes = 0;

    let mut twin_contract_locked_balances: BTreeMap<u32, BalanceOf<T>> = BTreeMap::new();

    for (ctr_id, l) in ContractLock::<T>::iter() {
        let ctr = Contracts::<T>::get(ctr_id);
        read_writes += 1;
        match ctr {
            Some(contract) => {
                // l.amount_locked
                twin_contract_locked_balances
                    .entry(contract.twin_id)
                    .and_modify(|v| *v += l.amount_locked)
                    .or_insert(BalanceOf::<T>::saturated_from(0 as u128));
            }
            None => (),
        }
    }

    for (t, total_contract_locked_balance) in twin_contract_locked_balances {
        let twin = pallet_tfgrid::Twins::<T>::get(t);
        read_writes += 1;
        match twin {
            Some(twin) => {
                let total_lock_balances = get_locked_balance::<T>(&twin.account_id);

                if total_lock_balances != total_contract_locked_balance {
                    debug!(
                        "total locked balance on twin {} account: {:?}",
                        t, total_lock_balances
                    );
                    debug!(
                        "should have locked only: {:?}",
                        total_contract_locked_balance
                    );

                    // get the total balance of the twin - minimum existence requirement
                    let total_balance = <T as Config>::Currency::total_balance(&twin.account_id)
                        - <T as Config>::Currency::minimum_balance();

                    // lock only an amount up to the total balance
                    // this will make sure the locked balance will not exceed the total balance on the twin's account
                    let amount_that_we_can_lock = total_balance.min(total_contract_locked_balance);
                    debug!("we can lock up to: {:?}", amount_that_we_can_lock);

                    // Unlock all balance & relock real locked amount
                    <T as Config>::Currency::remove_lock(GRID_LOCK_ID, &twin.account_id);
                    <T as Config>::Currency::set_lock(
                        GRID_LOCK_ID,
                        &twin.account_id,
                        amount_that_we_can_lock,
                        WithdrawReasons::all(),
                    );
                    read_writes += 2;
                }
            }
            None => {
                debug!("twin {} not found", t);
            }
        }
    }

    // Update pallet storage version
    PalletVersion::<T>::set(types::StorageVersion::V7);
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
