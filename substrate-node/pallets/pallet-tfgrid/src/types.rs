use codec::{Decode, Encode};
use core::cmp::Ordering;
use frame_support::{pallet_prelude::ConstU32, BoundedVec};
use scale_info::TypeInfo;
use sp_std::vec::Vec;
use tfchain_support::types::{FarmCertification, NodeCertification};

/// Utility type for managing upgrades/migrations.
#[derive(Encode, Decode, Clone, Debug, PartialEq, PartialOrd, TypeInfo)]
pub enum StorageVersion {
    V1Struct,
    V2Struct,
    V3Struct,
    V4Struct,
    V5Struct,
    V6Struct,
    V7Struct,
    V8Struct,
    V9Struct,
    V10Struct,
    V11Struct,
    V12Struct,
    V13Struct,
    V14Struct,
    V15Struct,
}

impl Default for StorageVersion {
    fn default() -> StorageVersion {
        StorageVersion::V14Struct
    }
}

#[derive(Encode, Decode, Debug, Default, PartialEq, Eq, Clone, TypeInfo)]
pub struct Entity<AccountId, City, Country> {
    pub version: u32,
    pub id: u32,
    pub name: Vec<u8>,
    pub account_id: AccountId,
    pub country: Country,
    pub city: City,
}

pub const MAX_PK_LENGTH: u32 = 128; // limited to 128 bytes
pub const MAX_RELAY_LENGTH: u32 = 1024; // limited to 1024 bytes (half of max url size)

//digital twin
#[derive(Clone, Encode, Decode, Debug, Eq, PartialEq, Default, TypeInfo)]
pub struct Twin<AccountId> {
    pub id: u32,
    // substrate account id = public key (32 bytes)
    pub account_id: AccountId,
    // relay address (proxy)
    pub relay: Option<BoundedVec<u8, ConstU32<MAX_RELAY_LENGTH>>>,
    // link to person's or companies who own this twin
    pub entities: Vec<EntityProof>,
    // public key of the encryption key used in rmb
    pub pk: Option<BoundedVec<u8, ConstU32<MAX_PK_LENGTH>>>,
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
    // Used for NRU
    pub fn factor_base_1000(&self) -> u128 {
        match self.unit {
            Unit::Bytes => 1,
            Unit::Kilobytes => 1000,
            Unit::Megabytes => 1000 * 1000,
            Unit::Gigabytes => 1000 * 1000 * 1000,
            Unit::Terrabytes => 1000 * 1000 * 1000 * 1000,
        }
    }

    // Used for other units (sru, hru, mru)
    pub fn factor_base_1024(&self) -> u128 {
        match self.unit {
            Unit::Bytes => 1,
            Unit::Kilobytes => 1024,
            Unit::Megabytes => 1024 * 1024,
            Unit::Gigabytes => 1024 * 1024 * 1024,
            Unit::Terrabytes => 1024 * 1024 * 1024 * 1024,
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
    // CU: expressed as mUSD / minting period
    pub cu: u32,
    // SU: expressed as mUSD / minting period
    pub su: u32,
    // NU: epxressed as mUSD / GB
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

impl<B: Ord> PartialOrd for FarmingPolicy<B> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<B: Ord> Ord for FarmingPolicy<B> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.farm_certification.cmp(&other.farm_certification) {
            Ordering::Equal => self.node_certification.cmp(&other.node_certification),
            ord => ord,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
pub struct TermsAndConditionsInput<AccountId, DocLink, DocHash> {
    pub account_id: AccountId,
    pub timestamp: u64,
    pub document_link: DocLink,
    pub document_hash: DocHash,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
pub struct LocationInput<City, Country, Latitude, Longitude> {
    pub city: City,
    pub country: Country,
    pub latitude: Latitude,
    pub longitude: Longitude,
}
