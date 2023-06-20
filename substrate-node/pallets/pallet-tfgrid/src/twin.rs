use crate::*;
use frame_support::{
    dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo},
    ensure,
};
use hex::FromHex;
use parity_scale_codec::Encode;
use sp_runtime::SaturatedConversion;
use sp_std::{vec, vec::Vec};

impl<T: Config> Pallet<T> {
    pub fn _user_accept_tc(
        account_id: T::AccountId,
        document_link: DocumentLinkInput,
        document_hash: DocumentHashInput,
    ) -> DispatchResultWithPostInfo {
        let timestamp = <pallet_timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;

        let input = TermsAndConditionsInput::<T> {
            account_id: account_id.clone(),
            timestamp,
            document_link,
            document_hash,
        };

        let t_and_c = Self::get_terms_and_conditions(input)?;

        let mut users_terms_and_condition =
            UsersTermsAndConditions::<T>::get(account_id.clone()).unwrap_or(vec![]);
        users_terms_and_condition.push(t_and_c);
        UsersTermsAndConditions::<T>::insert(account_id, users_terms_and_condition);

        Ok(().into())
    }

    pub fn _create_entity(
        target: T::AccountId,
        name: Vec<u8>,
        country: CountryNameInput,
        city: CityNameInput,
        signature: Vec<u8>,
    ) -> DispatchResultWithPostInfo {
        ensure!(
            !EntityIdByName::<T>::contains_key(&name),
            Error::<T>::EntityWithNameExists
        );
        ensure!(
            !EntityIdByAccountID::<T>::contains_key(&target),
            Error::<T>::EntityWithPubkeyExists
        );
        ensure!(
            signature.len() == 128,
            Error::<T>::SignatureLengthIsIncorrect
        );
        let decoded_signature_as_byteslice =
            <[u8; 64]>::from_hex(signature.clone()).expect("Decoding failed");
        let mut message = Vec::new();
        message.extend_from_slice(&name);
        message.extend_from_slice(&country);
        message.extend_from_slice(&city);

        ensure!(
            Self::verify_signature(decoded_signature_as_byteslice, &target, &message),
            Error::<T>::EntitySignatureDoesNotMatch
        );

        let mut id = EntityID::<T>::get();
        id = id + 1;

        let entity = TfgridEntity::<T> {
            version: TFGRID_ENTITY_VERSION,
            id,
            name: name.clone(),
            country: Self::get_country_name(country)?,
            city: Self::get_city_name(city)?,
            account_id: target.clone(),
        };

        Entities::<T>::insert(&id, &entity);
        EntityIdByName::<T>::insert(&name, id);
        EntityIdByAccountID::<T>::insert(&target, id);
        EntityID::<T>::put(id);

        Self::deposit_event(Event::EntityStored(entity));

        Ok(().into())
    }

    pub fn _update_entity(
        account_id: T::AccountId,
        name: Vec<u8>,
        country: CountryNameInput,
        city: CityNameInput,
    ) -> DispatchResultWithPostInfo {
        ensure!(
            !EntityIdByName::<T>::contains_key(&name),
            Error::<T>::EntityWithNameExists
        );

        let stored_entity_id =
            EntityIdByAccountID::<T>::get(&account_id).ok_or(Error::<T>::EntityNotExists)?;

        ensure!(
            Entities::<T>::contains_key(&stored_entity_id),
            Error::<T>::EntityNotExists
        );
        let mut stored_entity =
            Entities::<T>::get(stored_entity_id).ok_or(Error::<T>::EntityNotExists)?;

        ensure!(
            stored_entity.account_id == account_id,
            Error::<T>::CannotUpdateEntity
        );

        // remove entity by name id
        EntityIdByName::<T>::remove(&stored_entity.name);

        stored_entity.name = name.clone();
        stored_entity.country = Self::get_country_name(country)?;
        stored_entity.city = Self::get_city_name(city)?;

        // overwrite entity
        Entities::<T>::insert(&stored_entity_id, &stored_entity);

        // re-insert with new name
        EntityIdByName::<T>::insert(&name, stored_entity_id);

        Self::deposit_event(Event::EntityUpdated(stored_entity));

        Ok(().into())
    }

