//! # Validator Set Pallet
//!
//! The Validator Set Pallet allows addition and removal of
//! authorities/validators via extrinsics (transaction calls), in
//! Substrate-based PoA networks. It also integrates with the im-online pallet
//! to automatically remove offline validators.
//!
//! The pallet uses the Session pallet and implements related traits for session
//! management. Currently it uses periodic session rotation provided by the
//! session pallet to automatically rotate sessions. For this reason, the
//! validator addition and removal becomes effective only after 2 sessions
//! (queuing + applying).

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

mod mock;
mod tests;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
mod validator_set;

pub mod weights;

pub const LOG_TARGET: &'static str = "runtime::validator-set";

#[frame_support::pallet]
pub mod pallet {
    use super::weights::WeightInfo;
    use super::*;
    use frame_support::{
        ensure,
        pallet_prelude::*,
        traits::{EstimateNextSessionRotation, Get, ValidatorSet, ValidatorSetWithIdentification},
    };
    use frame_system::pallet_prelude::*;
    use log;
    use sp_runtime::traits::{Convert, Zero};
    use sp_staking::offence::{Offence, OffenceError, ReportOffence};
    use sp_std::convert::TryInto;
    use sp_std::{collections::btree_set::BTreeSet, prelude::*};

    /// Configure the pallet by specifying the parameters and types on which it
    /// depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_session::Config {
        /// The Event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Origin for adding or removing a validator.
        type AddRemoveOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Minimum number of validators to leave in the validator set during
        /// auto removal.
        type MinAuthorities: Get<u32>;

        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn validators)]
    pub type Validators<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn approved_validators)]
    pub type ApprovedValidators<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn validators_to_remove)]
    pub type OfflineValidators<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// New validator addition initiated. Effective in ~2 sessions.
        ValidatorAdditionInitiated(T::AccountId),

        /// Validator removal initiated. Effective in ~2 sessions.
        ValidatorRemovalInitiated(T::AccountId),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Target (post-removal) validator count is below the minimum.
        TooLowValidatorCount,
        /// Validator is already in the validator set.
        Duplicate,
        /// Validator is not approved for re-addition.
        ValidatorNotApproved,
        /// Only the validator can add itself back after coming online.
        BadOrigin,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub initial_validators: Vec<T::AccountId>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            Pallet::<T>::initialize_validators(&self.initial_validators);
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Add a new validator.
        ///
        /// New validator's session keys should be set in Session pallet before
        /// calling this.
        ///
        /// The origin can be configured using the `AddRemoveOrigin` type in the
        /// host runtime. Can also be set to sudo/root.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::add_validator())]
        pub fn add_validator(origin: OriginFor<T>, validator_id: T::AccountId) -> DispatchResult {
            T::AddRemoveOrigin::ensure_origin(origin)?;

            Self::do_add_validator(validator_id.clone())?;
            Self::approve_validator(validator_id)?;

            Ok(())
        }

        /// Remove a validator.
        ///
        /// The origin can be configured using the `AddRemoveOrigin` type in the
        /// host runtime. Can also be set to sudo/root.
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_validator())]
        pub fn remove_validator(
            origin: OriginFor<T>,
            validator_id: T::AccountId,
        ) -> DispatchResult {
            T::AddRemoveOrigin::ensure_origin(origin)?;

            Self::do_remove_validator(validator_id.clone())?;
            Self::unapprove_validator(validator_id)?;

            Ok(())
        }

        /// Add an approved validator again when it comes back online.
        ///
        /// For this call, the dispatch origin must be the validator itself.
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::add_validator_again())]
        pub fn add_validator_again(
            origin: OriginFor<T>,
            validator_id: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(who == validator_id, Error::<T>::BadOrigin);

            let approved_set: BTreeSet<_> = <ApprovedValidators<T>>::get().into_iter().collect();
            ensure!(
                approved_set.contains(&validator_id),
                Error::<T>::ValidatorNotApproved
            );

            Self::do_add_validator(validator_id)?;

            Ok(())
        }
    }

    // Provides the new set of validators to the session module when session is
    // being rotated.
    impl<T: Config> pallet_session::SessionManager<T::AccountId> for Pallet<T> {
        // Plan a new session and provide new validator set.
        fn new_session(_new_index: u32) -> Option<Vec<T::AccountId>> {
            // Remove any offline validators. This will only work when the runtime
            // also has the im-online pallet.
            Self::remove_offline_validators();

            log::debug!(
                target: LOG_TARGET,
                "New session called; updated validator set provided."
            );

            Some(Self::validators())
        }

        fn end_session(_end_index: u32) {}

        fn start_session(_start_index: u32) {}
    }

    impl<T: Config> EstimateNextSessionRotation<BlockNumberFor<T>> for Pallet<T> {
        fn average_session_length() -> BlockNumberFor<T> {
            Zero::zero()
        }

        fn estimate_current_session_progress(
            _now: BlockNumberFor<T>,
        ) -> (Option<sp_runtime::Permill>, frame_support::dispatch::Weight) {
            (None, Zero::zero())
        }

        fn estimate_next_session_rotation(
            _now: BlockNumberFor<T>,
        ) -> (Option<BlockNumberFor<T>>, frame_support::dispatch::Weight) {
            (None, Zero::zero())
        }
    }

    // Implementation of Convert trait for mapping ValidatorId with AccountId.
    pub struct ValidatorOf<T>(sp_std::marker::PhantomData<T>);

    impl<T: Config> Convert<T::ValidatorId, Option<T::ValidatorId>> for ValidatorOf<T> {
        fn convert(account: T::ValidatorId) -> Option<T::ValidatorId> {
            Some(account)
        }
    }

    impl<T: Config> ValidatorSet<T::AccountId> for Pallet<T> {
        type ValidatorId = T::ValidatorId;
        type ValidatorIdOf = T::ValidatorIdOf;

        fn session_index() -> sp_staking::SessionIndex {
            pallet_session::Pallet::<T>::current_index()
        }

        fn validators() -> Vec<Self::ValidatorId> {
            pallet_session::Pallet::<T>::validators()
        }
    }

    impl<T: Config> ValidatorSetWithIdentification<T::AccountId> for Pallet<T> {
        type Identification = T::ValidatorId;
        type IdentificationOf = ValidatorOf<T>;
    }

    // Offence reporting and unresponsiveness management.
    impl<T: Config, O: Offence<(T::AccountId, T::AccountId)>>
        ReportOffence<T::AccountId, (T::AccountId, T::AccountId), O> for Pallet<T>
    {
        fn report_offence(_reporters: Vec<T::AccountId>, offence: O) -> Result<(), OffenceError> {
            let offenders = offence.offenders();

            for (v, _) in offenders.into_iter() {
                Self::mark_for_removal(v);
            }

            Ok(())
        }

        fn is_known_offence(
            _offenders: &[(T::AccountId, T::AccountId)],
            _time_slot: &O::TimeSlot,
        ) -> bool {
            false
        }
    }
}
