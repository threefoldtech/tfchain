
//! Autogenerated weights for pallet_validator
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-05-17, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `R1-HP-ProBook-630-G8`, CPU: `11th Gen Intel(R) Core(TM) i5-1135G7 @ 2.40GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/tfchain
// benchmark
// pallet
// --chain=dev
// --pallet=pallet_validator
// --extrinsic=*
// --steps=50
// --repeat=20
// --execution=wasm
// --heap-pages=4096
// --output
// pallets/pallet-validator/src/weights.rs
// --template
// ./.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_validator.
pub trait WeightInfo {
	fn create_validator_request() -> Weight;
	fn activate_validator_node() -> Weight;
	fn change_validator_node_account() -> Weight;
	fn bond() -> Weight;
	fn approve_validator() -> Weight;
	fn remove_validator() -> Weight;
}

/// Weights for pallet_validator using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: Validator Validator (r:1 w:1)
	fn create_validator_request() -> Weight {
		// Minimum execution time: 23_331 nanoseconds.
		Weight::from_ref_time(25_562_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: Validator Validator (r:1 w:1)
	// Storage: ValidatorSet Validators (r:1 w:1)
	// Storage: ValidatorSet ApprovedValidators (r:1 w:1)
	fn activate_validator_node() -> Weight {
		// Minimum execution time: 42_958 nanoseconds.
		Weight::from_ref_time(44_151_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Validator Validator (r:1 w:1)
	// Storage: ValidatorSet Validators (r:1 w:1)
	// Storage: ValidatorSet ApprovedValidators (r:1 w:1)
	fn change_validator_node_account() -> Weight {
		// Minimum execution time: 56_451 nanoseconds.
		Weight::from_ref_time(63_248_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	// Storage: Validator Bonded (r:1 w:1)
	fn bond() -> Weight {
		// Minimum execution time: 20_742 nanoseconds.
		Weight::from_ref_time(23_165_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: CouncilMembership Members (r:1 w:1)
	// Storage: Validator Validator (r:1 w:1)
	// Storage: Council Proposals (r:1 w:0)
	// Storage: Council Members (r:0 w:1)
	// Storage: Council Prime (r:0 w:1)
	fn approve_validator() -> Weight {
		// Minimum execution time: 48_121 nanoseconds.
		Weight::from_ref_time(50_967_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: CouncilMembership Members (r:1 w:1)
	// Storage: Validator Validator (r:1 w:1)
	// Storage: Council Proposals (r:1 w:0)
	// Storage: CouncilMembership Prime (r:1 w:0)
	// Storage: Council Members (r:0 w:1)
	// Storage: Council Prime (r:0 w:1)
	fn remove_validator() -> Weight {
		// Minimum execution time: 39_847 nanoseconds.
		Weight::from_ref_time(41_288_000)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(4))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Validator Validator (r:1 w:1)
	fn create_validator_request() -> Weight {
		// Minimum execution time: 23_331 nanoseconds.
		Weight::from_ref_time(25_562_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: Validator Validator (r:1 w:1)
	// Storage: ValidatorSet Validators (r:1 w:1)
	// Storage: ValidatorSet ApprovedValidators (r:1 w:1)
	fn activate_validator_node() -> Weight {
		// Minimum execution time: 42_958 nanoseconds.
		Weight::from_ref_time(44_151_000)
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
	// Storage: Validator Validator (r:1 w:1)
	// Storage: ValidatorSet Validators (r:1 w:1)
	// Storage: ValidatorSet ApprovedValidators (r:1 w:1)
	fn change_validator_node_account() -> Weight {
		// Minimum execution time: 56_451 nanoseconds.
		Weight::from_ref_time(63_248_000)
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(3))
	}
	// Storage: Validator Bonded (r:1 w:1)
	fn bond() -> Weight {
		// Minimum execution time: 20_742 nanoseconds.
		Weight::from_ref_time(23_165_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: CouncilMembership Members (r:1 w:1)
	// Storage: Validator Validator (r:1 w:1)
	// Storage: Council Proposals (r:1 w:0)
	// Storage: Council Members (r:0 w:1)
	// Storage: Council Prime (r:0 w:1)
	fn approve_validator() -> Weight {
		// Minimum execution time: 48_121 nanoseconds.
		Weight::from_ref_time(50_967_000)
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
	// Storage: CouncilMembership Members (r:1 w:1)
	// Storage: Validator Validator (r:1 w:1)
	// Storage: Council Proposals (r:1 w:0)
	// Storage: CouncilMembership Prime (r:1 w:0)
	// Storage: Council Members (r:0 w:1)
	// Storage: Council Prime (r:0 w:1)
	fn remove_validator() -> Weight {
		// Minimum execution time: 39_847 nanoseconds.
		Weight::from_ref_time(41_288_000)
			.saturating_add(RocksDbWeight::get().reads(4))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
}
