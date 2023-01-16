use codec::{Decode, Encode};
use scale_info::TypeInfo;
use tfchain_support::resources::Resources;

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

#[derive(Encode, Decode, Clone, Debug, PartialEq, PartialOrd, TypeInfo, Default)]
pub struct NodeCounters {
    pub max_capacity: Resources,
    pub min_capacity: Resources,
    pub running_capacity: ResourceCounter,
    pub used_capacity: ResourceCounter,
    // Ipu is the amount of ip addresses assigned to the node in unit seconds
    pub ipu: u64,
    // Nru is the amount of public traffic used on the node (in bytes)
    pub nru: u64,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, PartialOrd, TypeInfo, Default)]
pub struct ResourceCounter {
    pub resources: ResourceSeconds,
    // Window is a time in seconds the above resources are used.
    pub window: u64,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, PartialOrd, TypeInfo, Default)]
pub struct ResourceSeconds {
    pub hru: u128,
    pub sru: u128,
    pub cru: u128,
    pub mru: u128,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, PartialOrd, TypeInfo)]
pub struct MintingPayout<B> {
    pub amount: B,
    pub recorded_uptime: u64,
}
