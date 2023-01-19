#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::DispatchResultWithPostInfo,
    ensure,
    traits::{Currency, Get},
};
// use log::info;
use pallet_timestamp as timestamp;
use sp_runtime::traits::SaturatedConversion;
pub mod types;
pub use pallet::*;
use pallet_tfgrid::pallet::{InterfaceOf, LocationOf, SerialNumberOf, TfgridNode};
use tfchain_support::{
    resources::{Resources, GIGABYTE, ONE_MILL},
    traits::{ChangeNode, MintingHook},
    types::FarmCertification,
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use super::types;
    use frame_support::pallet_prelude::*;
    use frame_support::{traits::Currency, Blake2_128Concat};
    use frame_system::pallet_prelude::*;
    use sp_std::{convert::TryInto, vec::Vec};

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // Period is a unix timestamp
    pub type Period = u64;
    pub type NodeId = u32;

    pub type BalanceOf<T> =
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
        // Allowed time for an uptime to drift in seconds
        type AllowedUptimeDrift: Get<u64>;
        // Treshold for period, indicates how long a period lasts in seconds
        type PeriodTreshold: Get<u64>;
        // Heartbeat interval indicates the maximum interval in seconds where a node can send uptime reports
        type HeartbeatInterval: Get<u64>;
        type EnableMintingUnixTime: Get<u64>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        MintTransactionCreated(),
        NodePeriodStarted {
            node_id: NodeId,
            start: Period,
        },
        UptimeReportReceived {
            node_id: NodeId,
            uptime: u64,
        },
        NodePeriodEnded {
            node_id: NodeId,
        },
        NodePeriodPaidOut {
            node_id: NodeId,
            amount: BalanceOf<T>,
            period_info: types::NodePeriodInformation,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        UptimeReportInvalid,
        UnauthorizedToTriggerPayout,
        UptimeNotMetInPeriodFollowingSla,
    }

    #[pallet::storage]
    #[pallet::getter(fn node_periods)]
    pub type NodeReport<T: Config> =
        StorageMap<_, Blake2_128Concat, NodeId, types::Report, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn payable_periods)]
    pub type PayablePeriods<T> =
        StorageMap<_, Blake2_128Concat, NodeId, Vec<types::NodePeriodInformation>, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(100_000_000)]
        pub fn report_uptime(origin: OriginFor<T>, uptime: u64) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;

            Self::process_uptime_report(&account_id, uptime)?;

            Ok(Pays::No.into())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(100_000_000)]
        pub fn payout_periods(origin: OriginFor<T>, node_id: u32) -> DispatchResultWithPostInfo {
            let account_id = ensure_signed(origin)?;

            Self::payout_node_periods(&account_id, node_id)?;

            Ok(Pays::Yes.into())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn process_uptime_report(
        account_id: &T::AccountId,
        uptime: u64,
    ) -> DispatchResultWithPostInfo {
        let now = Self::get_unix_timestamp();
        if now < T::EnableMintingUnixTime::get() {
            return Ok(().into());
        }

        ensure!(uptime > 0, Error::<T>::UptimeReportInvalid);

        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(pallet_tfgrid::Error::<T>::TwinNotExists)?;

        let node_id = pallet_tfgrid::NodeIdByTwinID::<T>::get(twin_id);
        let node = pallet_tfgrid::Nodes::<T>::get(node_id)
            .ok_or(pallet_tfgrid::Error::<T>::NodeNotExists)?;

        let mut node_report = NodeReport::<T>::get(node_id);

        let uptime = Self::validate_uptime_report(now, uptime, &node_report)?;
        // Save report
        node_report.last_updated = now;

        if node_report.period_start == 0 {
            // If it's the first node report, start the period
            node_report.period_start = now;
            // Save farming policy
            node_report.counters.farming_policy = node.farming_policy_id;
            // Initialise min capacity
            if node_report.counters.min_capacity == Resources::default() {
                node_report.counters.min_capacity = node.resources;
            }
            // Deposit event
            Self::deposit_event(Event::NodePeriodStarted {
                node_id,
                start: now,
            });
        } else if now >= node_report.period_start + T::PeriodTreshold::get() {
            Self::end_period(node_id, now, node.farming_policy_id, &mut node_report);
        }

        node_report.counters.uptime += uptime;

        // Add running capacity
        // Running capacity is the minimal capacity that is expressed in seconds
        node_report.counters.running_capacity = node_report
            .counters
            .running_capacity
            .add(node_report.counters.min_capacity, uptime);

        NodeReport::<T>::insert(node_id, &node_report);

        Self::deposit_event(Event::UptimeReportReceived {
            node_id,
            uptime: node_report.counters.uptime,
        });

        Ok(().into())
    }

    // Validate report
    // There might be some network latency. As such, it is required to implement a small window for which the report will be accepted,
    // in which case the uptime increment is equal to the difference of uptimes.
    // If the difference of uptime is Δuptime and the difference of report times is Δr, then a report is
    // valid if Δr - allowed_drift_time <= Δuptime <= Δr + allowed_drift_time. We choose Δuptime as the amount to increment uptime because this is the amount reported
    // by the node and should be free of any latency issues w.r.t. network or block production.
    // Returns uptime difference between previous uptime
    pub fn validate_uptime_report(
        now: u64,
        uptime: u64,
        node_report: &types::Report,
    ) -> Result<u64, Error<T>> {
        // Δr
        let report_diff = match node_report.last_updated {
            0 => 0,
            _ => now.checked_sub(node_report.last_updated).unwrap_or(0),
        };
        // Δuptime
        // If the saved report uptime is 0, initialize with the current sent uptime
        let uptime_diff = match node_report.counters.uptime {
            0 => uptime,
            _ => uptime.checked_sub(node_report.counters.uptime).unwrap_or(0),
        };

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

        Ok(uptime_diff)
    }

    // Ends a minting period for a node
    pub fn end_period(
        node_id: u32,
        now: u64,
        farming_policy_id: u32,
        mut node_report: &mut types::Report,
    ) {
        let time_elapsed_period = now - node_report.period_start;
        // - calculate up / down time
        let down_time = time_elapsed_period
            .checked_sub(node_report.counters.uptime)
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
            node_report
                .counters
                .uptime
                .checked_sub(down_time)
                .unwrap_or(0);
        }

        // Advance node period start with threshold
        node_report.period_start = node_report.period_start + T::PeriodTreshold::get();
        // Save last period uptime for future calculation
        node_report.last_period_uptime = node_report.counters.uptime;
        // Refetch farming policy
        node_report.counters.farming_policy = farming_policy_id;

        let mut payable_periods = PayablePeriods::<T>::get(node_id);
        payable_periods.push(node_report.counters);
        PayablePeriods::<T>::insert(node_id, payable_periods);

        Self::deposit_event(Event::NodePeriodEnded { node_id });
        // Reset counters
        node_report.counters = types::NodePeriodInformation::default();
    }

    // This method will calculate the minting rewards for all the periods that the node has accumulated
    // Can only called by the farmer that owns the node
    pub fn payout_node_periods(
        account_id: &T::AccountId,
        node_id: u32,
    ) -> DispatchResultWithPostInfo {
        let node = pallet_tfgrid::Nodes::<T>::get(node_id)
            .ok_or(pallet_tfgrid::Error::<T>::NodeNotExists)?;

        let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id)
            .ok_or(pallet_tfgrid::Error::<T>::TwinNotExists)?;

        let farm = pallet_tfgrid::Farms::<T>::get(node.farm_id)
            .ok_or(pallet_tfgrid::Error::<T>::FarmNotExists)?;

        ensure!(
            twin_id == farm.twin_id,
            Error::<T>::UnauthorizedToTriggerPayout
        );

        let mut payable_periods = PayablePeriods::<T>::get(node_id);
        log::debug!("period: {:?}", payable_periods.clone());
        payable_periods = payable_periods
            .into_iter()
            .filter(|period| {
                match Self::payout_period(&node, node.farm_id, node.connection_price, period) {
                    Ok(_) => true,
                    Err(e) => {
                        log::debug!("payout failed: {:?}", e);
                        false
                    }
                }
            })
            .collect();
        PayablePeriods::<T>::insert(node_id, payable_periods);

        Ok(().into())
    }

    pub fn payout_period(
        node: &pallet_tfgrid::TfgridNode<T>,
        farm_id: u32,
        connection_price: u32,
        period: &types::NodePeriodInformation,
    ) -> DispatchResultWithPostInfo {
        log::debug!("min capacity: {:?}", period.min_capacity);
        let (cu, su) = period.min_capacity.cloud_units_permill();
        log::debug!("cu: {}", cu);
        log::debug!("su: {}", su);
        let period_length = T::PeriodTreshold::get();

        let farming_policy = pallet_tfgrid::FarmingPoliciesMap::<T>::get(period.farming_policy);
        let farm = pallet_tfgrid::Farms::<T>::get(farm_id)
            .ok_or(pallet_tfgrid::Error::<T>::FarmNotExists)?;
        let farm_twin = pallet_tfgrid::Twins::<T>::get(farm.twin_id)
            .ok_or(pallet_tfgrid::Error::<T>::TwinNotExists)?;

        let uptime_percentage = (period.uptime * 1_000 / period_length) as u128;

        let cu_reward = cu * farming_policy.cu as u64;
        let su_reward = su * farming_policy.su as u64;

        if farming_policy.farm_certification == FarmCertification::Gold {
            log::debug!(
                "check: {:?}",
                (period_length * farming_policy.minimal_uptime as u64) / 100
            );
            let minimal_uptime_met =
                period.uptime > (period_length * farming_policy.minimal_uptime as u64) / 1000;
            if !minimal_uptime_met {
                return Err(Error::<T>::UptimeNotMetInPeriodFollowingSla.into());
            }
        }

        log::debug!("cu reward: {}", cu_reward);
        log::debug!("su reward: {}", su_reward);

        // Network traffic rewards are per Gigabyte for a period
        let nru_reward = (period.nru as u128 * ONE_MILL / GIGABYTE) * farming_policy.nu as u128;
        log::debug!("nru reward: {}", nru_reward);

        // Public IP rewards are per public ip per hour for a period
        let ipu_reward = period.ipu * ONE_MILL * farming_policy.ipv4 as u128 / 3600;
        log::debug!("ipu reward: {}", ipu_reward);

        // Base payout is expressed in mUSD (milli USD)
        let mut base_payout =
            (cu_reward as u128 + su_reward as u128 + nru_reward + ipu_reward) / ONE_MILL;
        log::debug!("total musd reward: {}", base_payout);

        // Inflate base payout for more precision
        base_payout = base_payout * 100_000_000;
        // Caculate actual payout for the period uptime
        base_payout = base_payout * uptime_percentage / 1_000;
        log::debug!(
            "total musd reward after inflation and uptime calculation: {}",
            base_payout
        );

        if matches!(
            node.certification,
            tfchain_support::types::NodeCertification::Certified
        ) {
            base_payout = base_payout * 5 / 4;
        }

        log::info!("connection price: {}", connection_price);
        // Calculate the amount of TFT rewarded
        let total_tft_reward = base_payout / connection_price as u128;

        log::info!(
            "Minting: {:?} to farmer twin {:?}",
            total_tft_reward,
            &farm_twin.account_id,
        );
        let amount_as_balance = BalanceOf::<T>::saturated_from(total_tft_reward);
        // This call mints tokens on the target account
        <T as Config>::Currency::deposit_creating(&farm_twin.account_id, amount_as_balance);

        Self::deposit_event(Event::NodePeriodPaidOut {
            node_id: node.id,
            amount: amount_as_balance,
            period_info: period.clone(),
        });

        Ok(().into())
    }

    pub fn get_unix_timestamp() -> u64 {
        <timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000
    }

    pub fn report_nru(node_id: u32, nru: u64, window: u64) {
        let mut node_report = NodeReport::<T>::get(node_id);
        node_report.counters.nru += nru * window;
        NodeReport::<T>::insert(node_id, node_report);
    }

    fn report_used_resources(node_id: u32, resources: Resources, window: u64, ipu: u32) {
        let mut node_report = NodeReport::<T>::get(node_id);

        let now = Self::get_unix_timestamp();
        // We ignore all reports for used resources if the node has not sent a heartbeat within the allowed interval
        if now > node_report.last_updated + T::HeartbeatInterval::get() {
            log::debug!(
                "got a used resources report from node: {}, but that node is not online",
                node_id
            );
            return;
        }

        if !resources.is_empty() {
            node_report.counters.used_capacity =
                node_report.counters.used_capacity.add(resources, window);
        }
        node_report.counters.ipu += (ipu as u64 * window) as u128;
        NodeReport::<T>::insert(node_id, node_report);
    }
}

