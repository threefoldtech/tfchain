#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, ensure, dispatch, debug,
	traits::{
		Get,
	},
};
use frame_system::{
    self as system, ensure_signed,
    offchain::{
		AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer,
	},
};
use system::{ensure_root, ensure_none};

use codec::{Encode};

use hex::{FromHex};

use sp_std::prelude::*;

use lite_json::json::JsonValue;
use sp_runtime::{
    offchain::{http, Duration},
};

use fixed::{types::U64F64};

use sp_core::crypto::KeyTypeId;
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"tft!");

pub mod crypto {
    use crate::KEY_TYPE;
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify,
        MultiSignature, MultiSigner,
    };

    app_crypto!(sr25519, KEY_TYPE);

    pub struct AuthId;

    // implemented for ocw-runtime
    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for AuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }

    // implemented for mock runtime in test
    impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
    for AuthId
    {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
}

#[cfg(test)]
mod tests;

mod types;

pub trait Trait: system::Trait + CreateSignedTransaction<Call<Self>> {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	// Add other types and constants required to configure this pallet.
    type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
    type Call: From<Call<Self>>;
}

decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		pub Farms get(fn farms): map hasher(blake2_128_concat) u64 => types::Farm; 
		pub FarmsByNameID get(fn farms_by_name_id): map hasher(blake2_128_concat) Vec<u8> => u64;

		pub Nodes get(fn nodes): map hasher(blake2_128_concat) u64 => types::Node;
		
		pub Entities get(fn entities): map hasher(blake2_128_concat) u64 => types::Entity<T>;
		pub EntitiesByPubkeyID get(fn entities_by_pubkey_id): map hasher(blake2_128_concat) T::AccountId => u64;
		pub EntitiesByNameID get(fn entities_by_name_id): map hasher(blake2_128_concat) Vec<u8> => u64;

		pub Twins get(fn twins): map hasher(blake2_128_concat) u64 => types::Twin<T>;
		pub TwinsByPubkey get(fn twin_ids_by_pubkey): map hasher(blake2_128_concat) T::AccountId => Vec<u64>;

		pub PricingPolicies get(fn pricing_policies): map hasher(blake2_128_concat) u64 => types::PricingPolicy;
		pub PricingPoliciesByNameID get(fn pricing_policies_by_name_id): map hasher(blake2_128_concat) Vec<u8> => u64;

		pub CertificationCodes get(fn certification_codes): map hasher(blake2_128_concat) u64 => types::CertificationCodes;
		pub CertificationCodesByNameID get(fn certification_codes_by_name_id): map hasher(blake2_128_concat) Vec<u8> => u64;

		// Token price
		TFTPrice: U64F64;

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
		FarmStored(u64, Vec<u8>, u64, u64, u64, u64, u64, types::CertificationType),
		FarmDeleted(u64),

		NodeStored(u64, u64, u64, types::Resources, types::Location, u64, u64),
		NodeDeleted(u64),

		EntityStored(u64, Vec<u8>, u64, u64, AccountId),
		EntityUpdated(u64, Vec<u8>, u64, u64, AccountId),
		EntityDeleted(u64),

		TwinStored(AccountId, u64, Vec<u8>),
		TwinUpdated(u64, Vec<u8>),

		TwinEntityStored(u64, u64, Vec<u8>),
		TwinEntityRemoved(u64, u64),
		TwinDeleted(u64),

		PricingPolicyStored(Vec<u8>, u64),
		CertificationCodeStored(Vec<u8>, u64),

		NewPrice(U64F64),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		NoneValue,
		StorageOverflow,

		CannotCreateNode,
		NodeNotExists,
		CannotDeleteNode,

		FarmExists,
		FarmNotExists,
		CannotDeleteFarm,

		EntityWithNameExists,
		EntityWithPubkeyExists,
		EntityNotExists,
		EntitySignatureDoesNotMatch,
		EntityWithSignatureAlreadyExists,
		CannotUpdateEntity,
		CannotDeleteEntity,
	
		TwinExists,
		TwinNotExists,
		CannotCreateTwin,
		UnauthorizedToUpdateTwin,

		PricingPolicyExists,

		CertificationCodeExists,

		OffchainSignedTxError,
		NoLocalAcctForSigning
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn create_farm(origin, farm: types::Farm) -> dispatch::DispatchResult {
			let _ = ensure_signed(origin)?;

			ensure!(Entities::<T>::contains_key(farm.entity_id), Error::<T>::EntityNotExists);
			ensure!(Twins::<T>::contains_key(farm.twin_id), Error::<T>::TwinNotExists);

			ensure!(!FarmsByNameID::contains_key(farm.name.clone()), Error::<T>::FarmExists);

			let id = FarmID::get();

			let mut new_farm = farm.clone();

			new_farm.id = id;

			Farms::insert(id, &new_farm);
			FarmsByNameID::insert(new_farm.name.clone(), id);
			FarmID::put(id + 1);

			Self::deposit_event(RawEvent::FarmStored(
				id, 
				new_farm.name, 
				new_farm.entity_id, 
				new_farm.twin_id, 
				new_farm.pricing_policy_id, 
				new_farm.country_id, 
				new_farm.city_id, 
				new_farm.certification_type
			));

			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn delete_farm(origin, id: u64) -> dispatch::DispatchResult {
			let pub_key = ensure_signed(origin)?;

			ensure!(Farms::contains_key(id), Error::<T>::FarmNotExists);
			let stored_farm = Farms::get(id);

			ensure!(Entities::<T>::contains_key(stored_farm.entity_id), Error::<T>::EntityNotExists);
			let stored_entity = Entities::<T>::get(stored_farm.entity_id);

			ensure!(stored_entity.pub_key == pub_key, Error::<T>::CannotDeleteFarm);

			// delete farm
			Farms::remove(id);

			// Remove stored farm by name and insert new one
			FarmsByNameID::remove(stored_farm.name);

			Self::deposit_event(RawEvent::FarmDeleted(id));

			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn create_node(origin, node: types::Node) -> dispatch::DispatchResult {
			let pub_key = ensure_signed(origin)?;

			ensure!(Twins::<T>::contains_key(node.twin_id), Error::<T>::TwinNotExists);
			ensure!(Farms::contains_key(node.farm_id), Error::<T>::FarmNotExists);

			let stored_twin = Twins::<T>::get(node.twin_id);
			ensure!(stored_twin.pub_key == pub_key, Error::<T>::CannotCreateNode);

			let stored_farm = Farms::get(node.farm_id);
			ensure!(stored_farm.twin_id == node.twin_id, Error::<T>::CannotCreateNode);

			let id = NodeID::get();

			let mut new_node = node.clone();
			new_node.id = id;

			Nodes::insert(id, &new_node);
			NodeID::put(id + 1);

			Self::deposit_event(RawEvent::NodeStored(
				id, 
				new_node.farm_id, 
				new_node.twin_id, 
				new_node.resources, 
				new_node.location, 
				new_node.country_id, 
				new_node.city_id
			));

			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn delete_node(origin, id: u64) -> dispatch::DispatchResult {
			let pub_key = ensure_signed(origin)?;

			ensure!(Nodes::contains_key(id), Error::<T>::NodeNotExists);

			let stored_node = Nodes::get(id);

			// check if the user can delete this node based on the twin id
			ensure!(Twins::<T>::contains_key(stored_node.twin_id), Error::<T>::TwinNotExists);
			let stored_twin = Twins::<T>::get(stored_node.twin_id);
			ensure!(stored_twin.pub_key == pub_key, Error::<T>::CannotDeleteNode);

			Nodes::remove(id);

			Self::deposit_event(RawEvent::NodeDeleted(id));

			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn create_entity(origin, name: Vec<u8>, country_id: u64, city_id: u64) -> dispatch::DispatchResult {
			let pub_key = ensure_signed(origin)?;

			ensure!(!EntitiesByNameID::contains_key(&name), Error::<T>::EntityWithNameExists);
			
			ensure!(!EntitiesByPubkeyID::<T>::contains_key(&pub_key), Error::<T>::EntityWithPubkeyExists);

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
			EntitiesByPubkeyID::<T>::insert(&pub_key, id);
			EntityID::put(id + 1);

			Self::deposit_event(RawEvent::EntityStored(id, name, country_id, city_id, pub_key));

			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn update_entity(origin, name: Vec<u8>, country_id: u64, city_id: u64) -> dispatch::DispatchResult {
			let pub_key = ensure_signed(origin)?;

			ensure!(EntitiesByPubkeyID::<T>::contains_key(&pub_key), Error::<T>::EntityNotExists);
			let stored_entity_id = EntitiesByPubkeyID::<T>::get(&pub_key);

			ensure!(Entities::<T>::contains_key(&stored_entity_id), Error::<T>::EntityNotExists);
			let stored_entity = Entities::<T>::get(stored_entity_id);

			ensure!(stored_entity.pub_key == pub_key, Error::<T>::CannotUpdateEntity);

			let entity = types::Entity::<T> {
				entity_id: stored_entity_id,
				name: name.clone(),
				country_id,
				city_id,
				pub_key: pub_key.clone(), 
			};

			// overwrite entity
			Entities::insert(&stored_entity_id, &entity);
			
			// remove entity by name id
			EntitiesByNameID::remove(&stored_entity.name);
			// re-insert with new name
			EntitiesByNameID::insert(&name, stored_entity_id);

			Self::deposit_event(RawEvent::EntityUpdated(stored_entity_id, name, country_id, city_id, pub_key));

			Ok(())
		}

		// TODO: delete all object that have an entity id reference?
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn delete_entity(origin) -> dispatch::DispatchResult {
			let pub_key = ensure_signed(origin)?;

			ensure!(EntitiesByPubkeyID::<T>::contains_key(&pub_key), Error::<T>::EntityNotExists);
			let stored_entity_id = EntitiesByPubkeyID::<T>::get(&pub_key);

			ensure!(Entities::<T>::contains_key(&stored_entity_id), Error::<T>::EntityNotExists);
			let stored_entity = Entities::<T>::get(stored_entity_id);

			ensure!(stored_entity.pub_key == pub_key, Error::<T>::CannotDeleteEntity);

			// Remove entity from storage
			Entities::<T>::remove(&stored_entity_id);
			
			// remove entity by name id
			EntitiesByNameID::remove(&stored_entity.name);

			// remove entity by pubkey id
			EntitiesByPubkeyID::<T>::remove(&pub_key);

			Self::deposit_event(RawEvent::EntityDeleted(stored_entity_id));

			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn create_twin(origin, peer_id: Vec<u8>) -> dispatch::DispatchResult {
			let pub_key = ensure_signed(origin)?;

			let twin_id = TwinID::get();

			let twin = types::Twin::<T> {
				twin_id,
				pub_key: pub_key.clone(),
				entities: Vec::new(),
				peer_id: peer_id.clone()
			};

			Twins::insert(&twin_id, &twin);
			TwinID::put(twin_id + 1);

			// add the twin id to this users map of twin ids
			let mut twins_by_pubkey = TwinsByPubkey::<T>::get(&pub_key.clone());
			twins_by_pubkey.push(twin_id);
			TwinsByPubkey::<T>::insert(&pub_key.clone(), twins_by_pubkey);

			Self::deposit_event(RawEvent::TwinStored(pub_key, twin_id, peer_id));
			
			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn update_twin(origin, twin_id: u64, peer_id: Vec<u8>) -> dispatch::DispatchResult {
			let pub_key = ensure_signed(origin)?;

			ensure!(Twins::<T>::contains_key(&twin_id), Error::<T>::TwinNotExists);

			let twin = Twins::<T>::get(&twin_id);
			// Make sure only the owner of this twin can update his twin
			ensure!(twin.pub_key == pub_key, Error::<T>::UnauthorizedToUpdateTwin);

			let updated_twin = types::Twin::<T> {
				twin_id,
				pub_key: twin.pub_key,
				entities: twin.entities,
				peer_id: peer_id.clone()
			};

			Twins::insert(&twin_id, &updated_twin);

			Self::deposit_event(RawEvent::TwinUpdated(twin_id, peer_id));
			
			Ok(())
		}

		// Method for twins only
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn add_twin_entity(origin, twin_id: u64, entity_id: u64, signature: Vec<u8>) -> dispatch::DispatchResult {
			let pub_key = ensure_signed(origin)?;

			ensure!(Twins::<T>::contains_key(&twin_id), Error::<T>::TwinNotExists);

			ensure!(Entities::<T>::contains_key(&entity_id), Error::<T>::EntityNotExists);
			let stored_entity = Entities::<T>::get(entity_id);

			let mut twin = Twins::<T>::get(&twin_id);
			// Make sure only the owner of this twin can call this method
			ensure!(twin.pub_key == pub_key, Error::<T>::UnauthorizedToUpdateTwin);

			let entity_proof = types::EntityProof{
				entity_id,
				signature: signature.clone()
			};

			ensure!(!twin.entities.contains(&entity_proof), Error::<T>::EntityWithSignatureAlreadyExists);

			let decoded_signature_as_byteslice = <[u8; 64]>::from_hex(signature.clone()).expect("Decoding failed");

			// Decode signature into a ed25519 signature
			let ed25519_signature = sp_core::ed25519::Signature::from_raw(decoded_signature_as_byteslice);
			// let sr25519_signature = sp_core::sr25519::Signature::from_raw(decoded_signature_as_byteslice);

			// Decode entity's public key
			let account_vec = &stored_entity.pub_key.encode();
			ensure!(account_vec.len() == 32, "AccountId must be 32 bytes.");
			let mut bytes = [0u8; 32];
			bytes.copy_from_slice(&account_vec);

			let entity_pubkey_ed25519 = sp_core::ed25519::Public::from_raw(bytes);
			debug::info!("Public key: {:?}", entity_pubkey_ed25519);

			// let entity_pubkey_sr25519 = sp_core::sr25519::Public::from_raw(bytes);
			// debug::info!("Public key: {:?}", entity_pubkey_sr25519);

			let mut message = vec![];

			message.extend_from_slice(&entity_id.to_be_bytes()); 
			message.extend_from_slice(&twin_id.to_be_bytes()); 

			debug::info!("Message: {:?}", message);
			
			// Verify that the signature contains the message with the entity's public key
			debug::info!("Checking signature");
			let ed25519_verified = sp_io::crypto::ed25519_verify(&ed25519_signature, &message, &entity_pubkey_ed25519);
			debug::info!("ed25519 verified? {:?}", ed25519_verified);
			
			// let sr25519_verified = sp_io::crypto::sr25519_verify(&sr25519_signature, &message, &entity_pubkey_sr25519);
			// let sr25519_verified = sr25519_signature.verify(message.as_slice(), &entity_pubkey_sr25519);
			// debug::info!("sr25519 verified? {:?}", sr25519_verified);

			ensure!(sp_io::crypto::ed25519_verify(&ed25519_signature, &message, &entity_pubkey_ed25519), Error::<T>::EntitySignatureDoesNotMatch);
			
			debug::info!("Signature is valid");

			// Store proof
			twin.entities.push(entity_proof);

			// Update twin
			Twins::insert(&twin_id, &twin);

			Self::deposit_event(RawEvent::TwinEntityStored(twin_id, entity_id, signature));

			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn delete_twin_entity(origin, twin_id: u64, entity_id: u64) -> dispatch::DispatchResult {
			let pub_key = ensure_signed(origin)?;

			ensure!(Twins::<T>::contains_key(&twin_id), Error::<T>::TwinNotExists);

			let mut twin = Twins::<T>::get(&twin_id);
			// Make sure only the owner of this twin can call this method
			ensure!(twin.pub_key == pub_key, Error::<T>::UnauthorizedToUpdateTwin);

			ensure!(twin.entities.iter().any(|v| v.entity_id == entity_id), Error::<T>::EntityNotExists);

			let index = twin.entities.iter().position(|x| x.entity_id == entity_id).unwrap();
			twin.entities.remove(index);

			// Update twin
			Twins::insert(&twin_id, &twin);

			Self::deposit_event(RawEvent::TwinEntityRemoved(twin_id, entity_id));

			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn delete_twin(origin, twin_id: u64) -> dispatch::DispatchResult {
			let pub_key = ensure_signed(origin)?;

			ensure!(Twins::<T>::contains_key(&twin_id), Error::<T>::TwinNotExists);

			let twin = Twins::<T>::get(&twin_id);
			// Make sure only the owner of this twin can call this method
			ensure!(twin.pub_key == pub_key, Error::<T>::UnauthorizedToUpdateTwin);

			Twins::<T>::remove(&twin_id);

			// remove twin id from this users map of twin ids
			let mut twins_by_pubkey = TwinsByPubkey::<T>::get(&pub_key.clone());
			if let Some(pos) = twins_by_pubkey.iter().position(|x| *x == twin_id) {
				twins_by_pubkey.remove(pos);
				TwinsByPubkey::<T>::insert(&pub_key.clone(), twins_by_pubkey);
			}
			
			Self::deposit_event(RawEvent::TwinDeleted(twin_id));

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

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn set_prices(origin, price: U64F64){
			ensure_root(origin)?;
			TFTPrice::put(price);
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn submit_price(origin, price: U64F64) {
			ensure_none(origin)?;
			TFTPrice::put(price);
		}

		fn offchain_worker(block_number: T::BlockNumber) {
			debug::debug!("hello from worker");
			match Self::offchain_signed_tx() {
				Ok(_) => debug::debug!("worker executed"),
				Err(err) => debug::debug!("err: {:?}", err)
			}
        }
	}
}

impl<T: Trait> Module<T> {
    /// Fetch current price and return the result in cents.
    fn fetch_price() -> Result<f64, http::Error> {
        let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));
 
        let request =
            http::Request::get("https://min-api.cryptocompare.com/data/price?fsym=TFT&tsyms=USD");

        let pending = request
            .deadline(deadline)
            .send()
            .map_err(|_| http::Error::IoError)?;

        let response = pending
            .try_wait(deadline)
            .map_err(|_| http::Error::DeadlineReached)??;
        // Let's check the status code before we proceed to reading the response.
        if response.code != 200 {
            debug::warn!("Unexpected status code: {}", response.code);
            return Err(http::Error::Unknown);
        }

        let body = response.body().collect::<Vec<u8>>();

        // Create a str slice from the body.
        let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
            debug::warn!("No UTF8 body");
            http::Error::Unknown
        })?;

        let price = match Self::parse_price(body_str) {
            Some(price) => Ok(price),
            None => {
                debug::warn!("Unable to extract price from the response: {:?}", body_str);
                Err(http::Error::Unknown)
            }
        }?;

        Ok(price)
	}
	
	fn parse_price(price_str: &str) -> Option<f64> {
        let val = lite_json::parse_json(price_str);
        let price = val.ok().and_then(|v| match v {
            JsonValue::Object(obj) => {
                let mut chars = "USD".chars();
                obj.into_iter()
                    .find(|(k, _)| k.iter().all(|k| Some(*k) == chars.next()))
                    .and_then(|v| match v.1 {
                        JsonValue::Number(number) => Some(number),
                        _ => None,
                    })
            }
            _ => None,
        })?;

		let exp = price.fraction_length.checked_sub(2).unwrap_or(0);
		
		let price = 
			(price.integer * 10_i64.pow(exp as u32) + price.fraction as i64) as f64
			/ (100 * 10_u32.pow(exp as u32)) as f64;
        Some(price)
	}
	
	fn offchain_signed_tx() -> Result<(), Error<T>> {
		let price = match Self::fetch_price() {
			Ok(v) => v,
			Err(_) => return Err(<Error<T>>::OffchainSignedTxError)
		};

        // retrieve contract account
        // let signer = Signer::<T, T::AuthorityId>::any_account();

		let price_to_fixed = U64F64::from_num(price);

        let signer = Signer::<T, T::AuthorityId>::any_account();

        let result = signer.send_signed_transaction(|_acct| {
            Call::set_prices(price_to_fixed)
        });
    
        // Display error if the signed tx fails.
        // Display error if the signed tx fails.
        if let Some((acc, res)) = result {
            if res.is_err() {
                debug::error!("failure: offchain_signed_tx: tx sent: {:?}", acc.id);
                return Err(<Error<T>>::OffchainSignedTxError);
            }
            // Transaction is sent successfully
            return Ok(());
        }
        // The case of `None`: no account is available for sending
        debug::error!("No local account available");
        return Err(<Error<T>>::OffchainSignedTxError)
    }
}
