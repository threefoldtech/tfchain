pub struct UpdateStorageVersion;
impl frame_support::traits::OnRuntimeUpgrade for UpdateStorageVersion {
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        frame_support::pallet_prelude::StorageVersion::new(1).put::<crate::Balances>();
        frame_support::pallet_prelude::StorageVersion::new(4).put::<crate::Scheduler>();
        frame_support::pallet_prelude::StorageVersion::new(1).put::<crate::Historical>();

        frame_support::weights::Weight::from_all(3)
    }
}
