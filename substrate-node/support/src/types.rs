use codec::{Decode, Encode, MaxEncodedLen};
use core::cmp::{Ord, Ordering, PartialOrd};
use frame_support::{traits::ConstU32, BoundedVec};
use scale_info::TypeInfo;
use sp_std::prelude::*;

#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo, MaxEncodedLen,
)]
pub struct Farm<Name, PublicIP> {
    pub version: u32,
    pub id: u32,
    pub name: Name,
    pub twin_id: u32,
    pub pricing_policy_id: u32,
    pub certification: FarmCertification,
    pub public_ips: BoundedVec<PublicIP, ConstU32<256>>,
    pub dedicated_farm: bool,
    pub farming_policy_limits: Option<FarmingPolicyLimit>,
}

#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo, MaxEncodedLen,
)]
pub struct PublicIP<Ip, Gateway> {
    pub ip: Ip,
    pub gateway: Gateway,
    pub contract_id: u64,
}

#[derive(
    PartialEq, PartialOrd, Eq, Clone, Encode, Decode, Debug, Copy, TypeInfo, MaxEncodedLen,
)]
pub enum FarmCertification {
    NotCertified,
    Gold,
}

impl Default for FarmCertification {
    fn default() -> FarmCertification {
        FarmCertification::NotCertified
    }
}

impl Ord for FarmCertification {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            FarmCertification::Gold if matches!(other, FarmCertification::NotCertified) => {
                Ordering::Greater
            }
            FarmCertification::NotCertified if matches!(other, FarmCertification::Gold) => {
                Ordering::Less
            }
            _ => Ordering::Equal,
        }
    }
}

#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo, MaxEncodedLen,
)]
pub struct FarmingPolicyLimit {
    pub farming_policy_id: u32,
    pub cu: Option<u64>,
    pub su: Option<u64>,
    pub end: Option<u64>,
    pub node_count: Option<u32>,
    pub node_certification: bool,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug, TypeInfo)]
pub enum CapacityReservationPolicy {
    Any {
        resources: Resources,
        features: Option<Vec<NodeFeatures>>,
    },
    Exclusive {
        group_id: u32,
        resources: Resources,
        features: Option<Vec<NodeFeatures>>,
    },
    Node {
        node_id: u32,
    },
}
impl Default for CapacityReservationPolicy {
    fn default() -> CapacityReservationPolicy {
        CapacityReservationPolicy::Any {
            resources: Resources::empty(),
            features: None,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug, TypeInfo)]
pub enum NodeFeatures {
    PublicNode,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug, TypeInfo)]
pub enum PowerTarget {
    Up,
    Down,
}

impl Default for PowerTarget {
    fn default() -> PowerTarget {
        PowerTarget::Down
    }
}

#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo, MaxEncodedLen,
)]
pub struct ConsumableResources {
    pub total_resources: Resources,
    pub used_resources: Resources,
}

impl ConsumableResources {
    pub fn can_consume_resources(&self, resources: &Resources) -> bool {
        (self.total_resources.hru - self.used_resources.hru) >= resources.hru
            && (self.total_resources.sru - self.used_resources.sru) >= resources.sru
            && (self.total_resources.cru - self.used_resources.cru) >= resources.cru
            && (self.total_resources.mru - self.used_resources.mru) >= resources.mru
    }

    pub fn consume(&mut self, resources: &Resources) {
        self.used_resources = self.used_resources.add(&resources);
    }

    pub fn free(&mut self, resources: &Resources) {
        self.used_resources = self.used_resources.substract(&resources);
    }

    pub fn calculate_increase_in_resources(&self, resources: &Resources) -> Resources {
        resources.substract(&self.total_resources)
    }

    pub fn calculate_reduction_in_resources(&self, resources: &Resources) -> Resources {
        self.total_resources.substract(&resources)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
pub struct Node<PubConfig, If> {
    pub version: u32,
    pub id: u32,
    pub farm_id: u32,
    pub twin_id: u32,
    pub resources: ConsumableResources,
    pub location: Location,
    pub country: Vec<u8>,
    pub city: Vec<u8>,
    pub power_target: PowerTarget,
    // optional public config
    pub public_config: Option<PubConfig>,
    pub created: u64,
    pub farming_policy_id: u32,
    pub interfaces: Vec<If>,
    pub certification: NodeCertification,
    pub secure_boot: bool,
    pub virtualized: bool,
    pub serial_number: Vec<u8>,
    pub connection_price: u32,
}

impl<PubConfig, If> Node<PubConfig, If> {
    pub fn can_be_shutdown(&self) -> bool {
        self.resources.used_resources.is_empty()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
pub struct Interface<Name, Mac, Ips> {
    pub name: Name,
    pub mac: Mac,
    pub ips: Ips,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
pub struct PublicConfig<IP4, IP6, Domain> {
    pub ip4: IP4,
    pub ip6: IP6,
    pub domain: Domain,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
pub struct IP<IpAddr, Gw> {
    pub ip: IpAddr,
    pub gw: Gw,
}

#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Encode,
    Decode,
    Default,
    Debug,
    TypeInfo,
    Copy,
    MaxEncodedLen,
)]
pub struct Resources {
    pub hru: u64,
    pub sru: u64,
    pub cru: u64,
    pub mru: u64,
}

impl Resources {
    pub fn empty() -> Resources {
        Resources {
            hru: 0,
            sru: 0,
            cru: 0,
            mru: 0,
        }
    }

    pub fn is_empty(self) -> bool {
        self.cru == 0 && self.sru == 0 && self.hru == 0 && self.mru == 0
    }

    pub fn add(mut self, other: &Resources) -> Resources {
        self.cru += other.cru;
        self.sru += other.sru;
        self.hru += other.hru;
        self.mru += other.mru;
        self
    }

    pub fn can_substract(self, other: &Resources) -> bool {
        self.cru >= other.cru
            && self.sru >= other.sru
            && self.hru >= other.hru
            && self.mru >= other.mru
    }

    pub fn substract(mut self, other: &Resources) -> Resources {
        self.cru = if self.cru < other.cru {
            0
        } else {
            self.cru - other.cru
        };
        self.sru = if self.sru < other.sru {
            0
        } else {
            self.sru - other.sru
        };
        self.hru = if self.hru < other.hru {
            0
        } else {
            self.hru - other.hru
        };
        self.mru = if self.mru < other.mru {
            0
        } else {
            self.mru - other.mru
        };

        self
    }
}

// Store Location long and lat as string
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
pub struct Location {
    pub longitude: Vec<u8>,
    pub latitude: Vec<u8>,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug, TypeInfo, Copy)]
pub enum NodeCertification {
    Diy,
    Certified,
}

impl Ord for NodeCertification {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            NodeCertification::Certified if matches!(other, NodeCertification::Diy) => {
                Ordering::Greater
            }
            NodeCertification::Diy if matches!(other, NodeCertification::Certified) => {
                Ordering::Less
            }
            _ => Ordering::Equal, // technically this is unreachable but I don't care at this point
        }
    }
}

impl PartialOrd for NodeCertification {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Default for NodeCertification {
    fn default() -> NodeCertification {
        NodeCertification::Diy
    }
}
