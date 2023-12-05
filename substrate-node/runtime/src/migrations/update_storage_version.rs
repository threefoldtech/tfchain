use frame_support::pallet_prelude::{PalletInfoAccess, PhantomData};

pub struct PalletBalancesToV1<T: pallet_balances::Config>(PhantomData<T>);
impl<T: pallet_balances::Config> frame_support::traits::OnRuntimeUpgrade for PalletBalancesToV1<T> {
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        // Remove the old `StorageVersion` type.
        frame_support::storage::unhashed::kill(&frame_support::storage::storage_prefix(
            pallet_balances::Pallet::<T>::name().as_bytes(),
            "StorageVersion".as_bytes(),
        ));

        frame_support::pallet_prelude::StorageVersion::new(1).put::<crate::Balances>();

        frame_support::weights::Weight::from_all(1)
    }
}

pub struct PalletSessionToV1<T: pallet_session::historical::Config>(PhantomData<T>);
impl<T: pallet_session::historical::Config> frame_support::traits::OnRuntimeUpgrade
    for PalletSessionToV1<T>
{
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        pallet_session::migrations::v1::migrate::<T, crate::Historical>()
    }
}
