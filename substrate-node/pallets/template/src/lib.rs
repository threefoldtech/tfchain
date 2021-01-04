#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, ensure, dispatch,
	traits::{
		Currency, Get,
	},
};
use frame_system::{
    self as system, ensure_signed,
};
// use pallet_timestamp as timestamp;


use sp_std::prelude::*;

#[cfg(test)]
mod tests;

mod types;

pub type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Currency: Currency<Self::AccountId>;
}

decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		pub Farms get(fn farms): map hasher(blake2_128_concat) u64 => types::Farm; 
		pub FarmsByNameID get(fn farms_by_name_id): map hasher(blake2_128_concat) Vec<u8> => u64;

		pub Nodes get(fn nodes): map hasher(blake2_128_concat) u64 => types::Node;
		
		pub Entities get(fn entities): map hasher(blake2_128_concat) u64 => types::Entity<T>;
		pub EntitiesByNameID get(fn entities_by_name_id): map hasher(blake2_128_concat) Vec<u8> => u64;

		pub Twins get(fn twins): map hasher(blake2_128_concat) u64 => types::Twin;
		pub TwinsByPubkeyID get(fn twins_by_pubkey_id): map hasher(blake2_128_concat) Vec<u8> => u64;

		pub PricingPolicies get(fn pricing_policies): map hasher(blake2_128_concat) u64 => types::PricingPolicy;
		pub PricingPoliciesByNameID get(fn pricing_policies_by_name_id): map hasher(blake2_128_concat) Vec<u8> => u64;

		pub CertificationCodes get(fn certification_codes): map hasher(blake2_128_concat) u64 => types::CertificationCodes;
		pub CertificationCodesByNameID get(fn certification_codes_by_name_id): map hasher(blake2_128_concat) Vec<u8> => u64;

		// ID maps
		FarmID: u64;
		NodeID: u64;
		EntityID: u64;
		TwinID: u64;
		PricingPolicyID: u64;
		CertificationCodeID: u64;
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		SomethingStored(u32, AccountId),
		FarmStored(u64),
		NodeStored(u64),
		EntityStored(u64, Vec<u8>, u64, u64, AccountId),
		TwinStored(Vec<u8>, u64),
		PricingPolicyStored(Vec<u8>, u64),
		CertificationCodeStored(Vec<u8>, u64),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		NoneValue,
		StorageOverflow,

		FarmExists,
		FarmNotExists,

		EntityExists,
		EntityNotExists,
	
		TwinExists,
		TwinNotExists,

		PricingPolicyExists,

		CertificationCodeExists,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn create_farm(origin,
			name: Vec<u8>,
			entity_id: u64,
			twin_id: u64,
			pricing_policy_id: u64,
			certification_type: types::CertificationType,
			country_id: u64,
			city_id: u64) -> dispatch::DispatchResult {
			let _ = ensure_signed(origin)?;

			ensure!(Entities::<T>::contains_key(entity_id), Error::<T>::EntityNotExists);
			ensure!(Twins::contains_key(twin_id), Error::<T>::TwinNotExists);

			ensure!(!FarmsByNameID::contains_key(name.clone()), Error::<T>::FarmExists);

			let id = FarmID::get();

			let farm = types::Farm {
				id,
				name: name.clone(),
				entity_id,
				twin_id,
				pricing_policy_id,
				country_id,
				city_id,
				certification_type
			};

			Farms::insert(id, &farm);
			FarmsByNameID::insert(name, id);
			FarmID::put(id + 1);

			Self::deposit_event(RawEvent::FarmStored(id));

			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn create_node(origin,
			farm_id: u64,
			twin_id: u64,
			resources: types::Resources,
			location: types::Location,
			country_id: u64,
			city_id: u64) -> dispatch::DispatchResult {
			let _ = ensure_signed(origin)?;

			ensure!(Twins::contains_key(twin_id), Error::<T>::TwinNotExists);
			ensure!(Farms::contains_key(&farm_id), Error::<T>::FarmNotExists);

			let id = NodeID::get();

			let node = types::Node {
				id,
				farm_id,
				twin_id,
				resources,
				location,
				country_id,
				city_id
			};

			Nodes::insert(id, &node);
			NodeID::put(id + 1);

			Self::deposit_event(RawEvent::NodeStored(id));

			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn create_entity(origin, name: Vec<u8>, country_id: u64, city_id: u64) -> dispatch::DispatchResult {
			let pub_key = ensure_signed(origin)?;

			ensure!(!EntitiesByNameID::contains_key(&name), Error::<T>::EntityExists);

			let id = EntityID::get();

			let entity = types::Entity::<T> {
				entity_id: id,
				name: name.clone(),
				country_id,
				city_id,
				pub_key: pub_key.clone(), 
			};

			Entities::insert(&id, &entity);
			EntitiesByNameID::insert(&name, id);
			EntityID::put(id + 1);

			Self::deposit_event(RawEvent::EntityStored(id, name, country_id, city_id, pub_key));

			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn create_twin(origin, pubkey: Vec<u8>, entity_id: u64) -> dispatch::DispatchResult {
			let _ = ensure_signed(origin)?;

			ensure!(!TwinsByPubkeyID::contains_key(&pubkey), Error::<T>::TwinExists);

			ensure!(Entities::<T>::contains_key(entity_id), Error::<T>::EntityNotExists);

			let id = TwinID::get();

			let twin = types::Twin {
				id,
				pubkey: pubkey.clone(),
				entity_id,
			};

			Twins::insert(&id, &twin);
			TwinsByPubkeyID::insert(&pubkey, &id);
			TwinID::put(id + 1);

			Self::deposit_event(RawEvent::TwinStored(pubkey, id));
			
			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn create_pricing_policy(origin, name: Vec<u8>, currency: Vec<u8>, su: u64, cu: u64, nu: u64) -> dispatch::DispatchResult {
			let _ = ensure_signed(origin)?;

			ensure!(!PricingPoliciesByNameID::contains_key(&name), Error::<T>::PricingPolicyExists);

			let id = PricingPolicyID::get();

			let policy = types::PricingPolicy {
				id,
				name: name.clone(),
				currency,
				su,
				cu,
				nu
			};

			PricingPolicies::insert(&id, &policy);
			PricingPoliciesByNameID::insert(&name, &id);
			PricingPolicyID::put(id + 1);

			Self::deposit_event(RawEvent::PricingPolicyStored(name, id));
			
			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn create_certification_code(origin, name: Vec<u8>, description: Vec<u8>, certification_code_type: types::CertificationCodeType) -> dispatch::DispatchResult {
			let _ = ensure_signed(origin)?;

			ensure!(!CertificationCodesByNameID::contains_key(&name), Error::<T>::CertificationCodeExists);

			let id = CertificationCodeID::get();

			let certification_code = types::CertificationCodes{
				id,
				name: name.clone(),
				description,
				certification_code_type
			};

			CertificationCodes::insert(&id, &certification_code);
			CertificationCodesByNameID::insert(&name, &id);
			CertificationCodeID::put(id + 1);

			Self::deposit_event(RawEvent::CertificationCodeStored(name, id));

			Ok(())
		}
	}
}