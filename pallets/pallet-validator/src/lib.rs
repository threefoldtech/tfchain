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

pub mod types;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;
		type AddRemoveOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;
		/// The receiver of the signal for when the membership has changed.
		type MembershipChanged: ChangeMembers<Self::AccountId>;
	}

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn validator_requests)]
	pub type ValidatorRequest<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, types::ValidatorRequest<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn bonded)]
	pub type Bonded<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, T::AccountId>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Bonded(T::AccountId, BalanceOf<T>),
		ValidatorRequestCreated(T::AccountId, types::ValidatorRequest<T::AccountId>),
	}

	#[pallet::error]
	pub enum Error<T> {
		AlreadyBonded,
		StashNotBonded,
		InsufficientValue,
		DuplicateValidatorRequest,
		ValidatorRequestNotFound,
		ValidatorRequestNotApproved,
		UnauthorizedToActivateValidator,
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
		// Stash account: the "bank" account of the validator (where rewards should be sent to) the stash should be bonded to a controller
		// Description: why someone wants to become a validator
		// Tf Connect ID: the threefold connect ID of the persion who wants to become a validator
		// Info: some public info about the validator (website link, blog link, ..)
		// A user can only have 1 validator request at a time
		#[pallet::weight(0)]
		pub fn create_validator_request(
			origin: OriginFor<T>,
			validator_account: T::AccountId,
			stash_account: T::AccountId,
			description: Vec<u8>,
			tf_connect_id: u64,
			info: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let address = ensure_signed(origin.clone())?;

			// Request should not be a duplicate
			ensure!(!<ValidatorRequest<T>>::contains_key(&validator_account), Error::<T>::DuplicateValidatorRequest);
			// Request stash account should be bonded
			ensure!(!<Bonded<T>>::contains_key(&stash_account), Error::<T>::StashNotBonded);

			let request = types::ValidatorRequest {
				council_account: address.clone(),
				validator_account: validator_account.clone(),
				stash_account,
				description,
				tf_connect_id,
				info,
				state: types::ValidatorRequestState::Created,
			};

			// Create a validator request object
			<ValidatorRequest<T>>::insert(validator_account, &request);
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
			validator_account: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let address = ensure_signed(origin)?;

			let mut validator_request = <ValidatorRequest<T>>::get(&validator_account)
				.ok_or(DispatchError::from(Error::<T>::ValidatorRequestNotFound))?;

			ensure!(
				validator_request.state == types::ValidatorRequestState::Approved,
				Error::<T>::ValidatorRequestNotApproved
			);
			ensure!(
				validator_request.council_account == address,
				Error::<T>::UnauthorizedToActivateValidator
			);

			// Update the validator request
			validator_request.state = types::ValidatorRequestState::Executed;
			<ValidatorRequest<T>>::insert(validator_account, &validator_request);

			// TODO
			// Call substrate pallet validatorset and add the validator

			Ok(().into())
		}

		/// Take the origin account as a stash and lock up `value` of its balance. `controller` will
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
			controller: <T::Lookup as StaticLookup>::Source,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let stash = ensure_signed(origin.clone())?;

			if <Bonded<T>>::contains_key(&stash) {
				Err(Error::<T>::AlreadyBonded)?
			}

			// reject a bond which is considered to be _dust_.
			if value < T::Currency::minimum_balance() {
				Err(Error::<T>::InsufficientValue)?
			}
			
			let controller = T::Lookup::lookup(controller)?;
			<Bonded<T>>::insert(&stash, &controller);

			let stash_balance = T::Currency::free_balance(&stash);
			let value = value.min(stash_balance);
			Self::deposit_event(Event::Bonded(stash.clone(), value));

			Ok(().into())
		}

		// Approve a validator request by ID
		#[pallet::weight(0)]
		pub fn approve_validator_request(
			origin: OriginFor<T>,
			validator_account: T::AccountId,
		) -> DispatchResultWithPostInfo {
			T::AddRemoveOrigin::ensure_origin(origin)?;

			let req = <ValidatorRequest<T>>::get(&validator_account);
			match req {
				Some(mut r) => {
					r.state = types::ValidatorRequestState::Approved;
					<ValidatorRequest<T>>::insert(validator_account, &r);
					Ok(().into())
				},
				None => {
					return Err(DispatchErrorWithPostInfo::from(
						Error::<T>::ValidatorRequestNotFound,
					))
				}
			}
		}
	}
}
