use codec::{Decode, Encode};
use core::cmp::{Ord, Ordering, PartialOrd};
use frame_support::traits::Vec;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct Farm {
    pub version: u32,
    pub id: u32,
    pub name: Vec<u8>,
    pub twin_id: u32,
    pub pricing_policy_id: u32,
    pub certification: Certification,
    pub public_ips: Vec<PublicIP>,
    pub dedicated_farm: bool,
    pub farming_policy_limits: Option<FarmingPolicyLimit>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct PublicIP {
    pub ip: Vec<u8>,
    pub gateway: Vec<u8>,
    pub contract_id: u64,
}

#[derive(PartialEq, PartialOrd, Eq, Clone, Encode, Decode, Debug, Copy)]
pub enum Certification {
    NotCertified,
    Gold,
}

impl Default for Certification {
    fn default() -> Certification {
        Certification::NotCertified
    }
}

impl Ord for Certification {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Certification::Gold if matches!(other, Certification::NotCertified) => {
                Ordering::Greater
            }
            Certification::NotCertified if matches!(other, Certification::Gold) => Ordering::Less,
            _ => Ordering::Equal,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct FarmingPolicyLimit {
    pub farming_policy_id: u32,
    pub cu: Option<u64>,
    pub su: Option<u64>,
    pub end: Option<u64>,
    pub node_count: Option<u32>,
    pub node_certification: bool,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct Node {
    pub version: u32,
    pub id: u32,
    pub farm_id: u32,
    pub twin_id: u32,
    pub resources: Resources,
    pub location: Location,
    pub country: Vec<u8>,
    pub city: Vec<u8>,
    // optional public config
    pub public_config: Option<PublicConfig>,
    pub created: u64,
    pub farming_policy_id: u32,
    pub interfaces: Vec<Interface>,
    pub certification_type: CertificationType,
    pub secure_boot: bool,
    pub virtualized: bool,
    pub serial_number: Vec<u8>,
    pub connection_price: u32,
}

pub type IP = Vec<u8>;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct Interface {
    pub name: Vec<u8>,
    pub mac: Vec<u8>,
    pub ips: Vec<IP>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct PublicConfig {
    pub ipv4: Vec<u8>,
    pub ipv6: Vec<u8>,
    pub gw4: Vec<u8>,
    pub gw6: Vec<u8>,
    pub domain: Vec<u8>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, Copy)]
pub struct Resources {
    pub hru: u64,
    pub sru: u64,
    pub cru: u64,
    pub mru: u64,
}

impl Resources {
    pub fn add(mut self, other: &Resources) -> Resources {
        self.cru += other.cru;
        self.sru += other.sru;
        self.hru += other.hru;
        self.mru += other.mru;
        self
    }
}

// Store Location long and lat as string
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct Location {
    pub longitude: Vec<u8>,
    pub latitude: Vec<u8>,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug, Copy)]
pub enum CertificationType {
    Diy,
    Certified,
}

impl Ord for CertificationType {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            CertificationType::Certified if matches!(other, CertificationType::Diy) => {
                Ordering::Greater
            }
            CertificationType::Diy if matches!(other, CertificationType::Certified) => {
                Ordering::Less
            }
            _ => Ordering::Equal, // technically this is unreachable but I don't care at this point
        }
    }
}

impl PartialOrd for CertificationType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Default for CertificationType {
    fn default() -> CertificationType {
        CertificationType::Diy
    }
}
