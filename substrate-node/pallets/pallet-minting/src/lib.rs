#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{dispatch::DispatchResultWithPostInfo, ensure, traits::Get};
// use log::info;
use pallet_timestamp as timestamp;
use sp_runtime::traits::SaturatedConversion;
pub mod types;
pub use pallet::*;
use pallet_tfgrid::pallet::{InterfaceOf, LocationOf, SerialNumberOf, TfgridNode};
use tfchain_support::{
    resources::Resources,
    traits::{ChangeNode, MintingHook},
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use super::types;
    use frame_support::pallet_prelude::{OptionQuery, *};
    use frame_support::{traits::Currency, Blake2_128Concat};
    use frame_system::pallet_prelude::*;
    use sp_std::convert::TryInto;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // Period is a unix timestamp
    pub type Period = u64;
    pub type NodeId = u32;

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + pallet_tfgrid::Config
        + pallet_timestamp::Config
        + pallet_balances::Config
    {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Currency: Currency<Self::AccountId>;
        // Allowed time for an uptime to drift
        type AllowedUptimeDrift: Get<u64>;
        // Treshold for period, indicates how long a period lasts
        type PeriodTreshold: Get<u64>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        MintTransactionCreated(),
        NodePeriodStarted { node_id: NodeId, start: Period },
        UptimeReportReceived { node_id: NodeId, uptime: u64 },
    }

    #[pallet::error]
    pub enum Error<T> {
        UptimeReportInvalid,
    }

    #[pallet::storage]
    #[pallet::getter(fn mints)]
    pub type Mints<T> = StorageValue<_, u64, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn payable_mints)]
    pub type PayableMints<T> = StorageMap<
        _,
        Blake2_128Concat,
        NodeId,
        Vec<types::MintingPayout<BalanceOf<T>>>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn node_periods)]
    pub type NodeReport<T: Config> =
        StorageMap<_, Blake2_128Concat, NodeId, types::Report, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn node_counters)]
    pub type NodeCounters<T: Config> =
        StorageMap<_, Blake2_128Concat, NodeId, types::NodeCounters, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(100_000_000)]
        pub fn report_uptime(origin: OriginFor<T>, uptime: u64) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;

            Self::process_uptime_report(&account_id, uptime)?;

            Ok(Pays::No.into())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn process_uptime_report(
        account_id: &T::AccountId,
        uptime: u64,
    ) -> DispatchResultWithPostInfo {
        ensure!(uptime > 0, Error::<T>::UptimeReportInvalid);

        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(pallet_tfgrid::Error::<T>::TwinNotExists)?;

        let node_id = pallet_tfgrid::NodeIdByTwinID::<T>::get(twin_id);
        let _ = pallet_tfgrid::Nodes::<T>::get(node_id)
            .ok_or(pallet_tfgrid::Error::<T>::NodeNotExists)?;

        let now = Self::get_unix_timestamp();

        let mut node_report = NodeReport::<T>::get(node_id);

        if node_report.period_start == 0 {
            // If it's the first node report, start the period
            node_report.period_start = now;
            Self::deposit_event(Event::NodePeriodStarted {
                node_id,
                start: now,
            });
        } else if now >= node_report.period_start + T::PeriodTreshold::get() {
            let time_elapsed_period = now - node_report.period_start;
            // TODO:
            // - calculate up / down time
            let down_time = time_elapsed_period
                .checked_sub(node_report.uptime)
                .unwrap_or(0);
            // - calculate remaining seconds in old period and subtract from downtime, if not sufficient, subtract from uptime
            let period_remaining_seconds = time_elapsed_period
                .checked_sub(T::PeriodTreshold::get())
                .unwrap_or(0);

            let mut diff = 0;
            if down_time > 0 {
                diff = down_time.checked_sub(period_remaining_seconds).unwrap_or(0);
            }
            if diff > 0 {
                node_report.uptime.checked_sub(down_time).unwrap_or(0);
            }
            // TODO - calculate minting rewards for this period

            // Advance node period start with threshold
            node_report.period_start = node_report.period_start + T::PeriodTreshold::get();
            // Save last period uptime for future calculation
            node_report.last_period_uptime = uptime;
        }

        // Validate report
        // There might be some network latency. As such, it is required to implement a small window for which the report will be accepted,
        // in which case the uptime increment is equal to the difference of uptimes.
        // If the difference of uptime is Δuptime and the difference of report times is Δr, then a report is
        // valid if Δr - allowed_drift_time <= Δuptime <= Δr + allowed_drift_time. We choose Δuptime as the amount to increment uptime because this is the amount reported
        // by the node and should be free of any latency issues w.r.t. network or block production.

        // Δr
        let report_diff = match node_report.last_updated {
            0 => 0,
            _ => now.checked_sub(node_report.last_updated).unwrap_or(0),
        };
        // Δuptime
        // If the saved report uptime is 0, initialize with the current sent uptime
        let uptime_diff = match node_report.uptime {
            0 => uptime,
            _ => uptime.checked_sub(node_report.uptime).unwrap_or(0),
        };

        log::info!("report diff: {:?}", report_diff);
        log::info!("uptime diff: {:?}", uptime_diff);

        if uptime_diff > 0 && report_diff > 0 {
            // Validate report
            let valid_report = report_diff
                .checked_sub(T::AllowedUptimeDrift::get())
                .unwrap_or(0)
                <= uptime_diff
                && uptime_diff <= report_diff + T::AllowedUptimeDrift::get();
            // TODO: if invalid, report offence?
            ensure!(valid_report, Error::<T>::UptimeReportInvalid);
        }

        // Save report
        node_report.last_updated = now;
        node_report.uptime += uptime_diff;
        NodeReport::<T>::insert(node_id, node_report);

        Self::deposit_event(Event::UptimeReportReceived { node_id, uptime });

        Ok(().into())
    }

    pub fn get_unix_timestamp() -> u64 {
        <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000
    }
}

