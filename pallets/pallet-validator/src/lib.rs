//! # Validator Pallet
//!
//! The Validator Pallet provides functionality for tfchain DAO validators.
//!

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support;
use frame_support::traits::{ChangeMembers, Currency};
use frame_support::{
	dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo},
	pallet_prelude::*,
};
use sp_std::prelude::*;
use sp_runtime::{
	traits::{StaticLookup}
};
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
	pub trait Config: frame_system::Config + substrate_validator_set::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;
		type CouncilOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;
		/// The receiver of the signal for when the membership has changed.
		type MembershipChanged: ChangeMembers<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn validator_requests)]
	pub type ValidatorRequest<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, types::Validator<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn bonded)]
	pub type Bonded<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, T::AccountId>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Bonded(T::AccountId),
		ValidatorRequestCreated(T::AccountId, types::Validator<T::AccountId>),
		ValidatorRequestApproved(types::Validator<T::AccountId>),
	}

	#[pallet::error]
	pub enum Error<T> {
		AlreadyBonded,
		StashNotBonded,
		StashBondedWithWrongValidator,
		InsufficientValue,
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
		#[pallet::weight(0)]
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
			ensure!(!<ValidatorRequest<T>>::contains_key(&validator_node_account), Error::<T>::DuplicateValidator);
			// Request stash account should be bonded
			ensure!(<Bonded<T>>::contains_key(&stash_account), Error::<T>::StashNotBonded);
			Self::check_bond(&stash_account, &validator_node_account)?;

			let request = types::Validator {
				council_account: address.clone(),
				validator_node_account: validator_node_account.clone(),
				stash_account,
				description,
				tf_connect_id,
				info,
				state: types::ValidatorRequestState::Created,
			};

			// Create a validator request object
			<ValidatorRequest<T>>::insert(validator_node_account, &request);
			Self::deposit_event(Event::ValidatorRequestCreated(address, request.clone()));

			Ok(().into())
		}

		// Activate Validator takes in a validator request ID
		// Based on that ID, it can fetch the validator request from storage
		// check if the signer is mentioned as the council address from the request (this way we know if the signer is an active council member)
		// if true, then we add the validator account from the request to the list of validators
		#[pallet::weight(0)]
		pub fn activate_validator(
			origin: OriginFor<T>,
			validator_node_account: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let address = ensure_signed(origin)?;

			let mut validator = <ValidatorRequest<T>>::get(&validator_node_account)
				.ok_or(DispatchError::from(Error::<T>::ValidatorNotFound))?;

			ensure!(
				validator.state != types::ValidatorRequestState::Validating,
				Error::<T>::ValidatorValidatingAlready
			);
			ensure!(
				validator.state == types::ValidatorRequestState::Approved,
				Error::<T>::ValidatorNotApproved
			);
			ensure!(
				validator.council_account == address,
				Error::<T>::UnauthorizedToActivateValidator
			);

			// Update the validator request
			validator.state = types::ValidatorRequestState::Validating;
			<ValidatorRequest<T>>::insert(validator_node_account.clone(), &validator);

			// Add the validator and rotate
			substrate_validator_set::Pallet::<T>::add_validator(
				frame_system::RawOrigin::Root.into(),
				validator_node_account
			)?;

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn change_node_validator_account(
			origin: OriginFor<T>,
			old_node_validator_account: T::AccountId,
			new_node_validator_account: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let address = ensure_signed(origin)?;

			let mut validator = <ValidatorRequest<T>>::get(&old_node_validator_account)
				.ok_or(DispatchError::from(Error::<T>::ValidatorNotFound))?;

			ensure!(
				validator.state == types::ValidatorRequestState::Validating,
				Error::<T>::ValidatorNotValidating
			);
			ensure!(
				validator.council_account == address,
				Error::<T>::UnauthorizedToActivateValidator
			);

			// Remove the old validator and rotate session
			substrate_validator_set::Pallet::<T>::remove_validator(
				frame_system::RawOrigin::Root.into(),
				old_node_validator_account.clone()
			)?;
			// Add the validator and rotate session
			substrate_validator_set::Pallet::<T>::add_validator(
				frame_system::RawOrigin::Root.into(),
				new_node_validator_account.clone()
			)?;

			<ValidatorRequest<T>>::remove(old_node_validator_account);

			validator.validator_node_account = new_node_validator_account.clone();
			<ValidatorRequest<T>>::insert(new_node_validator_account, validator);

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
		#[pallet::weight(0)]
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

		// Approve a validator request by validator account id
		#[pallet::weight(0)]
		pub fn approve_validator(
			origin: OriginFor<T>,
			validator_account: T::AccountId,
		) -> DispatchResultWithPostInfo {
			T::CouncilOrigin::ensure_origin(origin)?;

			let req = <ValidatorRequest<T>>::get(&validator_account);
			match req {
				Some(mut r) => {
					r.state = types::ValidatorRequestState::Approved;
					<ValidatorRequest<T>>::insert(validator_account, &r);
					Ok(().into())
				},
				None => {
					return Err(DispatchErrorWithPostInfo::from(
						Error::<T>::ValidatorNotFound,
					))
				}
			}
		}
	}

	impl <T:Config> Module<T> {
		fn check_bond(stash_account: &T::AccountId, validator: &T::AccountId) -> DispatchResultWithPostInfo {
			ensure!(<Bonded<T>>::contains_key(&stash_account), Error::<T>::StashNotBonded);
			let bonded_account = <Bonded<T>>::get(&stash_account);
			match bonded_account {
				Some(acc) => {
					if &acc != validator {
						return Err(DispatchErrorWithPostInfo::from(
							Error::<T>::StashBondedWithWrongValidator,
						))
					} else {
						Ok(().into())
					}
				},
				None => return Err(DispatchErrorWithPostInfo::from(
					Error::<T>::StashNotBonded,
				))
			}

		}

		fn do_approve_request(council_address: T::AccountId) {
			let validator_requests = <ValidatorRequest<T>>::iter();
			for (id, mut req) in validator_requests {
				if req.council_account == council_address {
					req.state = types::ValidatorRequestState::Approved;
					<ValidatorRequest<T>>::insert(id, &req);
					Self::deposit_event(Event::ValidatorRequestApproved(req));
				}
			}
		}
	}

	impl<T: Config> ChangeMembers<T::AccountId> for Pallet<T> {
		fn change_members_sorted(
			_incoming: &[T::AccountId],
			_outgoing: &[T::AccountId],
			new: &[T::AccountId],
		) {
			for council_member in new {
				Self::do_approve_request(council_member.clone());
			}
		}
	}
}
