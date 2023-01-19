use codec::{Decode, Encode};
use scale_info::TypeInfo;
use tfchain_support::resources::Resources;

#[derive(Encode, Decode, Clone, Debug, PartialEq, PartialOrd, TypeInfo, Default)]
pub struct Report {
    // Timestamp indicating when a previous report was received (in seconds)
    pub last_updated: u64,
    // Timestmap indicating when a period started (in seconds),
    // this will be set the first time a node sends a report
    pub period_start: u64,
    // Last Period Uptime indicates the uptime at the end of last period (in seconds)
    // This to calculate the uptime for current period since the nodes just
    // send uptime from the moment they are booted and we need to track uptime in a period
    pub last_period_uptime: u64,
    // Internal counters
    pub counters: NodePeriodInformation,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, PartialOrd, TypeInfo, Default, Copy)]
pub struct NodePeriodInformation {
    // Total uptime of a period (in seconds)
    pub uptime: u64,
    // Farming policy link for a node during a period
    pub farming_policy: u32,
    // Capacity trackers
    pub max_capacity: Resources,
    pub min_capacity: Resources,
    pub running_capacity: ResourceSeconds,
    pub used_capacity: ResourceSeconds,
    // Ipu is the amount of ip addresses assigned to the node in unit seconds
    pub ipu: u128,
    // Nru is the amount of public traffic used on the node (in bytes)
    pub nru: u64,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, PartialOrd, TypeInfo, Default, Copy)]
pub struct ResourceSeconds {
    pub hru: u128, // HDD Storage resource units in time (in bytes * seconds)
    pub sru: u128, // SSD Storage resource units in time (in bytes * seconds)
    pub cru: u128, // Compute resource units in time (in vCPU * seconds)
    pub mru: u128, // Memory resource units in time (in bytes * seconds)
}

impl ResourceSeconds {
    pub fn add(mut self, capacity: Resources, seconds: u64) -> ResourceSeconds {
        self.cru += (capacity.cru * seconds) as u128;
        self.sru += (capacity.sru * seconds) as u128;
        self.hru += (capacity.hru * seconds) as u128;
        self.mru += (capacity.mru * seconds) as u128;
        self
    }
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, PartialOrd, TypeInfo)]
pub struct MintingPayout<B> {
    pub amount: B,
    pub recorded_uptime: u64,
}
