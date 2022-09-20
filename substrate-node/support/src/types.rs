use codec::{Decode, Encode, MaxEncodedLen};
use core::cmp::{Ord, Ordering, PartialOrd};
use frame_support::{traits::ConstU32, BoundedVec};
use scale_info::TypeInfo;
use sp_std::prelude::*;
use super::resources::Resources;

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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
pub struct Node<Location, PubConfig, If, SerialNumber> {
    pub version: u32,
    pub id: u32,
    pub farm_id: u32,
    pub twin_id: u32,
    pub resources: Resources,
    pub location: Location,
    // optional public config
    pub public_config: Option<PubConfig>,
    pub created: u64,
    pub farming_policy_id: u32,
    pub interfaces: Vec<If>,
    pub certification: NodeCertification,
    pub secure_boot: bool,
    pub virtualized: bool,
    pub serial_number: SerialNumber,
    pub connection_price: u32,
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

// Store Location long and lat as string
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
pub struct Location {
    pub city: Vec<u8>,
    pub country: Vec<u8>,
    pub latitude: Vec<u8>,
    pub longitude: Vec<u8>,
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
