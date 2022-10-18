#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "128"]

use pallet_tfgrid::pallet::{InterfaceOf, PubConfigOf};
use sp_runtime::traits::Hash;
use sp_std::prelude::*;

use frame_support::{
    dispatch::{
        DispatchError, DispatchResult, DispatchResultWithPostInfo, Dispatchable, PostDispatchInfo,
    },
    ensure,
    traits::{EnsureOrigin, Get},
    weights::{GetDispatchInfo, Weight},
};
use tfchain_support::{
    constants, resources,
    traits::{ChangeNode, Tfgrid},
    types::{Node, Resources},
};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

mod proposal;

pub use weights::WeightInfo;

/// Simple index type for proposal counting.
pub type ProposalIndex = u32;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use pallet_tfgrid::farm::FarmName;
    use pallet_tfgrid::pub_ip::{GatewayIP, PublicIP};
    use sp_std::convert::TryInto;
    use tfchain_support::types::PublicIP as SupportPublicIP;

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + pallet_membership::Config<pallet_membership::Instance1>
        + pallet_tfgrid::Config
    {
        /// Because this pallet emits events, it depends on the runtime's definition of an event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type CouncilOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        /// The outer call dispatch type.
        type Proposal: Parameter
            + Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
            + From<frame_system::Call<Self>>
            + GetDispatchInfo;

        /// The time-out for council motions.
        type MotionDuration: Get<Self::BlockNumber>;

        /// The minimum amount of vetos to dissaprove a proposal
        type MinVetos: Get<u32>;

        type Tfgrid: Tfgrid<
            Self::AccountId,
            FarmName<Self>,
            SupportPublicIP<PublicIP<Self>, GatewayIP<Self>>,
        >;
        type NodeChanged: ChangeNode<PubConfigOf<Self>, InterfaceOf<Self>>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
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
        StorageMap<_, Identity, T::Hash, proposal::DaoProposal<ProposalIndex>, OptionQuery>;

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
        proposal::DaoVotes<ProposalIndex, T::BlockNumber, T::AccountId>,
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
        WrongProposalWeight,
        TooEarly,
        TimeLimitReached,
        OngoingVoteAndTresholdStillNotMet,
        FarmHasNoNodes,
        InvalidProposalDuration,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight((<T as pallet::Config>::WeightInfo::propose(), DispatchClass::Operational))]
        pub fn propose(
            origin: OriginFor<T>,
            #[pallet::compact] threshold: u32,
            action: Box<<T as Config>::Proposal>,
            description: Vec<u8>,
            link: Vec<u8>,
            duration: Option<T::BlockNumber>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::is_council_member(who.clone())?;

            let proposal_hash = T::Hashing::hash_of(&action);
            ensure!(
                !<ProposalOf<T>>::contains_key(proposal_hash),
                Error::<T>::DuplicateProposal
            );

            let now = frame_system::Pallet::<T>::block_number();
            let mut end = now + T::MotionDuration::get();
            if let Some(motion_duration) = duration {
                ensure!(
                    motion_duration < T::BlockNumber::from(constants::time::DAYS * 30),
                    Error::<T>::InvalidProposalDuration
                );
                end = now + motion_duration;
            }

            let index = Self::proposal_count();
            <ProposalCount<T>>::mutate(|i| *i += 1);
            <ProposalOf<T>>::insert(proposal_hash, *action);

            let p = proposal::DaoProposal {
                index,
                description,
                link,
            };
            <Proposals<T>>::insert(proposal_hash, p);

            let votes = {
                proposal::DaoVotes {
                    index,
                    threshold,
                    ayes: vec![],
                    nays: vec![],
                    end,
                    vetos: vec![],
                }
            };
            <Voting<T>>::insert(proposal_hash, votes);

            let mut active_proposal_hashes = <ProposalList<T>>::get();
            active_proposal_hashes.push(proposal_hash);
            <ProposalList<T>>::set(active_proposal_hashes);

            Self::deposit_event(Event::Proposed {
                account: who,
                proposal_index: index,
                proposal_hash,
                threshold,
            });

            Ok(().into())
        }

        #[pallet::weight((<T as pallet::Config>::WeightInfo::vote(), DispatchClass::Operational))]
        pub fn vote(
            origin: OriginFor<T>,
            farm_id: u32,
            proposal_hash: T::Hash,
            approve: bool,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            ensure!(
                T::Tfgrid::is_farm_owner(farm_id, who.clone()),
                Error::<T>::NotAuthorizedToVote
            );

            let stored_proposal =
                <Proposals<T>>::get(proposal_hash).ok_or(Error::<T>::ProposalMissing)?;

            let mut voting = Self::voting(proposal_hash).ok_or(Error::<T>::ProposalMissing)?;
            ensure!(
                voting.index == stored_proposal.index,
                Error::<T>::WrongIndex
            );

            // Don't allow votes after the time limit reached
            ensure!(
                frame_system::Pallet::<T>::block_number() <= voting.end,
                Error::<T>::TimeLimitReached
            );

            let position_yes = voting.ayes.iter().position(|a| a.farm_id == farm_id);
            let position_no = voting.nays.iter().position(|a| a.farm_id == farm_id);

            // Detects first vote of the member in the motion
            let is_account_voting_first_time = position_yes.is_none() && position_no.is_none();

            if approve {
                if position_yes.is_none() {
                    voting.ayes.push(proposal::VoteWeight {
                        farm_id: farm_id,
                        weight: Self::get_vote_weight(farm_id)?,
                    });
                } else {
                    return Err(Error::<T>::DuplicateVote.into());
                }
                if let Some(pos) = position_no {
                    voting.nays.swap_remove(pos);
                }
            } else {
                if position_no.is_none() {
                    voting.nays.push(proposal::VoteWeight {
                        farm_id: farm_id,
                        weight: Self::get_vote_weight(farm_id)?,
                    });
                } else {
                    return Err(Error::<T>::DuplicateVote.into());
                }
                if let Some(pos) = position_yes {
                    voting.ayes.swap_remove(pos);
                }
            }

            let yes_votes = voting.ayes.len() as u32;
            let no_votes = voting.nays.len() as u32;
            Self::deposit_event(Event::Voted {
                account: who,
                proposal_hash,
                voted: approve,
                yes: yes_votes,
                no: no_votes,
            });

            Voting::<T>::insert(&proposal_hash, voting);

            if is_account_voting_first_time {
                Ok(Pays::No.into())
            } else {
                Ok(Pays::Yes.into())
            }
        }

        #[pallet::weight((<T as pallet::Config>::WeightInfo::vote(), DispatchClass::Operational))]
        pub fn veto(origin: OriginFor<T>, proposal_hash: T::Hash) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::is_council_member(who.clone())?;

            let stored_proposal =
                <Proposals<T>>::get(proposal_hash).ok_or(Error::<T>::ProposalMissing)?;

            let mut voting = Self::voting(proposal_hash).ok_or(Error::<T>::ProposalMissing)?;
            ensure!(
                voting.index == stored_proposal.index,
                Error::<T>::WrongIndex
            );

            // push vote to vetos
            voting.vetos.push(who.clone());

            Self::deposit_event(Event::CouncilMemberVeto { proposal_hash, who });

            if voting.vetos.len() as u32 >= T::MinVetos::get() {
                Self::deposit_event(Event::ClosedByCouncil {
                    proposal_hash,
                    vetos: voting.vetos,
                });
                Self::do_disapprove_proposal(proposal_hash);
                return Ok(Pays::No.into());
            }

            Voting::<T>::insert(&proposal_hash, voting);

            return Ok(Pays::No.into());
        }

        #[pallet::weight((<T as pallet::Config>::WeightInfo::close(), DispatchClass::Operational))]
        pub fn close(
            origin: OriginFor<T>,
            proposal_hash: T::Hash,
            #[pallet::compact] proposal_index: ProposalIndex,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::is_council_member(who)?;

            let voting = Self::voting(&proposal_hash).ok_or(Error::<T>::ProposalMissing)?;
            ensure!(voting.index == proposal_index, Error::<T>::WrongIndex);

            let no_votes = voting.nays.len() as u32;
            let yes_votes = voting.ayes.len() as u32;

            // Only allow actual closing of the proposal after the voting threshold is met or voting period has ended
            ensure!(
                (no_votes + yes_votes) >= voting.threshold
                    || frame_system::Pallet::<T>::block_number() >= voting.end,
                Error::<T>::OngoingVoteAndTresholdStillNotMet
            );

            let total_aye_weight: u64 = voting.ayes.iter().map(|y| y.weight).sum();
            let total_naye_weight: u64 = voting.nays.iter().map(|y| y.weight).sum();

            let approved = total_aye_weight > total_naye_weight;

            if approved {
                let proposal = Self::validate_and_get_proposal(&proposal_hash)?;
                Self::deposit_event(Event::Closed {
                    proposal_hash,
                    yes: yes_votes,
                    yes_weight: total_aye_weight,
                    no: no_votes,
                    no_weight: total_naye_weight,
                });
                let _proposal_weight = Self::do_approve_proposal(proposal_hash, proposal);
                return Ok(Pays::No.into());
            }

            Self::deposit_event(Event::Closed {
                proposal_hash,
                yes: yes_votes,
                yes_weight: total_aye_weight,
                no: no_votes,
                no_weight: total_naye_weight,
            });
            Self::do_disapprove_proposal(proposal_hash);
            return Ok(Pays::No.into());
        }
    }
}