    pub fn _delete_entity(account_id: T::AccountId) -> DispatchResultWithPostInfo {
        let stored_entity_id =
            EntityIdByAccountID::<T>::get(&account_id).ok_or(Error::<T>::EntityNotExists)?;

        let stored_entity =
            Entities::<T>::get(stored_entity_id).ok_or(Error::<T>::EntityNotExists)?;

        ensure!(
            stored_entity.account_id == account_id,
            Error::<T>::CannotDeleteEntity
        );

        // Remove entity from storage
        Entities::<T>::remove(&stored_entity_id);

        // remove entity by name id
        EntityIdByName::<T>::remove(&stored_entity.name);

        // remove entity by pubkey id
        EntityIdByAccountID::<T>::remove(&account_id);

        Self::deposit_event(Event::EntityDeleted(stored_entity_id));

        Ok(().into())
    }

    pub fn _create_twin(
        account_id: T::AccountId,
        relay: RelayInput,
        pk: PkInput,
    ) -> DispatchResultWithPostInfo {
        ensure!(
            UsersTermsAndConditions::<T>::contains_key(account_id.clone()),
            Error::<T>::UserDidNotSignTermsAndConditions
        );

        ensure!(
            !TwinIdByAccountID::<T>::contains_key(&account_id),
            Error::<T>::TwinWithPubkeyExists
        );

        let mut twin_id = TwinID::<T>::get();
        twin_id = twin_id + 1;

        if let Some(relay_addr) = relay.clone() {
            ensure!(
                Self::validate_relay_address(relay_addr.into()),
                Error::<T>::InvalidRelayAddress
            );
        }

        let twin = types::Twin::<T::AccountId> {
            id: twin_id,
            account_id: account_id.clone(),
            relay,
            entities: Vec::new(),
            pk,
        };

        Twins::<T>::insert(&twin_id, &twin);
        TwinID::<T>::put(twin_id);

        // add the twin id to this users map of twin ids
        TwinIdByAccountID::<T>::insert(&account_id.clone(), twin_id);

        Self::deposit_event(Event::TwinStored(twin));

        Ok(().into())
    }

    pub fn _update_twin(
        account_id: T::AccountId,
        relay: RelayInput,
        pk: PkInput,
    ) -> DispatchResultWithPostInfo {
        let twin_id =
            TwinIdByAccountID::<T>::get(account_id.clone()).ok_or(Error::<T>::TwinNotExists)?;

        let mut twin = Twins::<T>::get(&twin_id).ok_or(Error::<T>::TwinNotExists)?;

        // Make sure only the owner of this twin can update his twin
        ensure!(
            twin.account_id == account_id,
            Error::<T>::UnauthorizedToUpdateTwin
        );

        if let Some(relay_addr) = relay.clone() {
            ensure!(
                Self::validate_relay_address(relay_addr.into()),
                Error::<T>::InvalidRelayAddress
            );
        }

        twin.relay = relay;
        twin.pk = pk;

        Twins::<T>::insert(&twin_id, &twin);

        Self::deposit_event(Event::TwinUpdated(twin));
        Ok(().into())
    }

    // Method for twins only
    pub fn _add_twin_entity(
        account_id: T::AccountId,
        twin_id: u32,
        entity_id: u32,
        signature: Vec<u8>,
    ) -> DispatchResultWithPostInfo {
        let stored_entity = Entities::<T>::get(entity_id).ok_or(Error::<T>::EntityNotExists)?;

        let mut twin = Twins::<T>::get(&twin_id).ok_or(Error::<T>::TwinNotExists)?;
        // Make sure only the owner of this twin can call this method
        ensure!(
            twin.account_id == account_id,
            Error::<T>::UnauthorizedToUpdateTwin
        );

        let entity_proof = types::EntityProof {
            entity_id,
            signature: signature.clone(),
        };

        ensure!(
            !twin.entities.contains(&entity_proof),
            Error::<T>::EntityWithSignatureAlreadyExists
        );

        let decoded_signature_as_byteslice =
            <[u8; 64]>::from_hex(signature.clone()).expect("Decoding failed");

        let mut message = Vec::new();
        message.extend_from_slice(&entity_id.to_be_bytes());
        message.extend_from_slice(&twin_id.to_be_bytes());

        ensure!(
            Self::verify_signature(
                decoded_signature_as_byteslice,
                &stored_entity.account_id,
                &message
            ),
            Error::<T>::EntitySignatureDoesNotMatch
        );

        // Store proof
        twin.entities.push(entity_proof);

        // Update twin
        Twins::<T>::insert(&twin_id, &twin);

        Self::deposit_event(Event::TwinEntityStored(twin_id, entity_id, signature));

        Ok(().into())
    }

