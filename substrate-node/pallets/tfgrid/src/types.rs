#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    sp_runtime::{
        traits::AccountIdConversion, ModuleId
	},
	traits::{
		Vec,
    },
};

const PALLET_ID: ModuleId = ModuleId(*b"ABCDEFG!");

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct Node {
	pub id: u64,
	pub farm_id: u64,
	pub twin_id: u64,
	pub resources: Resources,
	pub location: Location,
	pub country_id: u64,
	pub city_id: u64
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode)]
pub struct Entity<T: super::Trait> {
	pub entity_id: u64,
	pub name: Vec<u8>,
	pub country_id: u64,
	pub city_id: u64,
	pub pub_key: T::AccountId,
}

impl<T> Default for Entity<T>
    where T: super::Trait
{
    fn default() -> Entity<T> {
		let pub_key: T::AccountId = PALLET_ID.into_account();
		
        Entity { 
			entity_id: 0,
			name: [0].to_vec(),
			country_id: 0,
			city_id: 0,
			pub_key
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode)]
pub struct Twin<T: super::Trait> {
	pub twin_id: u64,
	pub pub_key: T::AccountId,
	pub peer_id: Vec<u8>,
	pub entities: Vec<EntityProof>
}

impl<T> Default for Twin<T>
    where T: super::Trait
{
    fn default() -> Twin<T> {
		let pub_key = PALLET_ID.into_account();

        Twin {
			twin_id: 0,
			pub_key,
			peer_id: Vec::new(),
			entities: Vec::new()
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default)]
pub struct EntityProof {
	pub entity_id: u64,
	pub signature: Vec<u8>
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, Copy)]
pub struct Resources {
	pub hru: u64,
	pub sru: u64,
	pub cru: u64,
	pub mru: u64,
}

// Store Location long and lat as string
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct Location {
	pub longitude: Vec<u8>,
	pub latitude: Vec<u8>
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct PricingPolicy {
	pub id: u64,
	pub name: Vec<u8>,
	pub currency: Vec<u8>,
	pub su: u64,
	pub cu: u64,
	pub nu: u64
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct CertificationCodes {
	pub id: u64,
	pub name: Vec<u8>,
	pub description: Vec<u8>,
	pub certification_code_type: CertificationCodeType
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug)]
pub enum CertificationCodeType {
	Farm,
	Entity
}

impl Default for CertificationCodeType {
    fn default() -> CertificationCodeType {
        CertificationCodeType::Farm
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug, Copy)]
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
