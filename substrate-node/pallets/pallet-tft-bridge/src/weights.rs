
//! Autogenerated weights for pallet_tft_bridge
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-06-15, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `R1-HP-ProBook-630-G8`, CPU: `11th Gen Intel(R) Core(TM) i5-1135G7 @ 2.40GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/tfchain
// benchmark
// pallet
// --chain=dev
// --pallet=pallet_tft_bridge
// --extrinsic=*
// --steps=50
// --repeat=20
// --execution=wasm
// --heap-pages=409
// --output
// ./pallets/pallet-tft-bridge/src/weights.rs
// --template
// ./.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_tft_bridge.
pub trait WeightInfo {
	fn add_bridge_validator() -> Weight;
	fn remove_bridge_validator() -> Weight;
	fn set_fee_account() -> Weight;
	fn set_withdraw_fee() -> Weight;
	fn set_deposit_fee() -> Weight;
	fn swap_to_stellar() -> Weight;
	fn propose_or_vote_mint_transaction() -> Weight;
	fn propose_burn_transaction_or_add_sig() -> Weight;
	fn set_burn_transaction_executed() -> Weight;
	fn create_refund_transaction_or_add_sig() -> Weight;
	fn set_refund_transaction_executed() -> Weight;
}

/// Weights for pallet_tft_bridge using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: TFTBridgeModule Validators (r:1 w:1)
	/// Proof Skipped: TFTBridgeModule Validators (max_values: Some(1), max_size: None, mode: Measured)
	fn add_bridge_validator() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `289`
		//  Estimated: `1774`
		// Minimum execution time: 18_778_000 picoseconds.
		Weight::from_parts(20_233_000, 1774)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: TFTBridgeModule Validators (r:1 w:1)
	/// Proof Skipped: TFTBridgeModule Validators (max_values: Some(1), max_size: None, mode: Measured)
	fn remove_bridge_validator() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `322`
		//  Estimated: `1807`
		// Minimum execution time: 12_346_000 picoseconds.
		Weight::from_parts(17_998_000, 1807)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: TFTBridgeModule FeeAccount (r:0 w:1)
	/// Proof Skipped: TFTBridgeModule FeeAccount (max_values: Some(1), max_size: None, mode: Measured)
	fn set_fee_account() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 4_548_000 picoseconds.
		Weight::from_parts(5_051_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: TFTBridgeModule WithdrawFee (r:0 w:1)
	/// Proof Skipped: TFTBridgeModule WithdrawFee (max_values: Some(1), max_size: None, mode: Measured)
	fn set_withdraw_fee() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 4_279_000 picoseconds.
		Weight::from_parts(4_493_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: TFTBridgeModule DepositFee (r:0 w:1)
	/// Proof Skipped: TFTBridgeModule DepositFee (max_values: Some(1), max_size: None, mode: Measured)
	fn set_deposit_fee() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 4_584_000 picoseconds.
		Weight::from_parts(4_876_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: TFTBridgeModule WithdrawFee (r:1 w:0)
	/// Proof Skipped: TFTBridgeModule WithdrawFee (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule FeeAccount (r:1 w:0)
	/// Proof Skipped: TFTBridgeModule FeeAccount (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: TFTBridgeModule BurnTransactionID (r:1 w:1)
	/// Proof Skipped: TFTBridgeModule BurnTransactionID (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule BurnTransactions (r:0 w:1)
	/// Proof Skipped: TFTBridgeModule BurnTransactions (max_values: None, max_size: None, mode: Measured)
	fn swap_to_stellar() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `286`
		//  Estimated: `3593`
		// Minimum execution time: 74_554_000 picoseconds.
		Weight::from_parts(76_009_000, 3593)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: TFTBridgeModule Validators (r:1 w:0)
	/// Proof Skipped: TFTBridgeModule Validators (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule ExecutedMintTransactions (r:1 w:1)
	/// Proof Skipped: TFTBridgeModule ExecutedMintTransactions (max_values: None, max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule MintTransactions (r:1 w:1)
	/// Proof Skipped: TFTBridgeModule MintTransactions (max_values: None, max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule DepositFee (r:1 w:0)
	/// Proof Skipped: TFTBridgeModule DepositFee (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule FeeAccount (r:1 w:0)
	/// Proof Skipped: TFTBridgeModule FeeAccount (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn propose_or_vote_mint_transaction() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `532`
		//  Estimated: `3997`
		// Minimum execution time: 102_293_000 picoseconds.
		Weight::from_parts(113_671_000, 3997)
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: TFTBridgeModule Validators (r:1 w:0)
	/// Proof Skipped: TFTBridgeModule Validators (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule ExecutedBurnTransactions (r:1 w:0)
	/// Proof Skipped: TFTBridgeModule ExecutedBurnTransactions (max_values: None, max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule BurnTransactions (r:1 w:1)
	/// Proof Skipped: TFTBridgeModule BurnTransactions (max_values: None, max_size: None, mode: Measured)
	fn propose_burn_transaction_or_add_sig() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `631`
		//  Estimated: `4096`
		// Minimum execution time: 41_613_000 picoseconds.
		Weight::from_parts(42_948_000, 4096)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: TFTBridgeModule Validators (r:1 w:0)
	/// Proof Skipped: TFTBridgeModule Validators (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule ExecutedBurnTransactions (r:1 w:1)
	/// Proof Skipped: TFTBridgeModule ExecutedBurnTransactions (max_values: None, max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule BurnTransactions (r:1 w:1)
	/// Proof Skipped: TFTBridgeModule BurnTransactions (max_values: None, max_size: None, mode: Measured)
	fn set_burn_transaction_executed() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `571`
		//  Estimated: `4036`
		// Minimum execution time: 28_712_000 picoseconds.
		Weight::from_parts(29_777_000, 4036)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: TFTBridgeModule Validators (r:1 w:0)
	/// Proof Skipped: TFTBridgeModule Validators (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule RefundTransactions (r:1 w:1)
	/// Proof Skipped: TFTBridgeModule RefundTransactions (max_values: None, max_size: None, mode: Measured)
	fn create_refund_transaction_or_add_sig() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `418`
		//  Estimated: `3883`
		// Minimum execution time: 36_551_000 picoseconds.
		Weight::from_parts(37_525_000, 3883)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: TFTBridgeModule Validators (r:1 w:0)
	/// Proof Skipped: TFTBridgeModule Validators (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule ExecutedRefundTransactions (r:1 w:1)
	/// Proof Skipped: TFTBridgeModule ExecutedRefundTransactions (max_values: None, max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule RefundTransactions (r:1 w:1)
	/// Proof Skipped: TFTBridgeModule RefundTransactions (max_values: None, max_size: None, mode: Measured)
	fn set_refund_transaction_executed() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `593`
		//  Estimated: `4058`
		// Minimum execution time: 34_047_000 picoseconds.
		Weight::from_parts(35_316_000, 4058)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: TFTBridgeModule Validators (r:1 w:1)
	/// Proof Skipped: TFTBridgeModule Validators (max_values: Some(1), max_size: None, mode: Measured)
	fn add_bridge_validator() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `289`
		//  Estimated: `1774`
		// Minimum execution time: 18_778_000 picoseconds.
		Weight::from_parts(20_233_000, 1774)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: TFTBridgeModule Validators (r:1 w:1)
	/// Proof Skipped: TFTBridgeModule Validators (max_values: Some(1), max_size: None, mode: Measured)
	fn remove_bridge_validator() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `322`
		//  Estimated: `1807`
		// Minimum execution time: 12_346_000 picoseconds.
		Weight::from_parts(17_998_000, 1807)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: TFTBridgeModule FeeAccount (r:0 w:1)
	/// Proof Skipped: TFTBridgeModule FeeAccount (max_values: Some(1), max_size: None, mode: Measured)
	fn set_fee_account() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 4_548_000 picoseconds.
		Weight::from_parts(5_051_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: TFTBridgeModule WithdrawFee (r:0 w:1)
	/// Proof Skipped: TFTBridgeModule WithdrawFee (max_values: Some(1), max_size: None, mode: Measured)
	fn set_withdraw_fee() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 4_279_000 picoseconds.
		Weight::from_parts(4_493_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: TFTBridgeModule DepositFee (r:0 w:1)
	/// Proof Skipped: TFTBridgeModule DepositFee (max_values: Some(1), max_size: None, mode: Measured)
	fn set_deposit_fee() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 4_584_000 picoseconds.
		Weight::from_parts(4_876_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: TFTBridgeModule WithdrawFee (r:1 w:0)
	/// Proof Skipped: TFTBridgeModule WithdrawFee (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule FeeAccount (r:1 w:0)
	/// Proof Skipped: TFTBridgeModule FeeAccount (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: TFTBridgeModule BurnTransactionID (r:1 w:1)
	/// Proof Skipped: TFTBridgeModule BurnTransactionID (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule BurnTransactions (r:0 w:1)
	/// Proof Skipped: TFTBridgeModule BurnTransactions (max_values: None, max_size: None, mode: Measured)
	fn swap_to_stellar() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `286`
		//  Estimated: `3593`
		// Minimum execution time: 74_554_000 picoseconds.
		Weight::from_parts(76_009_000, 3593)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: TFTBridgeModule Validators (r:1 w:0)
	/// Proof Skipped: TFTBridgeModule Validators (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule ExecutedMintTransactions (r:1 w:1)
	/// Proof Skipped: TFTBridgeModule ExecutedMintTransactions (max_values: None, max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule MintTransactions (r:1 w:1)
	/// Proof Skipped: TFTBridgeModule MintTransactions (max_values: None, max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule DepositFee (r:1 w:0)
	/// Proof Skipped: TFTBridgeModule DepositFee (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule FeeAccount (r:1 w:0)
	/// Proof Skipped: TFTBridgeModule FeeAccount (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn propose_or_vote_mint_transaction() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `532`
		//  Estimated: `3997`
		// Minimum execution time: 102_293_000 picoseconds.
		Weight::from_parts(113_671_000, 3997)
			.saturating_add(RocksDbWeight::get().reads(6_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: TFTBridgeModule Validators (r:1 w:0)
	/// Proof Skipped: TFTBridgeModule Validators (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule ExecutedBurnTransactions (r:1 w:0)
	/// Proof Skipped: TFTBridgeModule ExecutedBurnTransactions (max_values: None, max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule BurnTransactions (r:1 w:1)
	/// Proof Skipped: TFTBridgeModule BurnTransactions (max_values: None, max_size: None, mode: Measured)
	fn propose_burn_transaction_or_add_sig() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `631`
		//  Estimated: `4096`
		// Minimum execution time: 41_613_000 picoseconds.
		Weight::from_parts(42_948_000, 4096)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: TFTBridgeModule Validators (r:1 w:0)
	/// Proof Skipped: TFTBridgeModule Validators (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule ExecutedBurnTransactions (r:1 w:1)
	/// Proof Skipped: TFTBridgeModule ExecutedBurnTransactions (max_values: None, max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule BurnTransactions (r:1 w:1)
	/// Proof Skipped: TFTBridgeModule BurnTransactions (max_values: None, max_size: None, mode: Measured)
	fn set_burn_transaction_executed() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `571`
		//  Estimated: `4036`
		// Minimum execution time: 28_712_000 picoseconds.
		Weight::from_parts(29_777_000, 4036)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: TFTBridgeModule Validators (r:1 w:0)
	/// Proof Skipped: TFTBridgeModule Validators (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule RefundTransactions (r:1 w:1)
	/// Proof Skipped: TFTBridgeModule RefundTransactions (max_values: None, max_size: None, mode: Measured)
	fn create_refund_transaction_or_add_sig() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `418`
		//  Estimated: `3883`
		// Minimum execution time: 36_551_000 picoseconds.
		Weight::from_parts(37_525_000, 3883)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: TFTBridgeModule Validators (r:1 w:0)
	/// Proof Skipped: TFTBridgeModule Validators (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule ExecutedRefundTransactions (r:1 w:1)
	/// Proof Skipped: TFTBridgeModule ExecutedRefundTransactions (max_values: None, max_size: None, mode: Measured)
	/// Storage: TFTBridgeModule RefundTransactions (r:1 w:1)
	/// Proof Skipped: TFTBridgeModule RefundTransactions (max_values: None, max_size: None, mode: Measured)
	fn set_refund_transaction_executed() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `593`
		//  Estimated: `4058`
		// Minimum execution time: 34_047_000 picoseconds.
		Weight::from_parts(35_316_000, 4058)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
}