    pub fn _delete_twin_entity(
        account_id: T::AccountId,
        twin_id: u32,
        entity_id: u32,
    ) -> DispatchResultWithPostInfo {
        let mut twin = Twins::<T>::get(&twin_id).ok_or(Error::<T>::TwinNotExists)?;
        // Make sure only the owner of this twin can call this method
        ensure!(
            twin.account_id == account_id,
            Error::<T>::UnauthorizedToUpdateTwin
        );

        ensure!(
            twin.entities.iter().any(|v| v.entity_id == entity_id),
            Error::<T>::EntityNotExists
        );

        let index = twin
            .entities
            .iter()
            .position(|x| x.entity_id == entity_id)
            .unwrap();
        twin.entities.remove(index);

        // Update twin
        Twins::<T>::insert(&twin_id, &twin);

        Self::deposit_event(Event::TwinEntityRemoved(twin_id, entity_id));

        Ok(().into())
    }

    pub fn _bond_twin_account(
        account_id: T::AccountId,
        twin_id: u32,
    ) -> DispatchResultWithPostInfo {
        let twin = Twins::<T>::get(twin_id).ok_or(Error::<T>::TwinNotExists)?;
        ensure!(
            twin.account_id != account_id,
            Error::<T>::TwinCannotBoundToItself
        );

        TwinBoundedAccountID::<T>::insert(twin_id, &account_id);
        Self::deposit_event(Event::TwinAccountBounded(twin_id, account_id));

        Ok(().into())
    }

    fn get_terms_and_conditions(
        terms_cond: TermsAndConditionsInput<T>,
    ) -> Result<TermsAndConditionsOf<T>, DispatchErrorWithPostInfo> {
        let parsed_terms_cond = <T as Config>::TermsAndConditions::try_from(terms_cond)?;
        Ok(parsed_terms_cond)
    }

    fn verify_signature(signature: [u8; 64], target: &T::AccountId, payload: &Vec<u8>) -> bool {
        Self::verify_ed_signature(signature, target, payload)
            || Self::verify_sr_signature(signature, target, payload)
    }

    fn verify_ed_signature(signature: [u8; 64], target: &T::AccountId, payload: &Vec<u8>) -> bool {
        let entity_pubkey_ed25519 = Self::convert_account_to_ed25519(target);
        // Decode signature into a ed25519 signature
        let ed25519_signature = sp_core::ed25519::Signature::from_raw(signature);

        sp_io::crypto::ed25519_verify(&ed25519_signature, &payload, &entity_pubkey_ed25519)
    }

    fn verify_sr_signature(signature: [u8; 64], target: &T::AccountId, payload: &Vec<u8>) -> bool {
        let entity_pubkey_sr25519 = Self::convert_account_to_sr25519(target);
        // Decode signature into a sr25519 signature
        let sr25519_signature = sp_core::sr25519::Signature::from_raw(signature);

        sp_io::crypto::sr25519_verify(&sr25519_signature, &payload, &entity_pubkey_sr25519)
    }

    fn convert_account_to_ed25519(account: &T::AccountId) -> sp_core::ed25519::Public {
        // Decode entity's public key
        let account_vec = &account.encode();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&account_vec);
        sp_core::ed25519::Public::from_raw(bytes)
    }

    fn convert_account_to_sr25519(account: &T::AccountId) -> sp_core::sr25519::Public {
        // Decode entity's public key
        let account_vec = &account.encode();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&account_vec);
        sp_core::sr25519::Public::from_raw(bytes)
    }

    fn get_country_name(
        country: CountryNameInput,
    ) -> Result<CountryNameOf<T>, DispatchErrorWithPostInfo> {
        let parsed_country = <T as Config>::CountryName::try_from(country)?;
        Ok(parsed_country)
    }

    fn get_city_name(city: CityNameInput) -> Result<CityNameOf<T>, DispatchErrorWithPostInfo> {
        let parsed_city = <T as Config>::CityName::try_from(city)?;
        Ok(parsed_city)
    }

    fn validate_relay_address(relay_input: Vec<u8>) -> bool {
        if relay_input.len() == 0 {
            return false;
        }

        if relay_input[relay_input.len() - 1] == b'.' {
            return false;
        }

        let mut prev_idx = 0;

        for (idx, c) in relay_input.iter().enumerate() {
            match c {
                b'.' => {
                    if idx == 0 || idx - prev_idx == 1 {
                        return false;
                    } else {
                        prev_idx = idx
                    }
                }
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' => (),
                _ => return false,
            }
        }

        true
    }
}
