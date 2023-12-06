use frame_support::pallet_prelude::{PalletInfoAccess, PhantomData};

use frame_support::{
    pallet_prelude::StorageVersion,
    traits::{GetStorageVersion, OnRuntimeUpgrade},
    weights::Weight,
};

pub struct PalletBalancesToV1<T: pallet_balances::Config>(PhantomData<T>);
impl<T: pallet_balances::Config> OnRuntimeUpgrade for PalletBalancesToV1<T> {
    fn on_runtime_upgrade() -> Weight {
        // Remove the old `StorageVersion` type.
        frame_support::storage::unhashed::kill(&frame_support::storage::storage_prefix(
            pallet_balances::Pallet::<T>::name().as_bytes(),
            "StorageVersion".as_bytes(),
        ));

        StorageVersion::new(1).put::<crate::Balances>();

        Weight::from_all(1)
    }
}

pub struct PalletSessionToV1<T: pallet_session::historical::Config>(PhantomData<T>);
impl<T: pallet_session::historical::Config> OnRuntimeUpgrade for PalletSessionToV1<T> {
    fn on_runtime_upgrade() -> Weight {
        let on_chain_storage_version =
            <crate::Historical as GetStorageVersion>::on_chain_storage_version();

        if on_chain_storage_version < 1 {
            return pallet_session::migrations::v1::migrate::<T, crate::Historical>();
        }

        Weight::zero()
    }
}
