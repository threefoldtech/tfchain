use codec::{Decode, Encode};
use frame_support::traits::Vec;
use tfchain_support::types::{CertificationType};

/// Utility type for managing upgrades/migrations.
#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub enum StorageVersion {
    V1Struct,
    V2Struct,
    V3Struct,
    V4Struct,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Encode, Decode, Default)]
pub struct Entity<AccountId> {
    pub version: u32,
    pub id: u32,
    pub name: Vec<u8>,
    pub account_id: AccountId,
    pub country: Vec<u8>,
    pub city: Vec<u8>,
}

//digital twin
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct EntityProof {
    pub entity_id: u32,
    pub signature: Vec<u8>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
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
    pub discount_for_dedication_nodes: u8
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug)]
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct CertificationCodes {
    pub version: u32,
    pub id: u32,
    pub name: Vec<u8>,
    pub description: Vec<u8>,
    pub certification_code_type: CertificationCodeType,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug)]
pub enum CertificationCodeType {
    Farm,
    Entity,
}

impl Default for CertificationCodeType {
    fn default() -> CertificationCodeType {
        CertificationCodeType::Farm
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct FarmingPolicy {
    pub version: u32,
    pub id: u32,
    pub name: Vec<u8>,
    pub cu: u32,
    pub su: u32,
    pub nu: u32,
    pub ipv4: u32,
    pub timestamp: u64,
    pub certification_type: CertificationType,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct TermsAndConditions<AccountId> {
    pub account_id: AccountId,
    pub timestamp: u64,
    pub document_link: Vec<u8>,
    pub document_hash: Vec<u8>
}