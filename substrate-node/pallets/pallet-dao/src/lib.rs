#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "128"]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

mod dao;
mod proposal;

pub use pallet::*;
/// Simple index type for proposal counting.

#[frame_support::pallet]
pub mod pallet {
    use crate::proposal;
    use crate::proposal::ProposalIndex;
    use crate::weights::WeightInfo;
    use frame_support::{
        dispatch::{DispatchResult, DispatchResultWithPostInfo, GetDispatchInfo, PostDispatchInfo},
        pallet_prelude::*,
        traits::{EnsureOrigin, Get},
    };
    use frame_system::pallet_prelude::*;
    use pallet_tfgrid::farm::FarmName;
    use sp_runtime::traits::Dispatchable;
    use sp_std::prelude::*;
    use tfchain_support::traits::Tfgrid;

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + pallet_membership::Config<pallet_membership::Instance1>
        + pallet_tfgrid::Config
    {
        /// Because this pallet emits events, it depends on the runtime's definition of an event
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type CouncilOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// The outer call dispatch type.
        type Proposal: Parameter
            + Dispatchable<RuntimeOrigin = Self::RuntimeOrigin, PostInfo = PostDispatchInfo>
            + From<frame_system::Call<Self>>
            + GetDispatchInfo;

        /// The time-out for council motions.
        type MotionDuration: Get<BlockNumberFor<Self>>;

        /// The minimum amount of vetos to dissaprove a proposal
        type MinVetos: Get<u32>;

        type Tfgrid: Tfgrid<Self::AccountId, FarmName<Self>>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    /// The hashes of the active proposals.
    #[pallet::storage]
    #[pallet::getter(fn proposals_list_hashes)]
    pub type ProposalList<T: Config> = StorageValue<_, Vec<T::Hash>, ValueQuery>;

    /// A map that indexes a hash to an active proposal object.
    #[pallet::storage]
    #[pallet::getter(fn proposal_list)]
    pub type Proposals<T: Config> =
        StorageMap<_, Identity, T::Hash, proposal::DaoProposal, OptionQuery>;

    // Actual proposal for a given hash, if it's current.
    #[pallet::storage]
    #[pallet::getter(fn proposal_of)]
    pub type ProposalOf<T: Config> =
        StorageMap<_, Identity, T::Hash, <T as Config>::Proposal, OptionQuery>;

    /// Votes on a given proposal, if it is ongoing.
    #[pallet::storage]
    #[pallet::getter(fn voting)]
    pub type Voting<T: Config> = StorageMap<
        _,
        Identity,
        T::Hash,
        proposal::DaoVotes<BlockNumberFor<T>, T::AccountId>,
        OptionQuery,
    >;

    /// Proposals so far.
    #[pallet::storage]
    #[pallet::getter(fn proposal_count)]
    pub type ProposalCount<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn farm_weight)]
    pub type FarmWeight<T> = StorageMap<_, Identity, u32, u64, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Voted {
            account: T::AccountId,
            proposal_hash: T::Hash,
            voted: bool,
            yes: u32,
            no: u32,
        },
        /// A motion (given hash) has been proposed (by given account) with a threshold (given
        /// `MemberCount`).
        Proposed {
            account: T::AccountId,
            proposal_index: ProposalIndex,
            proposal_hash: T::Hash,
            threshold: u32,
        },
        /// A motion was approved by the required threshold.
        Approved { proposal_hash: T::Hash },
        /// A motion was not approved by the required threshold.
        Disapproved { proposal_hash: T::Hash },
        /// A motion was executed; result will be `Ok` if it returned without error.
        Executed {
            proposal_hash: T::Hash,
            result: DispatchResult,
        },
        /// A proposal_hash was closed because its threshold was reached or after its duration was up.
        Closed {
            proposal_hash: T::Hash,
            yes: u32,
            yes_weight: u64,
            no: u32,
            no_weight: u64,
        },
        ClosedByCouncil {
            proposal_hash: T::Hash,
            vetos: Vec<T::AccountId>,
        },
        CouncilMemberVeto {
            proposal_hash: T::Hash,
            who: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        NoneValue,
        StorageOverflow,
        FarmNotExists,
        NotCouncilMember,
        WrongProposalLength,
        DuplicateProposal,
        NotAuthorizedToVote,
        ProposalMissing,
        WrongIndex,
        DuplicateVote,
        DuplicateVeto,
        WrongProposalWeight,
        TooEarly,
        TimeLimitReached,
        OngoingVoteAndTresholdStillNotMet,
        FarmHasNoNodes,
        InvalidProposalDuration,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight((<T as Config>::WeightInfo::propose(), DispatchClass::Operational))]
        pub fn propose(
            origin: OriginFor<T>,
            #[pallet::compact] threshold: u32,
            action: Box<<T as Config>::Proposal>,
            description: Vec<u8>,
            link: Vec<u8>,
            duration: Option<BlockNumberFor<T>>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::_propose(who, threshold, action, description, link, duration)
        }

        #[pallet::call_index(1)]
        #[pallet::weight((<T as Config>::WeightInfo::vote(), DispatchClass::Operational))]
        pub fn vote(
            origin: OriginFor<T>,
            farm_id: u32,
            proposal_hash: T::Hash,
            approve: bool,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::_vote(who, farm_id, proposal_hash, approve)
        }

        #[pallet::call_index(2)]
        #[pallet::weight((<T as Config>::WeightInfo::veto(), DispatchClass::Operational))]
        pub fn veto(origin: OriginFor<T>, proposal_hash: T::Hash) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::_veto(who, proposal_hash)
        }

        #[pallet::call_index(3)]
        #[pallet::weight((<T as Config>::WeightInfo::close(), DispatchClass::Operational))]
        pub fn close(
            origin: OriginFor<T>,
            proposal_hash: T::Hash,
            #[pallet::compact] proposal_index: ProposalIndex,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::_close(who, proposal_hash, proposal_index)
        }
    }
}
