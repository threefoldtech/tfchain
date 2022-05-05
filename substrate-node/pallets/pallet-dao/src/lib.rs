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
use pallet_tfgrid::types as pallet_tfgrid_types;
use tfchain_support::traits::Tfgrid;

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
		type Proposal: Parameter
			+ Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
			+ From<frame_system::Call<Self>>
			+ GetDispatchInfo;

		/// The time-out for council motions.
		type MotionDuration: Get<Self::BlockNumber>;

		/// Maximum number of proposals allowed to be active in parallel.
		type MaxProposals: Get<ProposalIndex>;

		type Tfgrid: Tfgrid<Self::AccountId>;
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
		StorageMap<_, Identity, T::Hash, proposal::DaoProposal<ProposalIndex>, OptionQuery>;

	// Actual proposal for a given hash, if it's current.
	#[pallet::storage]
	#[pallet::getter(fn proposal_of)]
	pub type ProposalOf<T: Config> =
		StorageMap<_, Identity, T::Hash, <T as Config>::Proposal, OptionQuery>;

	/// Votes on a given proposal, if it is ongoing.
	#[pallet::storage]
	#[pallet::getter(fn voting)]
	pub type Voting<T: Config> =
		StorageMap<_, Identity, T::Hash, proposal::Votes<ProposalIndex, T::BlockNumber>, OptionQuery>;

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
			proposal_index: ProposalIndex,
			proposal_hash: T::Hash,
			threshold: u32,
		},
		/// A motion was approved by the required threshold.
		Approved { proposal_hash: T::Hash },
		/// A motion was not approved by the required threshold.
		Disapproved { proposal_hash: T::Hash },
		/// A motion was executed; result will be `Ok` if it returned without error.
		Executed { proposal_hash: T::Hash, result: DispatchResult },
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
		WrongProposalWeight,
		TooEarly,
		TimeLimitReached,
		VoteThresholdNotMet,
		FarmHasNoNodes,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn propose(
			origin: OriginFor<T>,
			#[pallet::compact] threshold: u32,
			action: Box<<T as Config>::Proposal>,
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
			
			let p = proposal::DaoProposal {
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

			Self::deposit_event(Event::Proposed {
				account: who,
				proposal_index: index,
				proposal_hash,
				threshold,
			});

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

			let f = <T as Config>::Tfgrid::get_farm(farm_id);
			println!("farm: {:?}", f);
			println!("farm: {:?}", f.id);

			let farm = pallet_tfgrid::Module::<T>::farms(farm_id);
			ensure!(farm.id != 0, Error::<T>::FarmNotExists);

			let twin = pallet_tfgrid::Module::<T>::twins(farm.twin_id);
			ensure!(twin.account_id == who, Error::<T>::NotAuthorizedToVote);

			ensure!(<Proposals<T>>::contains_key(proposal_hash), Error::<T>::ProposalNotExists);
			let stored_proposal = <Proposals<T>>::get(proposal_hash).ok_or(Error::<T>::ProposalMissing)?;

			let mut voting = Self::voting(proposal_hash).ok_or(Error::<T>::ProposalMissing)?;
			ensure!(voting.index == stored_proposal.index, Error::<T>::WrongIndex);

			// Don't allow votes after the time limit reached
			ensure!(
				frame_system::Pallet::<T>::block_number() <= voting.end,
				Error::<T>::TimeLimitReached
			);

			let position_yes = voting.ayes.iter().position(|a| a.farm_id == farm.id);
			let position_no = voting.nays.iter().position(|a| a.farm_id == farm.id);

			// Detects first vote of the member in the motion
			let is_account_voting_first_time = position_yes.is_none() && position_no.is_none();

			if approve {
				if position_yes.is_none() {
					voting.ayes.push(proposal::VoteWeight{
						farm_id: farm.id,
						weight: Self::get_vote_weight(farm.id)?
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
						farm_id: farm.id,
						weight: Self::get_vote_weight(farm.id)?
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
			
			// Only allow actual closing of the proposal after the voting period has ended.
			ensure!(
				frame_system::Pallet::<T>::block_number() >= voting.end,
				Error::<T>::TooEarly
			);

			let no_votes = voting.nays.len() as u32;
			let yes_votes = voting.ayes.len() as u32;

			ensure!(
				(no_votes + yes_votes) >= voting.threshold,
				Error::<T>::VoteThresholdNotMet
			);

			let total_aye_weight: u64 = voting.ayes.iter().map(|y| y.weight).sum();
			let total_naye_weight: u64 = voting.nays.iter().map(|y| y.weight).sum();

			let approved = total_aye_weight > total_naye_weight;

			if approved {
				let proposal = Self::validate_and_get_proposal(
					&proposal_hash,
					length_bound,
					proposal_weight_bound,
				)?;
				Self::deposit_event(Event::Closed { proposal_hash, yes: yes_votes, no: no_votes });
				let _proposal_weight =
					Self::do_approve_proposal(proposal_hash, proposal);
				return Ok(Pays::No.into())
			}

			Self::deposit_event(Event::Closed { proposal_hash, yes: yes_votes, no: no_votes });
			Self::do_disapprove_proposal(proposal_hash);
			return Ok(Pays::No.into())
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

const ONE_THOUSAND: u128 = 1_000;
const GIB: u128 = 1024 * 1024 * 1024;

impl<T: Config> Pallet<T> {
	// If a farmer does not have any nodes attached to it's farm, an error is returned
	pub fn get_vote_weight(farm_id: u32) -> Result<u64, DispatchError> {
		let nodes_ids = pallet_tfgrid::Module::<T>::node_ids_by_farm_id(farm_id);
		ensure!(nodes_ids.len() > 0, Error::<T>::FarmHasNoNodes);

		let mut total_weight = 0;
		for id in nodes_ids {
			let node = pallet_tfgrid::Module::<T>::nodes(id);
			let cu = Self::get_cu(node.resources);
			let su = Self::get_su(node.resources);
			
			let calculated_cu = 2*(cu as u128 / GIB/ ONE_THOUSAND);
			let calculated_su = su as u128 / ONE_THOUSAND;
			total_weight += calculated_cu as u64 + calculated_su as u64;
		}

		Ok(total_weight)
	}

	/// Ensure that the right proposal bounds were passed and get the proposal from storage.
	///
	/// Checks the length in storage via `storage::read` which adds an extra `size_of::<u32>() == 4`
	/// to the length.
	fn validate_and_get_proposal(
		hash: &T::Hash,
		length_bound: u32,
		weight_bound: Weight,
	) -> Result<<T as Config>::Proposal, DispatchError> {
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
		proposal_hash: T::Hash,
		proposal: <T as Config>::Proposal,
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
		Proposals::<T>::remove(&proposal_hash);
		let mut active_proposals = ProposalList::<T>::get();
		active_proposals.retain(|hash| hash != &proposal_hash);
		ProposalList::<T>::set(active_proposals);
	}

	pub fn get_cu(resources: pallet_tfgrid_types::Resources) -> u64 {
        let cru_min = resources.cru as u128 * 2 * GIB * ONE_THOUSAND;
		let mru_min = (resources.mru as u128 - 1 * GIB) * ONE_THOUSAND / 4;
		let sru_min = resources.sru as u128 * ONE_THOUSAND / 50;

		if cru_min < mru_min && cru_min < sru_min {
			cru_min as u64
		} else if mru_min < cru_min && mru_min < sru_min {
			mru_min as u64
		} else if sru_min < cru_min && sru_min < mru_min {
			sru_min as u64
		} else {
			0
		}
    }

	pub fn get_su(resources: pallet_tfgrid_types::Resources) -> u64 {
		let su = resources.hru as u128 * ONE_THOUSAND / 1200 + resources.sru as u128 * ONE_THOUSAND / 250;
		let calculated_su = su / GIB;
		calculated_su as u64
	}
}