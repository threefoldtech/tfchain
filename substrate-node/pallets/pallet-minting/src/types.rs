use codec::{Decode, Encode};
use scale_info::TypeInfo;

#[derive(Encode, Decode, Clone, Debug, PartialEq, PartialOrd, TypeInfo, Default)]
pub struct Report {
    // Total uptime of a period
    pub uptime: u64,
    // Timestamp indicating when a previous report was received
    pub last_updated: u64,
    // Timestmap indicating when a period started, this will be set the first time a node sends
    // a report
    pub period_start: u64,
    // Last Period Uptime indicates the uptime at the end of last period
    // This to calculate the uptime for current period since the nodes just
    // send uptime from the moment they are booted and we need to track uptime in a period
    pub last_period_uptime: u64,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, PartialOrd, TypeInfo)]
pub struct MintingPayout<B> {
    pub amount: B,
    pub recorded_uptime: u64,
}