/// Return the weight of a dispatch call result as an `Option`.
///
/// Will return the weight regardless of what the state of the result is.
fn get_result_weight(result: DispatchResultWithPostInfo) -> Option<Weight> {
    match result {
        Ok(post_info) => post_info.actual_weight,
        Err(err) => err.post_info.actual_weight,
    }
}

impl<T: Config> Pallet<T> {
    // If a farmer does not have any nodes attached to it's farm, an error is returned
    pub fn get_vote_weight(farm_id: u32) -> Result<u64, DispatchError> {
        let farm_weight = FarmWeight::<T>::get(farm_id);
        ensure!(farm_weight > 0, Error::<T>::FarmHasNoNodes);
        Ok(farm_weight)
    }

    /// Ensure that the right proposal bounds were passed and get the proposal from storage.
    ///
    /// Checks the length in storage via `storage::read` which adds an extra `size_of::<u32>() == 4`
    /// to the length.
    fn validate_and_get_proposal(hash: &T::Hash) -> Result<<T as Config>::Proposal, DispatchError> {
        let proposal = ProposalOf::<T>::get(hash).ok_or(Error::<T>::ProposalMissing)?;
        Ok(proposal)
    }

    /// Weight:
    /// If `approved`:
    /// - the weight of `proposal` preimage.
    /// - two events deposited.
    /// - two removals, one mutation.
    /// - computation and i/o `O(P + L)` where:
    ///   - `P` is number of active proposals,
    ///   - `L` is the encoded length of `proposal` preimage.
    ///
    /// If not `approved`:
    /// - one event deposited.
    /// Two removals, one mutation.
    /// Computation and i/o `O(P)` where:
    /// - `P` is number of active proposals
    fn do_approve_proposal(proposal_hash: T::Hash, proposal: <T as Config>::Proposal) -> Weight {
        Self::deposit_event(Event::Approved { proposal_hash });

        let dispatch_weight = proposal.get_dispatch_info().weight;
        // let origin = RawOrigin::Members(yes_votes, seats).into();
        let result = proposal.dispatch(frame_system::RawOrigin::Root.into());
        Self::deposit_event(Event::Executed {
            proposal_hash,
            result: result.map(|_| ()).map_err(|e| e.error),
        });
        // default to the dispatch info weight for safety
        let proposal_weight = get_result_weight(result).unwrap_or(dispatch_weight); // P1

        Self::remove_proposal(proposal_hash);
        proposal_weight
    }

