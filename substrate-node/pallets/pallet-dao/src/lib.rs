#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "128"]

use sp_std::prelude::*;
use frame_support::{
	dispatch::{DispatchResultWithPostInfo, Dispatchable, PostDispatchInfo},
	traits::{
		EnsureOrigin, Get,
	},
	weights::{GetDispatchInfo},
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
		type Proposal: Parameter
			+ Dispatchable<Origin = OriginFor<Self>, PostInfo = PostDispatchInfo>
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
	#[pallet::getter(fn proposal_list)]
	pub type ProposalList<T: Config> =
		StorageValue<_, Vec<T::Hash>, ValueQuery>;

	/// Actual proposal for a given hash, if it's current.
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
		SomethingStored(u32, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		StorageOverflow,
		TwinNotExists,
		NotMember,
		WrongProposalLength,
		DuplicateProposal
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Add a new proposal to either be voted on or executed directly.
		///
		/// Requires the sender to be member.
		///
		/// `threshold` determines whether `proposal` is executed directly (`threshold < 2`)
		/// or put up for voting.
		///
		/// # <weight>
		/// ## Weight
		/// - `O(B + M + P1)` or `O(B + M + P2)` where:
		///   - `B` is `proposal` size in bytes (length-fee-bounded)
		///   - `M` is members-count (code- and governance-bounded)
		///   - branching is influenced by `threshold` where:
		///     - `P1` is proposal execution complexity (`threshold < 2`)
		///     - `P2` is proposals-count (code-bounded) (`threshold >= 2`)
		/// - DB:
		///   - 1 storage read `is_member` (codec `O(M)`)
		///   - 1 storage read `ProposalOf::contains_key` (codec `O(1)`)
		///   - DB accesses influenced by `threshold`:
		///     - EITHER storage accesses done by `proposal` (`threshold < 2`)
		///     - OR proposal insertion (`threshold <= 2`)
		///       - 1 storage mutation `Proposals` (codec `O(P2)`)
		///       - 1 storage mutation `ProposalCount` (codec `O(1)`)
		///       - 1 storage write `ProposalOf` (codec `O(B)`)
		///       - 1 storage write `Voting` (codec `O(M)`)
		///   - 1 event
		/// # </weight>
		// #[pallet::weight((
		// 	if *threshold < 2 {
		// 		T::WeightInfo::propose_execute(
		// 			*length_bound, // B
		// 			T::MaxMembers::get(), // M
		// 		).saturating_add(proposal.get_dispatch_info().weight) // P1
		// 	} else {
		// 		T::WeightInfo::propose_proposed(
		// 			*length_bound, // B
		// 			T::MaxMembers::get(), // M
		// 			T::MaxProposals::get(), // P2
		// 		)
		// 	},
		// 	DispatchClass::Operational
		// ))]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn propose(
			origin: OriginFor<T>,
			action: Option<Box<<T as Config>::Proposal>>,
			description: Option<Vec<u8>>,
			link: Option<Vec<u8>>,
			#[pallet::compact] length_bound: u32,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let council_members = pallet_membership::Module::<T, pallet_membership::Instance1>::members();
			ensure!(council_members.contains(&who), Error::<T>::NotMember);

			let mut p = proposal::Proposal {
				action: None,
				description,
				link
			};

			match action {
				None => (),
				Some(a) => {
					let proposal_len = a.using_encoded(|x| x.len());
					ensure!(proposal_len <= length_bound as usize, Error::<T>::WrongProposalLength);
					let proposal_hash = T::Hashing::hash_of(&a);
					ensure!(
						!<ProposalOf<T>>::contains_key(proposal_hash),
						Error::<T>::DuplicateProposal
					);
					p.action = action;
				}
			}

			let index = Self::proposal_count();
			<ProposalCount<T>>::mutate(|i| *i += 1);
			<ProposalOf<T>>::insert(proposal_hash, *action);

			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn vote(
			origin: OriginFor<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let twin_id = pallet_tfgrid::Module::<T>::twin_ids_by_pubkey(&who);

			if twin_id == 0 {
				Err(Error::<T>::TwinNotExists)? 
			}

			Ok(().into())
		}
	}
}
