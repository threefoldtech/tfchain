//! # Validator Pallet
//!
//! The Validator Pallet provides functionality for tfchain DAO validators.
//!

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support;
use frame_support::traits::Currency;
use frame_support::{
	dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo},
	pallet_prelude::*,
};
use sp_runtime::traits::StaticLookup;
use sp_std::prelude::*;
use substrate_validator_set;

pub mod types;
pub use pallet::*;

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
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn validator_requests)]
	pub type Validator<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, types::Validator<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn bonded)]
	pub type Bonded<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, T::AccountId>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Bonded(T::AccountId),
		ValidatorCreated(T::AccountId, types::Validator<T::AccountId>),
		ValidatorApproved(types::Validator<T::AccountId>),
		ValidatorActivated(types::Validator<T::AccountId>),
		ValidatorRemoved(types::Validator<T::AccountId>),
	}

	#[pallet::error]
	pub enum Error<T> {
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
		// create a request to become a validator
		// Validator account: the account of the validator
		// Stash account: the "bank" account of the validator (where rewards should be sent to) the stash should be bonded to a validator
		// Description: why someone wants to become a validator
		// Tf Connect ID: the threefold connect ID of the persion who wants to become a validator
		// Info: some public info about the validator (website link, blog link, ..)
		// A user can only have 1 validator request at a time
		#[pallet::weight(100_000_000)]
		pub fn create_validator(
			origin: OriginFor<T>,
			validator_node_account: T::AccountId,
			stash_account: T::AccountId,
			description: Vec<u8>,
			tf_connect_id: u64,
			info: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let address = ensure_signed(origin.clone())?;

			// Request should not be a duplicate
			ensure!(
				!<Validator<T>>::contains_key(&address),
				Error::<T>::DuplicateValidator
			);
			// Request stash account should be bonded
			ensure!(
				<Bonded<T>>::contains_key(&stash_account),
				Error::<T>::StashNotBonded
			);
			Self::check_bond(&stash_account, &address)?;

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
			Self::deposit_event(Event::ValidatorCreated(address, request.clone()));

			Ok(().into())
		}

		// Activate Validator takes in a validator request ID
		// Based on that ID, it can fetch the validator request from storage
		// check if the signer is mentioned as the council address from the request (this way we know if the signer is an active council member)
		// if true, then we add the validator account from the request to the list of validators
		#[pallet::weight(100_000_000)]
		pub fn activate_validator(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
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

		#[pallet::weight(100_000_000)]
		pub fn change_node_validator_account(
			origin: OriginFor<T>,
			new_node_validator_account: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let address = ensure_signed(origin)?;

			let mut validator = <Validator<T>>::get(&address)
				.ok_or(DispatchError::from(Error::<T>::ValidatorNotFound))?;

			ensure!(
				validator.state == types::ValidatorRequestState::Validating,
				Error::<T>::ValidatorNotValidating
			);

			// Remove the old validator and rotate session
			substrate_validator_set::Pallet::<T>::remove_validator(
				frame_system::RawOrigin::Root.into(),
				validator.validator_node_account.clone(),
			)?;
			Self::deposit_event(Event::ValidatorRemoved(validator.clone()));

			// Set the new validator node account on the validator struct
			validator.validator_node_account = new_node_validator_account.clone();
			<Validator<T>>::insert(address, &validator);

			// Add the new validator and rotate session
			substrate_validator_set::Pallet::<T>::add_validator(
				frame_system::RawOrigin::Root.into(),
				new_node_validator_account.clone(),
			)?;
			Self::deposit_event(Event::ValidatorActivated(validator));

			Ok(().into())
		}

		/// Take the origin account as a stash and lock up `value` of its balance. `validator` will
		/// be the account that controls it.
		///
		/// `value` must be more than the `minimum_balance` specified by `T::Currency`.
		///
		/// The dispatch origin for this call must be _Signed_ by the stash account.
		///
		/// Emits `Bonded`.
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

			// Cannot bond with yourself
			if validator == stash {
				Err(Error::<T>::CannotBondWithSameAccount)?
			}

			<Bonded<T>>::insert(&stash, &validator);

			Self::deposit_event(Event::Bonded(stash.clone()));

			Ok(().into())
		}

		// Approve a validator request by validator account id
		#[pallet::weight(100_000_000)]
		pub fn approve_validator(
			origin: OriginFor<T>,
			validator_account: T::AccountId,
		) -> DispatchResultWithPostInfo {
			T::CouncilOrigin::ensure_origin(origin)?;

			let mut validator = <Validator<T>>::get(&validator_account)
				.ok_or(DispatchError::from(Error::<T>::ValidatorNotFound))?;

			validator.state = types::ValidatorRequestState::Approved;
			<Validator<T>>::insert(validator_account.clone(), &validator);

			// Add the validator as a council member
			pallet_membership::Module::<T, pallet_membership::Instance1>::add_member(
				frame_system::RawOrigin::Root.into(),
				validator_account.clone(),
			)?;

			Self::deposit_event(Event::ValidatorApproved(validator));

			Ok(().into())
		}

		// Removes a validator from:
		// 1. Council
		// 2. Storage
		// 3. Consensus
		// Can only be decided by the council
		#[pallet::weight(100_000_000)]
		pub fn remove_validator(
			origin: OriginFor<T>,
			validator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			T::CouncilOrigin::ensure_origin(origin)?;

			let v = <Validator<T>>::get(&validator)
				.ok_or(DispatchError::from(Error::<T>::ValidatorNotFound))?;

			// Remove the validator as a council member
			pallet_membership::Module::<T, pallet_membership::Instance1>::remove_member(
				frame_system::RawOrigin::Root.into(),
				validator.clone(),
			)?;

			// Remove the entry from the storage map
			<Validator<T>>::remove(validator);

			// Remove the old validator and rotate session
			substrate_validator_set::Pallet::<T>::remove_validator(
				frame_system::RawOrigin::Root.into(),
				v.validator_node_account.clone(),
			)?;

			Self::deposit_event(Event::ValidatorRemoved(v.clone()));

			Ok(().into())
		}
	}

	impl<T: Config> Module<T> {
		fn check_bond(
			stash_account: &T::AccountId,
			validator: &T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure!(
				<Bonded<T>>::contains_key(&stash_account),
				Error::<T>::StashNotBonded
			);
			let bonded_account = <Bonded<T>>::get(&stash_account)
				.ok_or(DispatchError::from(Error::<T>::StashNotBonded))?;

			if &bonded_account != validator {
				return Err(DispatchErrorWithPostInfo::from(
					Error::<T>::StashBondedWithWrongValidator,
				));
			} else {
				Ok(().into())
			}
		}
	}
}
