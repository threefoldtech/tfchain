use crate::pallet::{Config, Error, Event, Pallet, Validator};
use crate::{types, Bonded};
use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
use sp_runtime::traits::StaticLookup;
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> {
    pub fn _create_validator_request(
        address: T::AccountId,
        validator_node_account: T::AccountId,
        stash_account: T::AccountId,
        description: Vec<u8>,
        tf_connect_id: Vec<u8>,
        info: Vec<u8>,
    ) -> DispatchResultWithPostInfo {
        // Request should not be a duplicate
        ensure!(
            !<Validator<T>>::contains_key(&address),
            Error::<T>::DuplicateValidator
        );

        let request = types::Validator {
            validator_node_account: validator_node_account.clone(),
            stash_account,
            description,
            tf_connect_id,
            info,
            state: types::ValidatorRequestState::Created,
        };

        // Create a validator request object
        <Validator<T>>::insert(&address, &request);

        Self::deposit_event(Event::ValidatorRequestCreated(address, request.clone()));

        Ok(().into())
    }

    pub fn _activate_validator_node(address: T::AccountId) -> DispatchResultWithPostInfo {
        let mut validator = <Validator<T>>::get(&address)
            .ok_or(DispatchError::from(Error::<T>::ValidatorNotFound))?;

        ensure!(
            validator.state != types::ValidatorRequestState::Validating,
            Error::<T>::ValidatorValidatingAlready
        );
        ensure!(
            validator.state == types::ValidatorRequestState::Approved,
            Error::<T>::ValidatorNotApproved
        );

        // Update the validator request
        validator.state = types::ValidatorRequestState::Validating;
        <Validator<T>>::insert(address, &validator);

        // Add the validator and rotate
        substrate_validator_set::Pallet::<T>::add_validator(
            frame_system::RawOrigin::Root.into(),
            validator.validator_node_account.clone(),
        )?;

        Self::deposit_event(Event::ValidatorActivated(validator));

        Ok(().into())
    }

    pub fn _change_validator_node_account(
        address: T::AccountId,
        new_node_validator_account: T::AccountId,
    ) -> DispatchResultWithPostInfo {
        let mut validator = <Validator<T>>::get(&address)
            .ok_or(DispatchError::from(Error::<T>::ValidatorNotFound))?;

        // if validator is validating, also remove old one from consensus and add new one.
        if validator.state == types::ValidatorRequestState::Validating {
            // Remove the old validator and rotate session
            substrate_validator_set::Pallet::<T>::remove_validator(
                frame_system::RawOrigin::Root.into(),
                validator.validator_node_account.clone(),
            )?;
            Self::deposit_event(Event::NodeValidatorRemoved(
                validator.validator_node_account.clone(),
            ));

            // Add the new validator and rotate session
            substrate_validator_set::Pallet::<T>::add_validator(
                frame_system::RawOrigin::Root.into(),
                new_node_validator_account.clone(),
            )?;
            Self::deposit_event(Event::NodeValidatorChanged(
                new_node_validator_account.clone(),
            ));
        }

        // Set the new validator node account on the validator struct
        validator.validator_node_account = new_node_validator_account;
        <Validator<T>>::insert(address, &validator);
        Ok(().into())
    }

    pub fn _bond(
        stash: T::AccountId,
        validator: <T::Lookup as StaticLookup>::Source,
    ) -> DispatchResultWithPostInfo {
        if <Bonded<T>>::contains_key(&stash) {
            Err(Error::<T>::AlreadyBonded)?
        }
        let validator = T::Lookup::lookup(validator)?;
        // TOCHECK: enough to identify validator?

        <Bonded<T>>::insert(&stash, &validator);

        Self::deposit_event(Event::Bonded(stash.clone()));

        Ok(().into())
    }

    pub fn _approve_validator(
        who: T::AccountId,
        validator_account: <T::Lookup as StaticLookup>::Source,
    ) -> DispatchResultWithPostInfo {
        Self::is_council_member(&who)?;

        let v = T::Lookup::lookup(validator_account.clone())?;
        let mut validator =
            <Validator<T>>::get(&v).ok_or(DispatchError::from(Error::<T>::ValidatorNotFound))?;

        // Set state to approved
        validator.state = types::ValidatorRequestState::Approved;
        <Validator<T>>::insert(v.clone(), &validator);

        // Add the validator as a council member
        pallet_membership::Pallet::<T, pallet_membership::Instance1>::add_member(
            frame_system::RawOrigin::Root.into(),
            validator_account,
        )?;

        Self::deposit_event(Event::ValidatorRequestApproved(validator));

        Ok(().into())
    }

    pub fn _remove_validator(
        who: T::AccountId,
        validator_account: <T::Lookup as StaticLookup>::Source,
    ) -> DispatchResultWithPostInfo {
        let v = T::Lookup::lookup(validator_account.clone())?;

        if !(v == who || Self::is_council_member(&who).is_ok()) {
            Err(Error::<T>::BadOrigin)?
        }

        let _ =
            <Validator<T>>::get(&v).ok_or(DispatchError::from(Error::<T>::ValidatorNotFound))?;

        // Remove the validator as a council member
        pallet_membership::Pallet::<T, pallet_membership::Instance1>::remove_member(
            frame_system::RawOrigin::Root.into(),
            validator_account,
        )?;

        // Remove the entry from the storage map
        <Validator<T>>::remove(v);

        Ok(().into())
    }

    pub fn is_council_member(who: &T::AccountId) -> DispatchResultWithPostInfo {
        let council_members =
            pallet_membership::Pallet::<T, pallet_membership::Instance1>::members();

        ensure!(council_members.contains(&who), Error::<T>::NotCouncilMember,);

        Ok(().into())
    }
}
