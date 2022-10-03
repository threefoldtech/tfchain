//! # Validator Pallet
//!
//! The Validator Pallet provides functionality for tfchain DAO validators.
//!

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support;
use frame_support::traits::Currency;
use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
use sp_runtime::traits::StaticLookup;
use sp_std::convert::TryInto;
use sp_std::prelude::*;
use substrate_validator_set;

pub mod types;
pub use pallet::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_system::pallet_prelude::*;
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + substrate_validator_set::Config
        + pallet_membership::Config<pallet_membership::Instance1>
    {
        /// Because this pallet emits events, it depends on the runtime's definition of an event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: Currency<Self::AccountId>;
        type CouncilOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn validator_requests)]
    pub type Validator<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, types::Validator<T::AccountId>>;

    #[pallet::storage]
    #[pallet::getter(fn bonded)]
    pub type Bonded<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, T::AccountId>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Bonded(T::AccountId),
        ValidatorRequestCreated(T::AccountId, types::Validator<T::AccountId>),
        ValidatorRequestApproved(types::Validator<T::AccountId>),
        ValidatorActivated(types::Validator<T::AccountId>),
        ValidatorRemoved(types::Validator<T::AccountId>),
        NodeValidatorChanged(T::AccountId),
        NodeValidatorRemoved(T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        BadOrigin,
        AlreadyBonded,
        StashNotBonded,
        StashBondedWithWrongValidator,
        CannotBondWithSameAccount,
        DuplicateValidator,
        ValidatorNotFound,
        ValidatorNotApproved,
        UnauthorizedToActivateValidator,
        ValidatorValidatingAlready,
        ValidatorNotValidating,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub validators: Vec<T::AccountId>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                validators: Vec::new(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {}
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a request to become a validator
        /// Validator account (signer): the account of the validator (this account will be added to the council)
        /// Validator node account: the account that will validate on consensus layer
        /// Stash account: the "bank" account of the validator (where rewards should be sent to) the stash should be bonded to a validator
        /// Description: why someone wants to become a validator
        /// Tf Connect ID: the threefold connect ID of the person who wants to become a validator
        /// Info: some public info about the validator (website link, blog link, ..)
        /// A user can only have 1 validator request at a time
        #[pallet::weight(100_000_000)]
        pub fn create_validator_request(
            origin: OriginFor<T>,
            validator_node_account: T::AccountId,
            stash_account: T::AccountId,
            description: Vec<u8>,
            tf_connect_id: Vec<u8>,
            info: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let address = ensure_signed(origin.clone())?;

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

        /// Start participating in consensus
        /// Will activate the Validator node account on consensus level
        /// A user can only call this if his request to be a validator is approved by the council
        /// Should be called when his node is synced and ready to start validating
        #[pallet::weight(100_000_000)]
        pub fn activate_validator_node(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let address = ensure_signed(origin)?;

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

        /// Change validator node account
        /// In case the Validator wishes to change his validator node account
        /// he can call this method with the new node validator account
        /// this new account will be added as a new consensus validator if he is validating already
        #[pallet::weight(100_000_000)]
        pub fn change_validator_node_account(
            origin: OriginFor<T>,
            new_node_validator_account: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let address = ensure_signed(origin)?;

            let mut validator = <Validator<T>>::get(&address)
                .ok_or(DispatchError::from(Error::<T>::ValidatorNotFound))?;

            // Set the new validator node account on the validator struct
            validator.validator_node_account = new_node_validator_account.clone();
            <Validator<T>>::insert(address, &validator);

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
                Self::deposit_event(Event::NodeValidatorChanged(new_node_validator_account));
            }

            Ok(().into())
        }

        /// Bond an account to to a validator account
        /// Just proves that the stash account is indeed under control of the validator account
        #[pallet::weight(100_000_000)]
        pub fn bond(
            origin: OriginFor<T>,
            validator: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            let stash = ensure_signed(origin.clone())?;

            if <Bonded<T>>::contains_key(&stash) {
                Err(Error::<T>::AlreadyBonded)?
            }
            let validator = T::Lookup::lookup(validator)?;

            <Bonded<T>>::insert(&stash, &validator);

            Self::deposit_event(Event::Bonded(stash.clone()));

            Ok(().into())
        }

        /// Approve validator (council)
        /// Approves a validator to be added as a council member and
        /// to participate in consensus
        #[pallet::weight(100_000_000)]
        pub fn approve_validator(
            origin: OriginFor<T>,
            validator_account: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            T::CouncilOrigin::ensure_origin(origin)?;

            let mut validator = <Validator<T>>::get(&validator_account)
                .ok_or(DispatchError::from(Error::<T>::ValidatorNotFound))?;

            // Set state to approved
            validator.state = types::ValidatorRequestState::Approved;
            <Validator<T>>::insert(validator_account.clone(), &validator);

            // Add the validator as a council member
            pallet_membership::Pallet::<T, pallet_membership::Instance1>::add_member(
                frame_system::RawOrigin::Root.into(),
                validator_account.clone(),
            )?;

            Self::deposit_event(Event::ValidatorRequestApproved(validator));

            Ok(().into())
        }

        /// Remove validator
        /// Removes a validator from:
        /// 1. Council
        /// 2. Storage
        /// 3. Consensus
        /// Can only be called by the user or the council
        #[pallet::weight(100_000_000)]
        pub fn remove_validator(
            origin: OriginFor<T>,
            validator: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            if !(ensure_signed(origin.clone())? == validator
                || T::CouncilOrigin::ensure_origin(origin).is_ok())
            {
                Err(Error::<T>::BadOrigin)?
            }

            let _ = <Validator<T>>::get(&validator)
                .ok_or(DispatchError::from(Error::<T>::ValidatorNotFound))?;

            // Remove the validator as a council member
            pallet_membership::Pallet::<T, pallet_membership::Instance1>::remove_member(
                frame_system::RawOrigin::Root.into(),
                validator.clone(),
            )?;

            // Remove the entry from the storage map
            <Validator<T>>::remove(validator);

            // let node_validators = substrate_validator_set::Validators::<T>::get();

            // match node_validators {
            // 	Some(validators) => {
            // 		for (_, val) in validators.clone().into_iter().enumerate() {
            // 			if val == v.validator_node_account {
            // 				// Remove the old validator and rotate session
            // 				substrate_validator_set::Pallet::<T>::remove_validator(
            // 					frame_system::RawOrigin::Root.into(),
            // 					v.validator_node_account.clone(),
            // 				)?;

            // 				Self::deposit_event(Event::ValidatorRemoved(v.clone()));
            // 			}
            // 		}
            // 	},
            // 	None => ()
            // }

            Ok(().into())
        }
    }
}
