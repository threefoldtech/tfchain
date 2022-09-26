use super::*;

pub struct RemoveBridgeStorage;
impl frame_support::traits::OnRuntimeUpgrade for RemoveBridgeStorage {
    fn on_runtime_upgrade() -> Weight {
        use frame_support::storage::migration;

        let limit = None; // delete all values
        pallet_tft_bridge::MintTransactions::<Runtime>::remove_all(limit);
        pallet_tft_bridge::ExecutedMintTransactions::<Runtime>::remove_all(limit);
        pallet_tft_bridge::BurnTransactions::<Runtime>::remove_all(limit);
        pallet_tft_bridge::ExecutedBurnTransactions::<Runtime>::remove_all(limit);
        pallet_tft_bridge::RefundTransactions::<Runtime>::remove_all(limit);
        pallet_tft_bridge::ExecutedRefundTransactions::<Runtime>::remove_all(limit);

        <Runtime as frame_system::Config>::DbWeight::get().writes(6)
    }
}