impl<T: Config> MintingHook<T::AccountId> for Pallet<T> {
    fn report_uptime(source: &T::AccountId, uptime: u64) -> DispatchResultWithPostInfo {
        Self::process_uptime_report(source, uptime)
    }

    fn report_nru(source: &T::AccountId, nru: u64, window: u64) -> DispatchResultWithPostInfo {
        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(source)
            .ok_or(pallet_tfgrid::Error::<T>::TwinNotExists)?;

        let node_id = pallet_tfgrid::NodeIdByTwinID::<T>::get(twin_id);

        let mut node_counters = NodeCounters::<T>::get(node_id);
        node_counters.nru = nru * window;
        NodeCounters::<T>::insert(node_id, node_counters);

        Ok(().into())
    }

    fn report_used_resources(
        node_id: u32,
        dedicated: bool,
        resources: Resources,
        window: u64,
    ) -> DispatchResultWithPostInfo {
        // Todo, update used resources counters
        let _ = pallet_tfgrid::Nodes::<T>::get(node_id)
            .ok_or(pallet_tfgrid::Error::<T>::NodeNotExists)?;

        let mut _node_counters = NodeCounters::<T>::get(node_id);

        // TODO: process used resources for a node

        Ok(().into())
    }
}

impl<T: Config> ChangeNode<LocationOf<T>, InterfaceOf<T>, SerialNumberOf<T>> for Pallet<T> {
    fn node_changed(node: Option<&TfgridNode<T>>, new_node: &TfgridNode<T>) {
        match node {
            // If an old node is passed, it means the node got updated
            Some(old_node) => {
                if Resources::has_changed(&old_node.resources, &new_node.resources, 1) {
                    let mut node_counters = NodeCounters::<T>::get(new_node.id);
                    // If the resources are increased we need to update the max capacity
                    // But we also need to check if the connectionprice is still the same as when the node connected
                    // Otherwise we will not allow an update
                    if new_node.resources > node_counters.max_capacity
                        && new_node.connection_price == pallet_tfgrid::ConnectionPrice::<T>::get()
                    {
                        node_counters.max_capacity = new_node.resources.clone();
                    }
                    // Update counters
                    NodeCounters::<T>::insert(new_node.id, node_counters);
                }
            }
            // If no old node is passed, it means we got a new node
            None => {
                // Save a new node's min/max resources to current resources
                let mut node_counters = NodeCounters::<T>::get(new_node.id);
                node_counters.min_capacity = new_node.resources.clone();
                node_counters.max_capacity = node_counters.min_capacity.clone();
                NodeCounters::<T>::insert(new_node.id, node_counters);
            }
        }
    }

    fn node_deleted(_node: &TfgridNode<T>) {
        // TODO: handle payout?
    }
}
