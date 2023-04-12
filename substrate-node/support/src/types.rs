use super::resources::Resources;
use codec::{Decode, Encode, MaxEncodedLen};
use core::cmp::{Ord, Ordering, PartialOrd};
use frame_support::{traits::ConstU32, BoundedVec};
use scale_info::TypeInfo;
use sp_std::prelude::*;
use valip::ip4::{Ip as IPv4, CIDR as IPv4Cidr};
use valip::ip6::{Ip as IPv6, CIDR as IPv6Cidr};

pub const MAX_IP4_LENGTH: u32 = 18;
pub const MAX_GW4_LENGTH: u32 = 15;
pub const MAX_IP6_LENGTH: u32 = 43;
pub const MAX_GW6_LENGTH: u32 = 39;
pub const MAX_DOMAIN_NAME_LENGTH: u32 = 128;

#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo, MaxEncodedLen,
)]
pub struct Farm<Name> {
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
pub struct PublicIP {
    pub ip: BoundedVec<u8, ConstU32<MAX_IP4_LENGTH>>,
    pub gateway: BoundedVec<u8, ConstU32<MAX_GW4_LENGTH>>,
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
pub struct Node<Location, If, SerialNumber> {
    pub version: u32,
    pub id: u32,
    pub farm_id: u32,
    pub twin_id: u32,
    pub resources: Resources,
    pub location: Location,
    // optional public config
    pub public_config: Option<PublicConfig>,
    pub created: u64,
    pub farming_policy_id: u32,
    pub interfaces: Vec<If>,
    pub certification: NodeCertification,
    pub secure_boot: bool,
    pub virtualized: bool,
    pub serial_number: Option<SerialNumber>,
    pub connection_price: u32,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
pub struct Interface<Name, Mac, Ips> {
    pub name: Name,
    pub mac: Mac,
    pub ips: Ips,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
pub struct PublicConfig {
    pub ip4: IP4,
    pub ip6: Option<IP6>,
    pub domain: Option<BoundedVec<u8, ConstU32<MAX_DOMAIN_NAME_LENGTH>>>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
pub struct IP4 {
    pub ip: BoundedVec<u8, ConstU32<MAX_IP4_LENGTH>>,
    pub gw: BoundedVec<u8, ConstU32<MAX_GW4_LENGTH>>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum PublicIpError {
    InvalidIp4,
    InvalidGw4,
    InvalidIp6,
    InvalidGw6,
    InvalidPublicIp,
    InvalidDomain,
}

impl IP4 {
    pub fn is_valid(&self) -> Result<(), PublicIpError> {
        let gw4 = IPv4::parse(&self.gw).map_err(|_| PublicIpError::InvalidGw4)?;
        let ip4 = IPv4Cidr::parse(&self.ip).map_err(|_| PublicIpError::InvalidIp4)?;

        if gw4.is_public()
            && gw4.is_unicast()
            && ip4.is_public()
            && ip4.is_unicast()
            && ip4.contains(gw4)
        {
            Ok(())
        } else {
            Err(PublicIpError::InvalidPublicIp)
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
pub struct IP6 {
    pub ip: BoundedVec<u8, ConstU32<MAX_IP6_LENGTH>>,
    pub gw: BoundedVec<u8, ConstU32<MAX_GW6_LENGTH>>,
}

impl IP6 {
    pub fn is_valid(&self) -> Result<(), PublicIpError> {
        let gw6 = IPv6::parse(&self.gw).map_err(|_| PublicIpError::InvalidGw6)?;
        let ipv6 = IPv6Cidr::parse(&self.ip).map_err(|_| PublicIpError::InvalidIp6)?;

        if gw6.is_public()
            && gw6.is_unicast()
            && ipv6.is_public()
            && ipv6.is_unicast()
            && ipv6.contains(gw6)
        {
            Ok(())
        } else {
            Err(PublicIpError::InvalidPublicIp)
        }
    }
}

impl PublicConfig {
    pub fn is_valid(&self) -> Result<(), PublicIpError> {
        // Validate domain
        if let Some(domain) = &self.domain {
            if !is_valid_domain(&domain) {
                return Err(PublicIpError::InvalidDomain);
            }
        }

        // Validate ip4 config
        self.ip4.is_valid()?;

        // If ip6 config, validate
        if let Some(ip6) = &self.ip6 {
            Ok(ip6.is_valid()?)
        } else {
            Ok(())
        }
    }
}

fn is_valid_domain(input: &[u8]) -> bool {
    input
        .iter()
        .all(|c| matches!(c, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'.'))
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug, TypeInfo, Default)]
pub struct NodePower<B> {
    pub state: PowerState<B>,
    pub target: Power,
}

impl<B> NodePower<B> {
    pub fn is_down(&self) -> bool {
        matches!(self.state, PowerState::Down(_)) || matches!(self.target, Power::Down)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug, TypeInfo)]
pub enum PowerState<B> {
    Up,
    // Down holding the block when it has shut down
    Down(B),
}

impl<B> Default for PowerState<B> {
    fn default() -> Self {
        PowerState::Up
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug, TypeInfo)]
pub enum Power {
    Up,
    Down,
}

impl Default for Power {
    fn default() -> Self {
        Power::Up
    }
}
