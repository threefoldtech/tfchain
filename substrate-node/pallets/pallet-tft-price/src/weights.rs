
//! Autogenerated weights for pallet_tft_price
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-05-15, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `R1-HP-ProBook-630-G8`, CPU: `11th Gen Intel(R) Core(TM) i5-1135G7 @ 2.40GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/tfchain
// benchmark
// pallet
// --chain=dev
// --pallet=pallet_tft_price
// --extrinsic=*
// --steps=50
// --repeat=20
// --execution=wasm
// --heap-pages=4096
// --output
// pallets/pallet-tft-price/src/weights.rs
// --template
// ./.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_tft_price.
pub trait WeightInfo {
	fn set_prices() -> Weight;
	fn set_min_tft_price() -> Weight;
	fn set_max_tft_price() -> Weight;
}

/// Weights for pallet_tft_price using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: Session Validators (r:1 w:0)
	// Storage: TFTPriceModule BufferRange (r:1 w:1)
	// Storage: TFTPriceModule TftPriceHistory (r:2 w:1)
	// Storage: TFTPriceModule MinTftPrice (r:1 w:0)
	// Storage: TFTPriceModule MaxTftPrice (r:1 w:0)
	// Storage: TFTPriceModule TftPrice (r:0 w:1)
	// Storage: TFTPriceModule LastBlockSet (r:0 w:1)
	// Storage: TFTPriceModule AverageTftPrice (r:0 w:1)
	fn set_prices() -> Weight {
		// Minimum execution time: 56_471 nanoseconds.
		Weight::from_ref_time(57_331_000)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	// Storage: TFTPriceModule MaxTftPrice (r:1 w:0)
	// Storage: TFTPriceModule MinTftPrice (r:0 w:1)
	fn set_min_tft_price() -> Weight {
		// Minimum execution time: 10_811 nanoseconds.
		Weight::from_ref_time(11_099_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: TFTPriceModule MinTftPrice (r:1 w:0)
	// Storage: TFTPriceModule MaxTftPrice (r:0 w:1)
	fn set_max_tft_price() -> Weight {
		// Minimum execution time: 10_624 nanoseconds.
		Weight::from_ref_time(10_932_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: Session Validators (r:1 w:0)
	// Storage: TFTPriceModule BufferRange (r:1 w:1)
	// Storage: TFTPriceModule TftPriceHistory (r:2 w:1)
	// Storage: TFTPriceModule MinTftPrice (r:1 w:0)
	// Storage: TFTPriceModule MaxTftPrice (r:1 w:0)
	// Storage: TFTPriceModule TftPrice (r:0 w:1)
	// Storage: TFTPriceModule LastBlockSet (r:0 w:1)
	// Storage: TFTPriceModule AverageTftPrice (r:0 w:1)
	fn set_prices() -> Weight {
		// Minimum execution time: 56_471 nanoseconds.
		Weight::from_ref_time(57_331_000)
			.saturating_add(RocksDbWeight::get().reads(6))
			.saturating_add(RocksDbWeight::get().writes(5))
	}
	// Storage: TFTPriceModule MaxTftPrice (r:1 w:0)
	// Storage: TFTPriceModule MinTftPrice (r:0 w:1)
	fn set_min_tft_price() -> Weight {
		// Minimum execution time: 10_811 nanoseconds.
		Weight::from_ref_time(11_099_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: TFTPriceModule MinTftPrice (r:1 w:0)
	// Storage: TFTPriceModule MaxTftPrice (r:0 w:1)
	fn set_max_tft_price() -> Weight {
		// Minimum execution time: 10_624 nanoseconds.
		Weight::from_ref_time(10_932_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
}
