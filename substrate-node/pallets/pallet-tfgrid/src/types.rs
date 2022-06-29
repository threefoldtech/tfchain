use codec::{Decode, Encode};
use core::cmp::Ordering;
use scale_info::TypeInfo;
use sp_std::vec::Vec;
use tfchain_support::types::{FarmCertification, NodeCertification};

/// Utility type for managing upgrades/migrations.
#[derive(Encode, Decode, Clone, Debug, PartialEq, TypeInfo)]
pub enum StorageVersion {
    V1Struct,
    V2Struct,
    V3Struct,
    V4Struct,
    V5Struct,
}

impl Default for StorageVersion {
    fn default () -> StorageVersion {
        StorageVersion::V1Struct
    }
}

#[derive(Encode, Decode, Debug, Default, PartialEq, Eq, Clone, TypeInfo)]
pub struct Entity<AccountId> {
    pub version: u32,
    pub id: u32,
    pub name: Vec<u8>,
    pub account_id: AccountId,
    pub country: Vec<u8>,
    pub city: Vec<u8>,
}

//digital twin
#[derive(Clone, Encode, Decode, Debug, Eq, PartialEq, Default, TypeInfo)]
pub struct Twin<AccountId> {
    pub version: u32,
    pub id: u32,
    //substrate account id = public key (32 bytes)
    //also used by PAN network
    pub account_id: AccountId,
    pub ip: Vec<u8>,
    //link to person's or companies who own this twin
    pub entities: Vec<EntityProof>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
pub struct EntityProof {
    pub entity_id: u32,
    pub signature: Vec<u8>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
pub struct PricingPolicy<AccountId> {
    pub version: u32,
    pub id: u32,
    pub name: Vec<u8>,
    pub su: Policy,
    pub cu: Policy,
    pub nu: Policy,
    pub ipu: Policy,
    pub unique_name: Policy,
    pub domain_name: Policy,
    pub foundation_account: AccountId,
    pub certified_sales_account: AccountId,
    pub discount_for_dedication_nodes: u8,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
pub struct Policy {
    pub value: u32,
    pub unit: Unit,
}

impl Policy {
    pub fn factor(&self) -> u128 {
        match self.unit {
            Unit::Bytes => 1,
            Unit::Kilobytes => 1000,
            Unit::Megabytes => 1000 * 1000,
            Unit::Gigabytes => 1000 * 1000 * 1000,
            Unit::Terrabytes => 1000 * 1000 * 1000 * 1000,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug, TypeInfo)]
pub enum Unit {
    Bytes,
    Kilobytes,
    Megabytes,
    Gigabytes,
    Terrabytes,
}

impl Unit {
    pub fn from_u32(number: u32) -> Unit {
        match number {
            1 => Unit::Bytes,
            2 => Unit::Kilobytes,
            3 => Unit::Megabytes,
            4 => Unit::Gigabytes,
            5 => Unit::Terrabytes,
            _ => Unit::default(),
        }
    }
}

impl Default for Unit {
    fn default() -> Unit {
        Unit::Gigabytes
    }
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Default, Debug, TypeInfo)]
pub struct FarmingPolicy<BlockNumber> {
    pub version: u32,
    pub id: u32,
    pub name: Vec<u8>,
    // CU: expressed as mUSD / period
    pub cu: u32,
    // SU: expressed as mUSD / period
    pub su: u32,
    // NU: epxressed as mUSD
    pub nu: u32,
    // IPV4: expressed as mUSD / hour
    pub ipv4: u32,
    // Minimal uptime in order to benefit from this uptime.
    pub minimal_uptime: u16,
    pub policy_created: BlockNumber,
    // Indicated when this policy expires.
    pub policy_end: BlockNumber,
    // If this policy is immutable or not. Immutable policies can never be changed again.
    pub immutable: bool,
    // Indicates if the farming policy is a default one. Meaning it will be used when there is no
    // Farming policy defined on the farm itself
    pub default: bool,
    // If a node needs to be certified or not to benefit from this policy
    pub node_certification: NodeCertification,
    // Farm certification level
    pub farm_certification: FarmCertification,
}

impl<B> PartialOrd for FarmingPolicy<B>
where
    B: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<B> Ord for FarmingPolicy<B>
where
    B: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match self.farm_certification.cmp(&other.farm_certification) {
            Ordering::Equal => self.node_certification.cmp(&other.node_certification),
            ord => ord,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
pub struct TermsAndConditions<AccountId> {
    pub account_id: AccountId,
    pub timestamp: u64,
    pub document_link: Vec<u8>,
    pub document_hash: Vec<u8>,
}