impl<T: Config> MintingHook<T::AccountId> for Pallet<T> {
    fn report_uptime(source: &T::AccountId, uptime: u64) -> DispatchResultWithPostInfo {
        if Self::get_unix_timestamp() < T::EnableMintingUnixTime::get() {
            return Ok(().into());
        }
        Self::process_uptime_report(source, uptime)
    }

    fn report_nru(node_id: u32, nru: u64, window: u64) {
        if Self::get_unix_timestamp() < T::EnableMintingUnixTime::get() {
            return;
        }
        Self::report_nru(node_id, nru, window)
    }

    fn report_used_resources(node_id: u32, resources: Resources, window: u64, ipu: u32) {
        if Self::get_unix_timestamp() < T::EnableMintingUnixTime::get() {
            return;
        }
        Self::report_used_resources(node_id, resources, window, ipu)
    }
}

impl<T: Config> ChangeNode<LocationOf<T>, InterfaceOf<T>, SerialNumberOf<T>> for Pallet<T> {
    fn node_changed(node: Option<&TfgridNode<T>>, new_node: &TfgridNode<T>) {
        match node {
            // If an old node is passed, it means the node got updated
            Some(old_node) => {
                if Resources::has_changed(&old_node.resources, &new_node.resources, 1) {
                    let mut node_report = NodeReport::<T>::get(new_node.id);
                    // If the resources are increased we need to update the max capacity
                    // But we also need to check if the connectionprice is still the same as when the node connected
                    // Otherwise we will not allow an update
                    if new_node.resources > node_report.counters.max_capacity
                        && new_node.connection_price == pallet_tfgrid::ConnectionPrice::<T>::get()
                    {
                        node_report.counters.max_capacity = new_node.resources.clone();
                    }
                    // Update counters
                    NodeReport::<T>::insert(new_node.id, node_report);
                }
            }
            // If no old node is passed, it means we got a new node
            None => {
                // Save a new node's min/max resources to current resources
                let mut node_report = NodeReport::<T>::get(new_node.id);
                node_report.counters.min_capacity = new_node.resources.clone();
                node_report.counters.max_capacity = node_report.counters.min_capacity.clone();
                NodeReport::<T>::insert(new_node.id, node_report);
            }
        }
    }

    fn node_deleted(_node: &TfgridNode<T>) {
        // TODO: handle payout?
    }
}
