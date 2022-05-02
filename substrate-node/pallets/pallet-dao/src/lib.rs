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
use substrate_fixed::types::U64F64;

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
		WrongProposalWeight,
		TooEarly,
		TimeLimitReached,
		VoteThresholdNotMet
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

			let position_yes = voting.ayes.iter().position(|a| a.who == who);
			let position_no = voting.nays.iter().position(|a| a.who == who);

			// Detects first vote of the member in the motion
			let is_account_voting_first_time = position_yes.is_none() && position_no.is_none();

			if approve {
				if position_yes.is_none() {
					voting.ayes.push(proposal::VoteWeight{
						who: who.clone(),
						weight: Self::get_vote_weight(farm.id)
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
						weight: Self::get_vote_weight(farm.id)
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

impl<T: Config> Pallet<T> {
	pub fn get_vote_weight(farm_id: u32) -> u64 {
		let nodes_ids = pallet_tfgrid::Module::<T>::node_ids_by_farm_id(farm_id);
		let farm = pallet_tfgrid::Module::<T>::farms(farm_id);
		let pricing_policy = pallet_tfgrid::Module::<T>::pricing_policies(farm.pricing_policy_id);
		let mut total_weight = 1;

		for id in nodes_ids {
			let node = pallet_tfgrid::Module::<T>::nodes(id);
			let cu = Self::get_cu(node.resources, &pricing_policy);
			let su = 0;

			total_weight += 2*cu + su 
		}

		total_weight
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
	}

	pub fn get_cu(resources: pallet_tfgrid_types::Resources, pricing_policy: &pallet_tfgrid_types::PricingPolicy<T::AccountId>) -> u64 {
        let mru = U64F64::from_num(resources.mru) / pricing_policy.cu.factor();
        let cru = U64F64::from_num(resources.cru);

        let mru_used_1 = mru / 4;
        let cru_used_1 = cru / 2;
        let cu1 = if mru_used_1 > cru_used_1 {
            mru_used_1
        } else {
            cru_used_1
        };

        let mru_used_2 = mru / 8;
        let cru_used_2 = cru;
        let cu2 = if mru_used_2 > cru_used_2 {
            mru_used_2
        } else {
            cru_used_2
        };

        let mru_used_3 = mru / 2;
        let cru_used_3 = cru / 4;
        let cu3 = if mru_used_3 > cru_used_3 {
            mru_used_3
        } else {
            cru_used_3
        };

        let mut cu = if cu1 > cu2 { cu2 } else { cu1 };

        cu = if cu > cu3 { cu3 } else { cu };

        cu.ceil().to_num::<u64>()
    }

	// fn get_su(hru: u64, sru: u64) -> u64 {
	// 	let pricing_policy = pallet_tfgrid::Module::<T>::pricing_policies(1);
	// 	let hru = hru as u128 / pricing_policy.su.factor();
    //     let sru = sru as u128 / pricing_policy.su.factor();

	// 	(hru / 1200 + sru / 200).try_into().unwrap_or(0)
	// }
}