    fn do_disapprove_proposal(proposal_hash: T::Hash) {
        // disapproved
        Self::deposit_event(Event::Disapproved { proposal_hash });
        Self::remove_proposal(proposal_hash)
    }

    // Removes a proposal from the pallet, cleaning up votes and the vector of proposals.
    fn remove_proposal(proposal_hash: T::Hash) {
        // remove proposal and vote
        ProposalOf::<T>::remove(&proposal_hash);
        Voting::<T>::remove(&proposal_hash);
        Proposals::<T>::remove(&proposal_hash);
        let mut active_proposals = ProposalList::<T>::get();
        active_proposals.retain(|hash| hash != &proposal_hash);
        ProposalList::<T>::set(active_proposals);
    }

    pub fn get_node_weight(resources: Resources) -> u64 {
        let cu = resources::get_cu(resources);
        let su = resources::get_su(resources);
        cu * 2 + su
    }

    fn is_council_member(who: T::AccountId) -> DispatchResultWithPostInfo {
        let council_members =
            pallet_membership::Pallet::<T, pallet_membership::Instance1>::members();

        ensure!(council_members.contains(&who), Error::<T>::NotCouncilMember,);

        Ok(().into())
    }
}

impl<T: Config> ChangeNode<PubConfigOf<T>, InterfaceOf<T>> for Pallet<T> {
    fn node_changed(
        old_node: Option<&Node<PubConfigOf<T>, InterfaceOf<T>>>,
        new_node: &Node<PubConfigOf<T>, InterfaceOf<T>>,
    ) {
        let new_node_weight = Self::get_node_weight(new_node.resources.total_resources);
        match old_node {
            Some(node) => {
                let old_node_weight = Self::get_node_weight(node.resources.total_resources);

                if node.farm_id != new_node.farm_id {
                    let mut old_farm_weight = FarmWeight::<T>::get(node.farm_id);
                    old_farm_weight = old_farm_weight.checked_sub(old_node_weight).unwrap_or(0);
                    FarmWeight::<T>::insert(node.farm_id, old_farm_weight);

                    let mut new_farm_weight = FarmWeight::<T>::get(new_node.farm_id);
                    new_farm_weight += new_node_weight;
                    FarmWeight::<T>::insert(new_node.farm_id, new_farm_weight);
                } else {
                    // Node got updated
                    let mut farm_weight = FarmWeight::<T>::get(node.farm_id);
                    farm_weight = farm_weight.checked_sub(old_node_weight).unwrap_or(0);
                    farm_weight += new_node_weight;
                    FarmWeight::<T>::insert(node.farm_id, farm_weight);
                }
            }
            None => {
                // New node got added, just add the weight to the farmweight
                let mut farm_weight = FarmWeight::<T>::get(new_node.farm_id);
                farm_weight += new_node_weight;
                FarmWeight::<T>::insert(new_node.farm_id, farm_weight);
            }
        };
    }

    fn node_deleted(node: &Node<PubConfigOf<T>, InterfaceOf<T>>) {
        let node_weight = Self::get_node_weight(node.resources.total_resources);
        let mut farm_weight = FarmWeight::<T>::get(node.farm_id);
        farm_weight = farm_weight.checked_sub(node_weight).unwrap_or(0);
        FarmWeight::<T>::insert(node.farm_id, farm_weight);
    }
}
