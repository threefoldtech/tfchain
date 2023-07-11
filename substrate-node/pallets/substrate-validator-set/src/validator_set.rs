use crate::pallet::{Error, Event};
use crate::ApprovedValidators;
use crate::Config;
use crate::Pallet;
use crate::Validators;
use crate::{OfflineValidators, LOG_TARGET};
use frame_support::traits::Get;
use frame_support::{dispatch::DispatchResult, ensure};
use log;
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};

impl<T: Config> Pallet<T> {
    pub(crate) fn initialize_validators(validators: &[T::AccountId]) {
        assert!(
            validators.len() as u32 >= T::MinAuthorities::get(),
            "Initial set of validators must be at least T::MinAuthorities"
        );
        assert!(
            <Validators<T>>::get().is_empty(),
            "Validators are already initialized!"
        );

        <Validators<T>>::put(validators);
        <ApprovedValidators<T>>::put(validators);
    }

    pub(crate) fn do_add_validator(validator_id: T::AccountId) -> DispatchResult {
        let validator_set: BTreeSet<_> = <Validators<T>>::get().into_iter().collect();
        ensure!(
            !validator_set.contains(&validator_id),
            Error::<T>::Duplicate
        );
        <Validators<T>>::mutate(|v| v.push(validator_id.clone()));

        Self::deposit_event(Event::ValidatorAdditionInitiated(validator_id.clone()));
        log::debug!(target: LOG_TARGET, "Validator addition initiated.");

        Ok(())
    }

    pub(crate) fn do_remove_validator(validator_id: T::AccountId) -> DispatchResult {
        let mut validators = <Validators<T>>::get();

        // Ensuring that the post removal, target validator count doesn't go
        // below the minimum.
        ensure!(
            validators.len().saturating_sub(1) as u32 >= T::MinAuthorities::get(),
            Error::<T>::TooLowValidatorCount
        );

        validators.retain(|v| *v != validator_id);

        <Validators<T>>::put(validators);

        Self::deposit_event(Event::ValidatorRemovalInitiated(validator_id.clone()));
        log::debug!(target: LOG_TARGET, "Validator removal initiated.");

        Ok(())
    }

    pub(crate) fn approve_validator(validator_id: T::AccountId) -> DispatchResult {
        let approved_set: BTreeSet<_> = <ApprovedValidators<T>>::get().into_iter().collect();
        ensure!(!approved_set.contains(&validator_id), Error::<T>::Duplicate);
        <ApprovedValidators<T>>::mutate(|v| v.push(validator_id.clone()));
        Ok(())
    }

    pub(crate) fn unapprove_validator(validator_id: T::AccountId) -> DispatchResult {
        let mut approved_set = <ApprovedValidators<T>>::get();
        approved_set.retain(|v| *v != validator_id);
        Ok(())
    }

    // Adds offline validators to a local cache for removal at new session.
    pub(crate) fn mark_for_removal(validator_id: T::AccountId) {
        <OfflineValidators<T>>::mutate(|v| v.push(validator_id));
        log::debug!(
            target: LOG_TARGET,
            "Offline validator marked for auto removal."
        );
    }

    // Removes offline validators from the validator set and clears the offline
    // cache. It is called in the session change hook and removes the validators
    // who were reported offline during the session that is ending. We do not
    // check for `MinAuthorities` here, because the offline validators will not
    // produce blocks and will have the same overall effect on the runtime.
    pub(crate) fn remove_offline_validators() {
        let validators_to_remove: BTreeSet<_> = <OfflineValidators<T>>::get().into_iter().collect();

        // Delete from active validator set.
        <Validators<T>>::mutate(|vs| vs.retain(|v| !validators_to_remove.contains(v)));
        log::debug!(
            target: LOG_TARGET,
            "Initiated removal of {:?} offline validators.",
            validators_to_remove.len()
        );

        // Clear the offline validator list to avoid repeated deletion.
        <OfflineValidators<T>>::put(Vec::<T::AccountId>::new());
    }
}
