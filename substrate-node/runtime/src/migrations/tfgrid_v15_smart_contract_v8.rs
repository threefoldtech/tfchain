use crate::pallet_smart_contract::{BalanceOf, Config, GRID_LOCK_ID};
use crate::pallet_tfgrid::{types::Twin, AccountIdOf};
use crate::sp_api_hidden_includes_construct_runtime::hidden_include::traits::{
    Currency, LockableCurrency,
};
use crate::*;
use frame_support::{
    traits::OnRuntimeUpgrade,
    traits::{Get, WithdrawReasons},
    weights::Weight,
};
use log::{debug, info};
use sp_std::{collections::btree_map::BTreeMap, marker::PhantomData};

#[cfg(feature = "try-runtime")]
use codec::Decode;
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

pub struct Migrate<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for Migrate<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        info!(
            "current pallet version: {:?}",
            pallet_tfgrid::PalletVersion::<T>::get()
        );
        assert!(
            pallet_tfgrid::PalletVersion::<T>::get()
                >= pallet_tfgrid::types::StorageVersion::V14Struct
        );

        let twins_count: u64 = pallet_tfgrid::Twins::<T>::iter().count() as u64;
        log::info!(
            "🔎 MigrateTwinsV15 pre migration: Number of existing twins {:?}",
            twins_count
        );

        info!("👥  TFGrid pallet to v14 passes PRE migrate checks ✅",);
        Ok(twins_count.encode())
    }

    fn on_runtime_upgrade() -> Weight {
        let mut twin_ids: BTreeMap<u32, AccountIdOf<T>> = BTreeMap::new();
        let mut total_reads = 0;
        let mut total_writes = 0;
        if pallet_tfgrid::PalletVersion::<T>::get()
            == pallet_tfgrid::types::StorageVersion::V14Struct
        {
            let (reads, writes) = migrate_tfgrid::<T>(&mut twin_ids);
            total_reads += reads;
            total_writes += writes;
        } else {
            info!(" >>> Unused TFGrid pallet V15 migration");
        }
        if pallet_smart_contract::PalletVersion::<T>::get()
            >= pallet_smart_contract::types::StorageVersion::V6
        {
            let (reads, writes) = migrate_smart_contract::<T>(&twin_ids);
            total_reads += reads;
            total_writes += writes;
        } else {
            info!(" >>> Unused Smart Contract pallet V8 migration");
        }
        T::DbWeight::get().reads_writes(total_reads, total_writes)
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(pre_twins_count: Vec<u8>) -> Result<(), &'static str> {
        info!(
            "current pallet version: {:?}",
            pallet_tfgrid::PalletVersion::<T>::get()
        );
        assert!(
            pallet_tfgrid::PalletVersion::<T>::get()
                >= pallet_tfgrid::types::StorageVersion::V15Struct
        );

        // Check number of twins against pre-check result
        let pre_twins_count: u64 = Decode::decode(&mut pre_twins_count.as_slice())
            .expect("the state parameter should be something that was generated by pre_upgrade");
        assert_eq!(
            pallet_tfgrid::Twins::<T>::iter().count() as u64,
            pre_twins_count,
            "Number of twins migrated does not match"
        );

        info!(
            "👥  TFGrid pallet migration to {:?} passes POST migrate checks ✅",
            pallet_tfgrid::Pallet::<T>::pallet_version()
        );

        Ok(())
    }
}

pub fn migrate_tfgrid<T: Config>(twins: &mut BTreeMap<u32, AccountIdOf<T>>) -> (u64, u64) {
    info!(" >>> Migrating twin storage...");

    let mut reads_writes = 0;
    pallet_tfgrid::Twins::<T>::translate::<pallet_tfgrid::migrations::types::v14::Twin<Vec<u8>, AccountIdOf<T>>, _>(
        |k, twin| {
            debug!("migrated twin: {:?}", k);

            let new_twin = Twin::<AccountIdOf<T>> {
                id: twin.id,
                account_id: twin.account_id,
                relay: None,
                entities: twin.entities,
                pk: None,
            };
            twins.insert(twin.id, new_twin.account_id.clone());
            reads_writes += 1;
            reads_writes += 1;
            Some(new_twin)
        },
    );

    // Update pallet storage version
    pallet_tfgrid::PalletVersion::<T>::set(pallet_tfgrid::types::StorageVersion::V15Struct);
    info!(" <<< Twin migration success, storage version upgraded");

    // Return the weight consumed by the migration.
    return (reads_writes, reads_writes + 1);
}

pub fn migrate_smart_contract<T: Config>(twins: &BTreeMap<u32, AccountIdOf<T>>) -> (u64, u64) {
    debug!(
        " >>> Starting contract pallet migration, pallet version: {:?}",
        pallet_smart_contract::PalletVersion::<T>::get()
    );

    let mut reads = 0;
    let mut writes = 0;
    let mut twin_contract_locked_balances: BTreeMap<u32, BalanceOf<T>> = BTreeMap::new();
    // Fetch all locked balances based on the contract locks in storage and accumulate them by twin id
    for (ctr_id, l) in pallet_smart_contract::ContractLock::<T>::iter() {
        let ctr = pallet_smart_contract::Contracts::<T>::get(ctr_id);
        reads += 1;
        match ctr {
            Some(contract) => {
                if !twins.contains_key(&contract.twin_id) {
                    debug!(
                        "twins: {} does not exist, removing contract and lock",
                        contract.twin_id
                    );
                    pallet_smart_contract::Contracts::<T>::remove(ctr_id);
                    pallet_smart_contract::ContractLock::<T>::remove(ctr_id);
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
                pallet_smart_contract::ContractLock::<T>::remove(ctr_id);
            }
        }
    }

    for (twin_id, account_id) in twins {
        // If the twin needs to have some locked balance on his account because of running contracts
        // Check how much we can actually lock based on his current total balance
        // this will make sure the locked balance will not exceed the total balance on the twin's account
        let should_lock = twin_contract_locked_balances.get(&twin_id).map(|b| {
            debug!(
                "contract locked balance on twin {} account: {:?}",
                twin_id, b
            );
            (<T as Config>::Currency::total_balance(&account_id)
                - <T as Config>::Currency::minimum_balance())
            .min(*b)
        });

        // Unlock all balance for the twin
        <T as Config>::Currency::remove_lock(GRID_LOCK_ID, &account_id);
        writes += 1;

        if let Some(should_lock) = should_lock {
            debug!("we should lock: {:?}", should_lock);
            // Only do a set lock if we actually have to lock
            <T as Config>::Currency::set_lock(
                GRID_LOCK_ID,
                &account_id,
                should_lock,
                WithdrawReasons::all(),
            );
            writes += 1;
        }
    }

    // Update pallet storage version
    pallet_smart_contract::PalletVersion::<T>::set(
        pallet_smart_contract::types::StorageVersion::V8,
    );
    debug!(" <<< Storage version upgraded");

    info!("👥  Smart Contract pallet to V8 succeeded");
    // Return the weight consumed by the migration.
    return (reads, writes + 1);
}
