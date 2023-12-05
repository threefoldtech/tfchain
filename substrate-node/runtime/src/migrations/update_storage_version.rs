use frame_support::pallet_prelude::PhantomData;

pub struct UpdateStorageVersion;
impl frame_support::traits::OnRuntimeUpgrade for UpdateStorageVersion {
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
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
