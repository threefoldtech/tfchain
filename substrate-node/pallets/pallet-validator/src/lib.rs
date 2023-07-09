//! # Validator Pallet
//!
//! The Validator Pallet provides functionality for tfchain DAO validators.
//!

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod types;
mod validator;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use super::{weights::WeightInfo, *};
    use frame_support::{
        dispatch::DispatchResultWithPostInfo, pallet_prelude::*, traits::Currency,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::StaticLookup;
    use sp_std::{convert::TryInto, prelude::*};
    use substrate_validator_set;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + substrate_validator_set::Config
        + pallet_membership::Config<pallet_membership::Instance1>
    {
        /// Because this pallet emits events, it depends on the runtime's definition of an event
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Currency: Currency<Self::AccountId>;
        type CouncilOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
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
        NotCouncilMember,
        AlreadyBonded,
        StashNotBonded,                // TOCHECK: Unused
        StashBondedWithWrongValidator, // TOCHECK: Unused
        CannotBondWithSameAccount,     // TOCHECK: Unused
        DuplicateValidator,
        ValidatorNotFound,
        ValidatorNotApproved,
        UnauthorizedToActivateValidator, // TOCHECK: Unused
        ValidatorValidatingAlready,
        ValidatorNotValidating, // TOCHECK: Unused
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
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::create_validator_request())]
        pub fn create_validator_request(
            origin: OriginFor<T>,
            validator_node_account: T::AccountId,
            stash_account: T::AccountId,
            description: Vec<u8>,
            tf_connect_id: Vec<u8>,
            info: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let address = ensure_signed(origin.clone())?;
            Self::_create_validator_request(
                address,
                validator_node_account,
                stash_account,
                description,
                tf_connect_id,
                info,
            )
        }

        /// Start participating in consensus
        /// Will activate the Validator node account on consensus level
        /// A user can only call this if his request to be a validator is approved by the council
        /// Should be called when his node is synced and ready to start validating
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::activate_validator_node())]
        pub fn activate_validator_node(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let address = ensure_signed(origin)?;
            Self::_activate_validator_node(address)
        }

        /// Change validator node account
        /// In case the Validator wishes to change his validator node account
        /// he can call this method with the new node validator account
        /// this new account will be added as a new consensus validator if he is validating already
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::change_validator_node_account())]
        pub fn change_validator_node_account(
            origin: OriginFor<T>,
            new_node_validator_account: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let address = ensure_signed(origin)?;
            Self::_change_validator_node_account(address, new_node_validator_account)
        }

        /// Bond an account to a validator account
        /// Just proves that the stash account is indeed under control of the validator account
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::bond())]
        pub fn bond(
            origin: OriginFor<T>,
            validator: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            let stash = ensure_signed(origin.clone())?;
            Self::_bond(stash, validator)
        }

        /// Approve validator (council)
        /// Approves a validator to be added as a council member and
        /// to participate in consensus
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::approve_validator())]
        pub fn approve_validator(
            origin: OriginFor<T>,
            validator_account: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::_approve_validator(who, validator_account)
        }

        /// Remove validator
        /// Removes a validator from:
        /// 1. Council
        /// 2. Storage
        /// 3. Consensus
        /// Can only be called by the user or the council
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_validator())]
        pub fn remove_validator(
            origin: OriginFor<T>,
            validator_account: <T::Lookup as StaticLookup>::Source,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin.clone())?;
            Self::_remove_validator(who, validator_account)
        }
    }
}
