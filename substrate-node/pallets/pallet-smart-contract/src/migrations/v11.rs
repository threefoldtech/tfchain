use crate::*;
use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};
use log::{debug, info};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use sp_std::marker::PhantomData;
use scale_info::TypeInfo;

#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;


#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo, MaxEncodedLen,
)]
pub struct ContractLockV10<BalanceOf> {
    pub amount_locked: BalanceOf,
    pub lock_updated: u64,
    pub cycles: u16,
}

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
    ContractLock::<T>::translate::<ContractLockV10<BalanceOf<T>>, _>(|k, fp| {
        r += 1;
        w += 1;
        debug!("Migrating contract lock {:?}", k);
        Some(types::ContractLock {
            amount_locked: fp.amount_locked,
            // Default to 0
            extra_amount_locked: BalanceOf::<T>::saturated_from(0 as u128),
            lock_updated: fp.lock_updated,
            cycles: fp.cycles,
        })
    });

    // Set the new storage version
    PalletVersion::<T>::put(types::StorageVersion::V11);
    w += 1;

    T::DbWeight::get().reads_writes(r, w)
}