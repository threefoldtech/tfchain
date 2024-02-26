
//! Autogenerated weights for pallet_dao
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2024-02-26, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `7c9e9392584e`, CPU: `AMD Ryzen 7 5800X 8-Core Processor`
//! EXECUTION: , WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/tfchain
// benchmark
// pallet
// --chain=dev
// --wasm-execution=compiled
// --pallet=pallet-dao
// --extrinsic=*
// --steps=50
// --repeat=20
// --heap-pages=409
// --output
// ./pallets/pallet-dao/src/weights.rs
// --template
// ./.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_dao.
pub trait WeightInfo {
	fn propose() -> Weight;
	fn vote() -> Weight;
	fn veto() -> Weight;
	fn close() -> Weight;
}

/// Weights for pallet_dao using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `CouncilMembership::Members` (r:1 w:0)
	/// Proof: `CouncilMembership::Members` (`max_values`: Some(1), `max_size`: Some(3202), added: 3697, mode: `MaxEncodedLen`)
	/// Storage: `Dao::ProposalOf` (r:1 w:1)
	/// Proof: `Dao::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Dao::ProposalCount` (r:1 w:1)
	/// Proof: `Dao::ProposalCount` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Dao::ProposalList` (r:1 w:1)
	/// Proof: `Dao::ProposalList` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Dao::Voting` (r:0 w:1)
	/// Proof: `Dao::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Dao::Proposals` (r:0 w:1)
	/// Proof: `Dao::Proposals` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn propose() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `208`
		//  Estimated: `4687`
		// Minimum execution time: 30_828_000 picoseconds.
		Weight::from_parts(31_480_000, 4687)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(5_u64))
	}
	/// Storage: `TfgridModule::Farms` (r:1 w:0)
	/// Proof: `TfgridModule::Farms` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `TfgridModule::Twins` (r:1 w:0)
	/// Proof: `TfgridModule::Twins` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Dao::Proposals` (r:1 w:0)
	/// Proof: `Dao::Proposals` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Dao::Voting` (r:1 w:1)
	/// Proof: `Dao::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Dao::FarmWeight` (r:1 w:0)
	/// Proof: `Dao::FarmWeight` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn vote() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `979`
		//  Estimated: `4444`
		// Minimum execution time: 42_109_000 picoseconds.
		Weight::from_parts(43_242_000, 4444)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `CouncilMembership::Members` (r:1 w:0)
	/// Proof: `CouncilMembership::Members` (`max_values`: Some(1), `max_size`: Some(3202), added: 3697, mode: `MaxEncodedLen`)
	/// Storage: `Dao::Proposals` (r:1 w:0)
	/// Proof: `Dao::Proposals` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Dao::Voting` (r:1 w:1)
	/// Proof: `Dao::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn veto() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `487`
		//  Estimated: `4687`
		// Minimum execution time: 17_753_000 picoseconds.
		Weight::from_parts(18_295_000, 4687)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `CouncilMembership::Members` (r:1 w:0)
	/// Proof: `CouncilMembership::Members` (`max_values`: Some(1), `max_size`: Some(3202), added: 3697, mode: `MaxEncodedLen`)
	/// Storage: `Dao::Voting` (r:1 w:1)
	/// Proof: `Dao::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Dao::ProposalList` (r:1 w:1)
	/// Proof: `Dao::ProposalList` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Dao::Proposals` (r:0 w:1)
	/// Proof: `Dao::Proposals` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Dao::ProposalOf` (r:0 w:1)
	/// Proof: `Dao::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn close() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `469`
		//  Estimated: `4687`
		// Minimum execution time: 23_665_000 picoseconds.
		Weight::from_parts(24_106_000, 4687)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: `CouncilMembership::Members` (r:1 w:0)
	/// Proof: `CouncilMembership::Members` (`max_values`: Some(1), `max_size`: Some(3202), added: 3697, mode: `MaxEncodedLen`)
	/// Storage: `Dao::ProposalOf` (r:1 w:1)
	/// Proof: `Dao::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Dao::ProposalCount` (r:1 w:1)
	/// Proof: `Dao::ProposalCount` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Dao::ProposalList` (r:1 w:1)
	/// Proof: `Dao::ProposalList` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Dao::Voting` (r:0 w:1)
	/// Proof: `Dao::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Dao::Proposals` (r:0 w:1)
	/// Proof: `Dao::Proposals` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn propose() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `208`
		//  Estimated: `4687`
		// Minimum execution time: 30_828_000 picoseconds.
		Weight::from_parts(31_480_000, 4687)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(5_u64))
	}
	/// Storage: `TfgridModule::Farms` (r:1 w:0)
	/// Proof: `TfgridModule::Farms` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `TfgridModule::Twins` (r:1 w:0)
	/// Proof: `TfgridModule::Twins` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Dao::Proposals` (r:1 w:0)
	/// Proof: `Dao::Proposals` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Dao::Voting` (r:1 w:1)
	/// Proof: `Dao::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Dao::FarmWeight` (r:1 w:0)
	/// Proof: `Dao::FarmWeight` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn vote() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `979`
		//  Estimated: `4444`
		// Minimum execution time: 42_109_000 picoseconds.
		Weight::from_parts(43_242_000, 4444)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `CouncilMembership::Members` (r:1 w:0)
	/// Proof: `CouncilMembership::Members` (`max_values`: Some(1), `max_size`: Some(3202), added: 3697, mode: `MaxEncodedLen`)
	/// Storage: `Dao::Proposals` (r:1 w:0)
	/// Proof: `Dao::Proposals` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Dao::Voting` (r:1 w:1)
	/// Proof: `Dao::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn veto() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `487`
		//  Estimated: `4687`
		// Minimum execution time: 17_753_000 picoseconds.
		Weight::from_parts(18_295_000, 4687)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `CouncilMembership::Members` (r:1 w:0)
	/// Proof: `CouncilMembership::Members` (`max_values`: Some(1), `max_size`: Some(3202), added: 3697, mode: `MaxEncodedLen`)
	/// Storage: `Dao::Voting` (r:1 w:1)
	/// Proof: `Dao::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Dao::ProposalList` (r:1 w:1)
	/// Proof: `Dao::ProposalList` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Dao::Proposals` (r:0 w:1)
	/// Proof: `Dao::Proposals` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Dao::ProposalOf` (r:0 w:1)
	/// Proof: `Dao::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn close() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `469`
		//  Estimated: `4687`
		// Minimum execution time: 23_665_000 picoseconds.
		Weight::from_parts(24_106_000, 4687)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
}
