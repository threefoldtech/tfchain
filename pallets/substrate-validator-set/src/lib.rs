//! # Validator Set Pallet
//!
//! The Validator Set Pallet provides functionality to add/remove validators through extrinsics, in a Substrate-based
//! PoA network.
//!
//! The pallet is based on the Substrate session pallet and implements related traits for session
//! management when validators are added or removed.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{ChangeMembers, InitializeMembers};
use pallet_session::Module as Session;
use sp_runtime::traits::Convert;
use sp_std::prelude::*;

pub use pallet::*;

mod types;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::{DispatchError, DispatchErrorWithPostInfo, DispatchResultWithPostInfo},
		pallet_prelude::*,
	};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_session::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Origin for adding or removing a validator.
		type AddRemoveOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

		/// The receiver of the signal for when the membership has changed.
		type MembershipChanged: ChangeMembers<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	#[pallet::storage]
	#[pallet::getter(fn validators)]
	pub type Validators<T: Config> = StorageValue<_, Vec<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn flag)]
	pub type Flag<T: Config> = StorageValue<_, bool>;

	#[pallet::storage]
	#[pallet::getter(fn validator_requests)]
	pub type ValidatorRequest<T: Config> =
		StorageMap<_, Identity, u32, types::ValidatorRequest<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn executed_validator_requests)]
	pub type ExecutedValidatorRequest<T: Config> =
		StorageMap<_, Identity, u32, types::ValidatorRequest<T::AccountId>>;

	/// Proposals so far.
	#[pallet::storage]
	#[pallet::getter(fn request_count)]
	pub type RequestCount<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// New validator added.
		ValidatorAdded(T::AccountId),

		// Validator removed.
		ValidatorRemoved(T::AccountId),

		// Validator requests
		ValidatorRequestCreated(types::ValidatorRequest<T::AccountId>),
		ValidatorRequestApproved(types::ValidatorRequest<T::AccountId>),
		ValidatorRequestExecuted(types::ValidatorRequest<T::AccountId>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NoValidators,
		DuplicateValidatorRequest,
		UnauthorizedToActivateValidator,
		ValidatorRequestNotFound,
		ValidatorRequestNotApproved,
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
		fn build(&self) {
			Pallet::<T>::initialize_validators(&self.validators);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Add a new validator using root/sudo privileges.
		///
		/// New validator's session keys should be set in session module before calling this.
		#[pallet::weight(0)]
		pub fn add_validator(
			origin: OriginFor<T>,
			validator_id: T::AccountId,
		) -> DispatchResultWithPostInfo {
			T::AddRemoveOrigin::ensure_origin(origin)?;

			Self::do_add(validator_id.clone());
			// Calling rotate_session to queue the new session keys.
			Session::<T>::rotate_session();
			// Triggering rotate session again for the queued keys to take effect.
			Flag::<T>::put(true);

			Ok(().into())
		}

		/// Remove a validator using root/sudo privileges.
		#[pallet::weight(0)]
		pub fn remove_validator(
			origin: OriginFor<T>,
			validator_id: T::AccountId,
		) -> DispatchResultWithPostInfo {
			T::AddRemoveOrigin::ensure_origin(origin)?;
			let mut validators = <Validators<T>>::get().ok_or(Error::<T>::NoValidators)?;

			// Assuming that this will be a PoA network for enterprise use-cases,
			// the validator count may not be too big; the for loop shouldn't be too heavy.
			// In case the validator count is large, we need to find another way.
			for (i, v) in validators.clone().into_iter().enumerate() {
				if v == validator_id {
					validators.swap_remove(i);
				}
			}
			<Validators<T>>::put(validators);
			// Calling rotate_session to queue the new session keys.
			<pallet_session::Module<T>>::rotate_session();
			Self::deposit_event(Event::ValidatorRemoved(validator_id));

			// Triggering rotate session again for the queued keys to take effect.
			Flag::<T>::put(true);
			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn force_change_session(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			T::AddRemoveOrigin::ensure_origin(origin)?;
			<pallet_session::Module<T>>::rotate_session();
			Flag::<T>::put(true);
			Ok(().into())
		}

		// Activate Validator takes in a validator request ID
		// Based on that ID, it can fetch the validator request from storage
		// check if the signer is mentioned as the council address from the request (this way we know if the signer is an active council member)
		// if true, then we add the validator account from the request to the list of validators
		#[pallet::weight(0)]
		pub fn activate_validator(
			origin: OriginFor<T>,
			validator_request_id: u32,
		) -> DispatchResultWithPostInfo {
			let address = ensure_signed(origin)?;

			let validator_request = <ValidatorRequest<T>>::get(validator_request_id)
				.ok_or(DispatchError::from(Error::<T>::ValidatorRequestNotFound))?;

			ensure!(
				validator_request.approved,
				Error::<T>::ValidatorRequestNotApproved
			);
			ensure!(
				validator_request.council_account == address,
				Error::<T>::UnauthorizedToActivateValidator
			);

			// Remove the request from the original map
			<ValidatorRequest<T>>::remove(validator_request_id);
			// Insert the request into the executed ones, this way this council member cannot
			// do this call more than once
			<ExecutedValidatorRequest<T>>::insert(validator_request_id, &validator_request);

			Self::do_add(validator_request.validator_account);
			// Calling rotate_session to queue the new session keys.
			Session::<T>::rotate_session();
			// Triggering rotate session again for the queued keys to take effect.
			Flag::<T>::put(true);

			Ok(().into())
		}

		// create a request to become a validator
		// Validator account: the account of the validator
		// Stash account: the "bank" account of the validator (where rewards should be sent to)
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

			let validator_requests = <ValidatorRequest<T>>::iter();
			for (_, req) in validator_requests {
				if req.council_account == address {
					return Err(DispatchErrorWithPostInfo::from(
						Error::<T>::DuplicateValidatorRequest,
					));
				}
			}

			let id = Self::request_count();
			let request = types::ValidatorRequest {
				id,
				council_account: address,
				validator_account,
				stash_account,
				description,
				tf_connect_id,
				info,
				approved: false,
			};

			<RequestCount<T>>::mutate(|i| *i += 1);

			// Create a validator request object
			<ValidatorRequest<T>>::insert(id, &request);
			Self::deposit_event(Event::ValidatorRequestCreated(request.clone()));

			Ok(().into())
		}

		// Approve a validator request by ID
		#[pallet::weight(0)]
		pub fn approve_validator_request(
			origin: OriginFor<T>,
			validator_request_id: u32,
		) -> DispatchResultWithPostInfo {
			T::AddRemoveOrigin::ensure_origin(origin)?;

			let validator_requests = <ValidatorRequest<T>>::iter();
			for (id, mut req) in validator_requests {
				if req.id == validator_request_id {
					req.approved = true;
					<ValidatorRequest<T>>::insert(id, &req);
					Self::deposit_event(Event::ValidatorRequestApproved(req));
				}
			}

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn do_add(validator_id: T::AccountId) {
		let mut validators: Vec<T::AccountId>;

		if <Validators<T>>::get().is_none() {
			validators = vec![validator_id.clone()];
		} else {
			validators = <Validators<T>>::get().unwrap();
			// insert the validator if it does not exist yet in the list
			if !validators.contains(&validator_id.clone()) {
				validators.push(validator_id.clone());
				Self::deposit_event(Event::ValidatorAdded(validator_id));
			}
		}

		<Validators<T>>::put(validators);
	}

	fn do_approve_request(council_address: T::AccountId) {
		let validator_requests = <ValidatorRequest<T>>::iter();
		for (id, mut req) in validator_requests {
			if req.council_account == council_address {
				req.approved = true;
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

impl<T: Config> InitializeMembers<T::AccountId> for Pallet<T> {
	fn initialize_members(init: &[T::AccountId]) {
		<Validators<T>>::put(init);
		// Shouldn't need a flag update here as this should happen at genesis
	}
}

impl<T: Config> Pallet<T> {
	fn initialize_validators(validators: &[T::AccountId]) {
		if !validators.is_empty() {
			assert!(
				<Validators<T>>::get().is_none(),
				"Validators are already initialized!"
			);
			<Validators<T>>::put(validators);
		}
	}
}

/// Indicates to the session module if the session should be rotated.
/// We set this flag to true when we add/remove a validator.
impl<T: Config> pallet_session::ShouldEndSession<T::BlockNumber> for Module<T> {
	fn should_end_session(_now: T::BlockNumber) -> bool {
		Flag::<T>::get().unwrap()
	}
}

/// Provides the new set of validators to the session module when session is being rotated.
impl<T: Config> pallet_session::SessionManager<T::AccountId> for Module<T> {
	fn new_session(_new_index: u32) -> Option<Vec<T::AccountId>> {
		// Flag is set to false so that the session doesn't keep rotating.
		Flag::<T>::put(false);

		Self::validators()
	}

	fn end_session(_end_index: u32) {}

	fn start_session(_start_index: u32) {}
}

impl<T: Config> frame_support::traits::EstimateNextSessionRotation<T::BlockNumber> for Module<T> {
	fn estimate_next_session_rotation(_now: T::BlockNumber) -> Option<T::BlockNumber> {
		None
	}

	// The validity of this weight depends on the implementation of `estimate_next_session_rotation`
	fn weight(_now: T::BlockNumber) -> u64 {
		0
	}
}

/// Implementation of Convert trait for mapping ValidatorId with AccountId.
/// This is mainly used to map stash and controller keys.
/// In this module, for simplicity, we just return the same AccountId.
pub struct ValidatorOf<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> Convert<T::AccountId, Option<T::AccountId>> for ValidatorOf<T> {
	fn convert(account: T::AccountId) -> Option<T::AccountId> {
		Some(account)
	}
}
