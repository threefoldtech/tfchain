use crate::*;
use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};
use log::info;
use sp_core::Get;
use sp_runtime::Saturating;
use sp_std::{marker::PhantomData, vec};

#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

pub struct ReworkBillingLoopInsertion<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for ReworkBillingLoopInsertion<T> {
    fn on_runtime_upgrade() -> Weight {
        rework_billing_loop_insertion::<T>()
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_: Vec<u8>) -> Result<(), &'static str> {
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V10);

        super::v9::check_contracts_to_bill_at::<T>();

        info!(
            "👥  Smart Contract pallet to {:?} passes POST migrate checks ✅",
            PalletVersion::<T>::get()
        );

        Ok(())
    }
}

pub fn rework_billing_loop_insertion<T: Config>() -> frame_support::weights::Weight {
    if PalletVersion::<T>::get() == types::StorageVersion::V9 {
        info!(
            ">>> Starting contract pallet migration, pallet version: {:?}",
            PalletVersion::<T>::get()
        );

        // !!! This map storage is re-built from zero !!!

        // 1. Remove all items under ContractsToBillAt
        let _ = frame_support::migration::clear_storage_prefix(
            b"SmartContractModule",
            b"ContractsToBillAt",
            b"",
            None,
            None,
        );

        let mut r = 0u64;
        let mut w = 0u64;

        let billing_frequency = BillingFrequency::<T>::get();
        r.saturating_inc();
        let mut new_billing_loop = vec![vec![]; billing_frequency as usize];

        // 2. Insert contract ids in billing loop based on existing contracts
        for (contract_id, _contract) in Contracts::<T>::iter() {
            r.saturating_inc();
            let index = contract_id % billing_frequency;
            new_billing_loop[index as usize].push(contract_id);
        }

        // 3. Rebuild billing loop storage
        for (index, contract_ids) in new_billing_loop.iter().enumerate() {
            ContractsToBillAt::<T>::insert(index as u64, &contract_ids);
            w.saturating_inc();
        }

        // Update pallet storage version
        PalletVersion::<T>::set(types::StorageVersion::V10);
        w.saturating_inc();
        info!("<<< Storage version upgraded");

        // Return the weight consumed by the migration.
        T::DbWeight::get().reads_writes(r, w)
    } else {
        info!(">>> Unused Smart Contract pallet V10 migration");
        Weight::zero()
    }
}
