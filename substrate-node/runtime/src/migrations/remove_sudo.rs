use frame_system::Config;
use frame_support::{
    traits::OnRuntimeUpgrade, weights::Weight, storage::migration
};
use log::debug;
use sp_core::Get;
use sp_std::marker::PhantomData;

pub struct RemoveSudo<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for RemoveSudo<T> {
    fn on_runtime_upgrade() -> Weight {
        debug!("Removing Sudo");
        let _ = migration::clear_storage_prefix(b"Sudo", b"Key", b"", None, None);
        debug!("Sudo removed");
        T::DbWeight::get().writes(1)
    }
}