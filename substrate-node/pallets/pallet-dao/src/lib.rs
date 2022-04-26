#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "128"]

use sp_std::prelude::*;
use sp_io::storage;
use sp_runtime::{traits::Hash};

use frame_support::{
	dispatch::{DispatchError, DispatchResult, DispatchResultWithPostInfo, Dispatchable, PostDispatchInfo},
	ensure,
	traits::{
		EnsureOrigin, Get,
	},
	weights::{GetDispatchInfo, Weight},
};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod proposal;

/// Simple index type for proposal counting.
pub type ProposalIndex = u32;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

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
		type ProposalCall: Parameter
			+ Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
			+ From<frame_system::Call<Self>>
			+ GetDispatchInfo;

		/// The time-out for council motions.
		type MotionDuration: Get<Self::BlockNumber>;

		/// Maximum number of proposals allowed to be active in parallel.
		type MaxProposals: Get<ProposalIndex>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// The hashes of the active proposals.
	#[pallet::storage]
	#[pallet::getter(fn proposals_list_hashes)]
	pub type ProposalList<T: Config> =
		StorageValue<_, Vec<T::Hash>, ValueQuery>;

	/// A map that indexes a hash to an active proposal object.
	#[pallet::storage]
	#[pallet::getter(fn proposal_list)]
	pub type Proposals<T: Config> =
		StorageMap<_, Identity, T::Hash, proposal::Proposal<ProposalIndex>, OptionQuery>;

	// Actual proposal for a given hash, if it's current.
	#[pallet::storage]
	#[pallet::getter(fn proposal_of)]
	pub type ProposalOf<T: Config> =
		StorageMap<_, Identity, T::Hash, <T as Config>::ProposalCall, OptionQuery>;

	/// Votes on a given proposal, if it is ongoing.
	#[pallet::storage]
	#[pallet::getter(fn voting)]
	pub type Voting<T: Config> =
		StorageMap<_, Identity, T::Hash, proposal::Votes<ProposalIndex, T::AccountId, T::BlockNumber>, OptionQuery>;

	/// Proposals so far.
	#[pallet::storage]
	#[pallet::getter(fn proposal_count)]
	pub type ProposalCount<T> = StorageValue<_, u32, ValueQuery>;

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
			proposal_hash: T::Hash,
			threshold: u32,
		},
		/// A motion was approved by the required threshold.
		Approved { proposal_hash: T::Hash },
		/// A motion was not approved by the required threshold.
		Disapproved { proposal_hash: T::Hash },
		/// A motion was executed; result will be `Ok` if it returned without error.
		Executed { proposal_hash: T::Hash, result: DispatchResult },
		/// A single member did some action; result will be `Ok` if it returned without error.
		MemberExecuted { proposal_hash: T::Hash, result: DispatchResult },
		/// A proposal_hash was closed because its threshold was reached or after its duration was up.
		Closed { proposal_hash: T::Hash, yes: u32, no: u32 },
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
		ProposalNotExists,
		ProposalMissing,
		WrongIndex,
		DuplicateVote,
		WrongProposalWeight
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn propose(
			origin: OriginFor<T>,
			#[pallet::compact] threshold: u32,
			action: Box<<T as Config>::ProposalCall>,
			description: Vec<u8>,
			link: Vec<u8>,
			#[pallet::compact] length_bound: u32,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let council_members = pallet_membership::Module::<T, pallet_membership::Instance1>::members();
			ensure!(council_members.contains(&who), Error::<T>::NotCouncilMember);

			let proposal_len = action.using_encoded(|x| x.len());
			ensure!(proposal_len <= length_bound as usize, Error::<T>::WrongProposalLength);
			let proposal_hash = T::Hashing::hash_of(&action);
			ensure!(
				!<ProposalOf<T>>::contains_key(proposal_hash),
				Error::<T>::DuplicateProposal
			);

			let index = Self::proposal_count();
			<ProposalCount<T>>::mutate(|i| *i += 1);
			<ProposalOf<T>>::insert(proposal_hash, *action);
			
			let p = proposal::Proposal {
				index,
				description,
				link
			};
			<Proposals<T>>::insert(proposal_hash, p);

			let votes = {
				let end = frame_system::Pallet::<T>::block_number() + T::MotionDuration::get();
				proposal::Votes { index, threshold, ayes: vec![], nays: vec![], end }
			};
			<Voting<T>>::insert(proposal_hash, votes);

			let mut active_proposal_hashes = <ProposalList<T>>::get();
			active_proposal_hashes.push(proposal_hash);
			<ProposalList<T>>::set(active_proposal_hashes);

			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn vote(
			origin: OriginFor<T>,
			farm_id: u32,
			proposal_hash: T::Hash,
			approve: bool
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let farm = pallet_tfgrid::Module::<T>::farms(farm_id);
			ensure!(farm.id != 0, Error::<T>::FarmNotExists);

			let twin = pallet_tfgrid::Module::<T>::twins(farm.twin_id);
			ensure!(twin.account_id == who, Error::<T>::NotAuthorizedToVote);

			ensure!(<Proposals<T>>::contains_key(proposal_hash), Error::<T>::ProposalNotExists);
			let stored_proposal = <Proposals<T>>::get(proposal_hash).ok_or(Error::<T>::ProposalMissing)?;

			let mut voting = Self::voting(proposal_hash).ok_or(Error::<T>::ProposalMissing)?;
			ensure!(voting.index == stored_proposal.index, Error::<T>::WrongIndex);

			let position_yes = voting.ayes.iter().position(|a| a.who == who);
			let position_no = voting.nays.iter().position(|a| a.who == who);

			// Detects first vote of the member in the motion
			let is_account_voting_first_time = position_yes.is_none() && position_no.is_none();

			if approve {
				if position_yes.is_none() {
					voting.ayes.push(proposal::VoteWeight{
						who: who.clone(),
						weight: Self::get_vote_weight(&who)
					});
				} else {
					return Err(Error::<T>::DuplicateVote.into())
				}
				if let Some(pos) = position_no {
					voting.nays.swap_remove(pos);
				}
			} else {
				if position_no.is_none() {
					voting.nays.push(proposal::VoteWeight{
						who: who.clone(),
						weight: Self::get_vote_weight(&who)
					});
				} else {
					return Err(Error::<T>::DuplicateVote.into())
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

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn close(
			origin: OriginFor<T>,
			proposal_hash: T::Hash,
			#[pallet::compact] proposal_index: ProposalIndex,
			#[pallet::compact] proposal_weight_bound: Weight,
			#[pallet::compact] length_bound: u32,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_signed(origin)?;

			let voting = Self::voting(&proposal_hash).ok_or(Error::<T>::ProposalMissing)?;
			ensure!(voting.index == proposal_index, Error::<T>::WrongIndex);

			let mut no_votes = voting.nays.len() as u32;
			let mut yes_votes = voting.ayes.len() as u32;
			// let seats = Self::members().len() as u32;

			let total_aye_weight: u32 = voting.ayes.iter().map(|y| Self::get_vote_weight(&y.who)).sum();
			let total_naye_weight: u32 = voting.nays.iter().map(|y| Self::get_vote_weight(&y.who)).sum();

			let approved = total_aye_weight >= voting.threshold;
			let disapproved = total_naye_weight >= voting.threshold;
			// let approved = yes_votes >= voting.threshold;
			// let disapproved = seats.saturating_sub(no_votes) < voting.threshold;
			// Allow (dis-)approving the proposal as soon as there are enough votes.
			if approved {
				let proposal = Self::validate_and_get_proposal(
					&proposal_hash,
					length_bound,
					proposal_weight_bound,
				)?;
				Self::deposit_event(Event::Closed { proposal_hash, yes: yes_votes, no: no_votes });
				let proposal_weight =
					Self::do_approve_proposal(yes_votes, proposal_hash, proposal);
				return Ok(Pays::No.into())
			} else if disapproved {
				Self::deposit_event(Event::Closed { proposal_hash, yes: yes_votes, no: no_votes });
				Self::do_disapprove_proposal(proposal_hash);
				return Ok(Pays::No.into())
			}

			// // Only allow actual closing of the proposal after the voting period has ended.
			// ensure!(
			// 	frame_system::Pallet::<T>::block_number() >= voting.end,
			// 	Error::<T>::TooEarly
			// );

			// let prime_vote = Self::prime().map(|who| voting.ayes.iter().any(|a| a == &who));

			// // default voting strategy.
			// let default = T::DefaultVote::default_vote(prime_vote, yes_votes, no_votes, seats);

			// let abstentions = seats - (yes_votes + no_votes);
			// match default {
			// 	true => yes_votes += abstentions,
			// 	false => no_votes += abstentions,
			// }
			// let approved = yes_votes >= voting.threshold;

			// if approved {
			// 	let (proposal, len) = Self::validate_and_get_proposal(
			// 		&proposal_hash,
			// 		length_bound,
			// 		proposal_weight_bound,
			// 	)?;
			// 	Self::deposit_event(Event::Closed { proposal_hash, yes: yes_votes, no: no_votes });
			// 	let (proposal_weight, proposal_count) =
			// 		Self::do_approve_proposal(seats, yes_votes, proposal_hash, proposal);
			// 	Ok((
			// 		Some(
			// 			T::WeightInfo::close_approved(len as u32, seats, proposal_count)
			// 				.saturating_add(proposal_weight),
			// 		),
			// 		Pays::Yes,
			// 	)
			// 		.into())
			// } else {
			// 	Self::deposit_event(Event::Closed { proposal_hash, yes: yes_votes, no: no_votes });
			// 	let proposal_count = Self::do_disapprove_proposal(proposal_hash);
			// 	Ok((Some(T::WeightInfo::close_disapproved(seats, proposal_count)), Pays::No).into())
			// }
			Ok(().into())
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
	pub fn get_vote_weight(_who: &T::AccountId) -> u32 {
		1
	}

	/// Ensure that the right proposal bounds were passed and get the proposal from storage.
	///
	/// Checks the length in storage via `storage::read` which adds an extra `size_of::<u32>() == 4`
	/// to the length.
	fn validate_and_get_proposal(
		hash: &T::Hash,
		length_bound: u32,
		weight_bound: Weight,
	) -> Result<<T as Config>::ProposalCall, DispatchError> {
		let key = ProposalOf::<T>::hashed_key_for(hash);
		// read the length of the proposal storage entry directly
		let proposal_len =
			storage::read(&key, &mut [0; 0], 0).ok_or(Error::<T>::ProposalMissing)?;
		ensure!(proposal_len <= length_bound, Error::<T>::WrongProposalLength);
		let proposal = ProposalOf::<T>::get(hash).ok_or(Error::<T>::ProposalMissing)?;
		let proposal_weight = proposal.get_dispatch_info().weight;
		ensure!(proposal_weight <= weight_bound, Error::<T>::WrongProposalWeight);
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
	fn do_approve_proposal(
		yes_votes: u32,
		proposal_hash: T::Hash,
		proposal: <T as Config>::ProposalCall,
	) -> Weight {
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
	}
}