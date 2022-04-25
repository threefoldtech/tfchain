use super::*;

/// 1. SystemToDualRefCount: from `unique`  to `dual` reference counting.
/// 2. frame_system::Pallet<System>(automatically): from `dual` to dual `triple` reference counting.
pub struct SystemToDualRefCount;
impl frame_support::traits::OnRuntimeUpgrade for SystemToDualRefCount {
    fn on_runtime_upgrade() -> frame_support::weights::Weight {
        frame_system::migrations::migrate_to_dual_ref_count::<Runtime>()
    }
}
