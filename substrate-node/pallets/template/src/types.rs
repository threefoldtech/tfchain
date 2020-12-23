#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use fixed::{types::U64F64};
use frame_support::traits::Vec;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default)]
pub struct Farm {
	pub id: u64,
	pub name: Vec<u8>,
	pub entity_id: u64,
	pub twin_id: u64,
	pub pricing_policy_id: u64,
	pub certification_type: CertificationType,
	pub country_id: u64,
	pub city_id: u64
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default)]
pub struct Node {
	pub id: u64,
	pub farm_id: u64,
	pub twin_id: u64,
	pub resources: Resources,
	pub location: Location,
	pub country_id: u64,
	pub city_id: u64
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default)]
pub struct Entity {
	pub id: u64,
	pub name: Vec<u8>,
	pub country_id: u64,
	pub city_id: u64
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default)]
pub struct Twin {
	pub id: u64,
	pub pubkey: Vec<u8>,
	pub entity_id: u64
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct Resources {
	pub hru: u64,
	pub sru: u64,
	pub cru: u64,
	pub mru: u64,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct Location {
	pub longitude: U64F64,
	pub latitude: U64F64
}

struct PricingPolicy {
	pub id: u64,
	pub name: Vec<u8>,
	pub currency: Vec<u8>,
	pub su: u64,
	pub cu: u64,
	pub nu: u64
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug)]
pub enum CertificationType {
	None,
	Silver,
	Gold
}

impl Default for CertificationType {
    fn default() -> CertificationType {
        CertificationType::None
    }
}
