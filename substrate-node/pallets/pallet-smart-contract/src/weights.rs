
//! Autogenerated weights for pallet_smart_contract
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-03-01, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `R1-HP-ProBook-630-G8`, CPU: `11th Gen Intel(R) Core(TM) i5-1135G7 @ 2.40GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/tfchain
// benchmark
// pallet
// --chain=dev
// --pallet=pallet_smart_contract
// --extrinsic=*
// --steps=50
// --repeat=20
// --execution=wasm
// --heap-pages=4096
// --output
// pallets/pallet-smart-contract/src/weights.rs
// --template
// ./.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_smart_contract.
pub trait WeightInfo {
	fn create_node_contract() -> Weight;
	fn update_node_contract() -> Weight;
	fn cancel_contract() -> Weight;
	fn create_name_contract() -> Weight;
	fn cancel_name_contract() -> Weight;
	fn add_nru_reports() -> Weight;
	fn report_contract_resources() -> Weight;
	fn create_rent_contract() -> Weight;
	fn cancel_rent_contract() -> Weight;
	fn create_solution_provider() -> Weight;
	fn approve_solution_provider() -> Weight;
	fn bill_contract_for_block() -> Weight;
	fn service_contract_create() -> Weight;
	fn service_contract_set_metadata() -> Weight;
	fn service_contract_set_fees() -> Weight;
	fn service_contract_approve() -> Weight;
	fn service_contract_reject() -> Weight;
	fn service_contract_cancel() -> Weight;
	fn service_contract_bill() -> Weight;
	fn change_billing_frequency() -> Weight;
}

/// Weights for pallet_smart_contract using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: TfgridModule TwinIdByAccountID (r:1 w:0)
	// Storage: TfgridModule Nodes (r:1 w:0)
	// Storage: TfgridModule NodePower (r:1 w:0)
	// Storage: TfgridModule Farms (r:1 w:1)
	// Storage: SmartContractModule ActiveRentContractForNode (r:1 w:0)
	// Storage: SmartContractModule ContractIDByNodeIDAndHash (r:1 w:1)
	// Storage: SmartContractModule ContractID (r:1 w:1)
	// Storage: SmartContractModule BillingFrequency (r:1 w:0)
	// Storage: SmartContractModule ContractsToBillAt (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: SmartContractModule ActiveNodeContracts (r:1 w:1)
	// Storage: SmartContractModule Contracts (r:0 w:1)
	// Storage: SmartContractModule ContractBillingInformationByID (r:0 w:1)
	// Storage: SmartContractModule ContractLock (r:0 w:1)
	fn create_node_contract() -> Weight {
		// Minimum execution time: 79_356 nanoseconds.
		Weight::from_ref_time(80_937_000)
			.saturating_add(T::DbWeight::get().reads(11))
			.saturating_add(T::DbWeight::get().writes(8))
	}
	// Storage: SmartContractModule Contracts (r:1 w:1)
	// Storage: TfgridModule Twins (r:1 w:0)
	// Storage: SmartContractModule ActiveNodeContracts (r:1 w:1)
	// Storage: SmartContractModule ContractIDByNodeIDAndHash (r:0 w:2)
	fn update_node_contract() -> Weight {
		// Minimum execution time: 49_586 nanoseconds.
		Weight::from_ref_time(50_752_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: SmartContractModule Contracts (r:1 w:1)
	// Storage: TfgridModule Twins (r:1 w:0)
	// Storage: SmartContractModule ActiveNodeContracts (r:1 w:1)
	// Storage: SmartContractModule BillingFrequency (r:1 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: SmartContractModule ContractLock (r:1 w:1)
	// Storage: TfgridModule PricingPolicies (r:1 w:0)
	// Storage: SmartContractModule ContractBillingInformationByID (r:1 w:1)
	// Storage: TfgridModule Nodes (r:1 w:0)
	// Storage: SmartContractModule NodeContractResources (r:1 w:1)
	// Storage: SmartContractModule ActiveRentContractForNode (r:1 w:0)
	// Storage: SmartContractModule ContractIDByNodeIDAndHash (r:0 w:1)
	fn cancel_contract() -> Weight {
		// Minimum execution time: 109_934 nanoseconds.
		Weight::from_ref_time(123_338_000)
			.saturating_add(T::DbWeight::get().reads(11))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	// Storage: TfgridModule TwinIdByAccountID (r:1 w:0)
	// Storage: SmartContractModule ContractIDByNameRegistration (r:1 w:1)
	// Storage: SmartContractModule ContractID (r:1 w:1)
	// Storage: SmartContractModule BillingFrequency (r:1 w:0)
	// Storage: SmartContractModule ContractsToBillAt (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: SmartContractModule Contracts (r:0 w:1)
	// Storage: SmartContractModule ContractLock (r:0 w:1)
	fn create_name_contract() -> Weight {
		// Minimum execution time: 48_722 nanoseconds.
		Weight::from_ref_time(49_772_000)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	// Storage: SmartContractModule Contracts (r:1 w:1)
	// Storage: TfgridModule Twins (r:1 w:0)
	// Storage: SmartContractModule BillingFrequency (r:1 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: SmartContractModule ContractLock (r:1 w:1)
	// Storage: TfgridModule PricingPolicies (r:1 w:0)
	// Storage: TFTPriceModule AverageTftPrice (r:1 w:0)
	// Storage: TFTPriceModule MinTftPrice (r:1 w:0)
	// Storage: TFTPriceModule MaxTftPrice (r:1 w:0)
	// Storage: SmartContractModule ContractBillingInformationByID (r:1 w:1)
	// Storage: SmartContractModule ContractIDByNameRegistration (r:0 w:1)
	fn cancel_name_contract() -> Weight {
		// Minimum execution time: 98_590 nanoseconds.
		Weight::from_ref_time(127_355_000)
			.saturating_add(T::DbWeight::get().reads(10))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: TfgridModule TwinIdByAccountID (r:1 w:0)
	// Storage: TfgridModule NodeIdByTwinID (r:1 w:0)
	// Storage: TfgridModule Nodes (r:1 w:0)
	// Storage: TfgridModule Farms (r:1 w:0)
	// Storage: TfgridModule PricingPolicies (r:1 w:0)
	// Storage: SmartContractModule Contracts (r:1 w:0)
	// Storage: SmartContractModule ContractBillingInformationByID (r:1 w:1)
	fn add_nru_reports() -> Weight {
		// Minimum execution time: 65_944 nanoseconds.
		Weight::from_ref_time(67_476_000)
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: TfgridModule TwinIdByAccountID (r:1 w:0)
	// Storage: TfgridModule NodeIdByTwinID (r:1 w:0)
	// Storage: SmartContractModule Contracts (r:1 w:0)
	// Storage: SmartContractModule NodeContractResources (r:0 w:1)
	fn report_contract_resources() -> Weight {
		// Minimum execution time: 42_133 nanoseconds.
		Weight::from_ref_time(43_238_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: SmartContractModule ActiveRentContractForNode (r:1 w:1)
	// Storage: TfgridModule Nodes (r:1 w:0)
	// Storage: TfgridModule Farms (r:1 w:0)
	// Storage: TfgridModule NodePower (r:1 w:0)
	// Storage: SmartContractModule ActiveNodeContracts (r:1 w:0)
	// Storage: TfgridModule TwinIdByAccountID (r:1 w:0)
	// Storage: SmartContractModule ContractID (r:1 w:1)
	// Storage: SmartContractModule BillingFrequency (r:1 w:0)
	// Storage: SmartContractModule ContractsToBillAt (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: SmartContractModule Contracts (r:0 w:1)
	// Storage: SmartContractModule ContractLock (r:0 w:1)
	fn create_rent_contract() -> Weight {
		// Minimum execution time: 63_783 nanoseconds.
		Weight::from_ref_time(72_172_000)
			.saturating_add(T::DbWeight::get().reads(10))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	// Storage: SmartContractModule Contracts (r:1 w:1)
	// Storage: TfgridModule Twins (r:1 w:0)
	// Storage: SmartContractModule ActiveNodeContracts (r:1 w:0)
	// Storage: SmartContractModule BillingFrequency (r:1 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: SmartContractModule ContractLock (r:1 w:1)
	// Storage: TfgridModule PricingPolicies (r:1 w:0)
	// Storage: TfgridModule Nodes (r:1 w:0)
	// Storage: TFTPriceModule AverageTftPrice (r:1 w:0)
	// Storage: TFTPriceModule MinTftPrice (r:1 w:0)
	// Storage: TFTPriceModule MaxTftPrice (r:1 w:0)
	// Storage: SmartContractModule ContractBillingInformationByID (r:1 w:1)
	// Storage: SmartContractModule ActiveRentContractForNode (r:0 w:1)
	fn cancel_rent_contract() -> Weight {
		// Minimum execution time: 103_720 nanoseconds.
		Weight::from_ref_time(106_347_000)
			.saturating_add(T::DbWeight::get().reads(12))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: SmartContractModule SolutionProviderID (r:1 w:1)
	// Storage: SmartContractModule SolutionProviders (r:0 w:1)
	fn create_solution_provider() -> Weight {
		// Minimum execution time: 23_905 nanoseconds.
		Weight::from_ref_time(26_479_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: SmartContractModule SolutionProviders (r:1 w:1)
	fn approve_solution_provider() -> Weight {
		// Minimum execution time: 27_670 nanoseconds.
		Weight::from_ref_time(30_376_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: SmartContractModule Contracts (r:1 w:0)
	// Storage: TfgridModule Twins (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: SmartContractModule BillingFrequency (r:1 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: SmartContractModule ContractLock (r:1 w:1)
	// Storage: TfgridModule PricingPolicies (r:1 w:0)
	// Storage: SmartContractModule ContractBillingInformationByID (r:1 w:1)
	// Storage: TfgridModule Nodes (r:1 w:0)
	// Storage: SmartContractModule NodeContractResources (r:1 w:0)
	// Storage: SmartContractModule ActiveRentContractForNode (r:1 w:0)
	// Storage: TFTPriceModule AverageTftPrice (r:1 w:0)
	// Storage: TFTPriceModule MinTftPrice (r:1 w:0)
	// Storage: TFTPriceModule MaxTftPrice (r:1 w:0)
	// Storage: Balances Locks (r:1 w:1)
	fn bill_contract_for_block() -> Weight {
		// Minimum execution time: 110_792 nanoseconds.
		Weight::from_ref_time(114_740_000)
			.saturating_add(T::DbWeight::get().reads(15))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	// Storage: TfgridModule TwinIdByAccountID (r:2 w:0)
	// Storage: SmartContractModule ServiceContractID (r:1 w:1)
	// Storage: SmartContractModule ServiceContracts (r:0 w:1)
	fn service_contract_create() -> Weight {
		// Minimum execution time: 34_632 nanoseconds.
		Weight::from_ref_time(35_528_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: TfgridModule TwinIdByAccountID (r:1 w:0)
	// Storage: SmartContractModule ServiceContracts (r:1 w:1)
	fn service_contract_set_metadata() -> Weight {
		// Minimum execution time: 31_482 nanoseconds.
		Weight::from_ref_time(35_295_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: TfgridModule TwinIdByAccountID (r:1 w:0)
	// Storage: SmartContractModule ServiceContracts (r:1 w:1)
	fn service_contract_set_fees() -> Weight {
		// Minimum execution time: 31_024 nanoseconds.
		Weight::from_ref_time(35_066_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: TfgridModule TwinIdByAccountID (r:1 w:0)
	// Storage: SmartContractModule ServiceContracts (r:1 w:1)
	fn service_contract_approve() -> Weight {
		// Minimum execution time: 31_558 nanoseconds.
		Weight::from_ref_time(33_168_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: TfgridModule TwinIdByAccountID (r:1 w:0)
	// Storage: SmartContractModule ServiceContracts (r:1 w:1)
	fn service_contract_reject() -> Weight {
		// Minimum execution time: 35_446 nanoseconds.
		Weight::from_ref_time(36_650_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: TfgridModule TwinIdByAccountID (r:1 w:0)
	// Storage: SmartContractModule ServiceContracts (r:1 w:1)
	fn service_contract_cancel() -> Weight {
		// Minimum execution time: 34_093 nanoseconds.
		Weight::from_ref_time(35_933_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: TfgridModule TwinIdByAccountID (r:1 w:0)
	// Storage: SmartContractModule ServiceContracts (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: TfgridModule Twins (r:2 w:0)
	// Storage: System Account (r:1 w:0)
	fn service_contract_bill() -> Weight {
		// Minimum execution time: 53_157 nanoseconds.
		Weight::from_ref_time(61_498_000)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: SmartContractModule BillingFrequency (r:1 w:1)
	fn change_billing_frequency() -> Weight {
		// Minimum execution time: 33_324 nanoseconds.
		Weight::from_ref_time(33_940_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: TfgridModule TwinIdByAccountID (r:1 w:0)
	// Storage: TfgridModule Nodes (r:1 w:0)
	// Storage: TfgridModule NodePower (r:1 w:0)
	// Storage: TfgridModule Farms (r:1 w:1)
	// Storage: SmartContractModule ActiveRentContractForNode (r:1 w:0)
	// Storage: SmartContractModule ContractIDByNodeIDAndHash (r:1 w:1)
	// Storage: SmartContractModule ContractID (r:1 w:1)
	// Storage: SmartContractModule BillingFrequency (r:1 w:0)
	// Storage: SmartContractModule ContractsToBillAt (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: SmartContractModule ActiveNodeContracts (r:1 w:1)
	// Storage: SmartContractModule Contracts (r:0 w:1)
	// Storage: SmartContractModule ContractBillingInformationByID (r:0 w:1)
	// Storage: SmartContractModule ContractLock (r:0 w:1)
	fn create_node_contract() -> Weight {
		// Minimum execution time: 79_356 nanoseconds.
		Weight::from_ref_time(80_937_000)
			.saturating_add(RocksDbWeight::get().reads(11))
			.saturating_add(RocksDbWeight::get().writes(8))
	}
	// Storage: SmartContractModule Contracts (r:1 w:1)
	// Storage: TfgridModule Twins (r:1 w:0)
	// Storage: SmartContractModule ActiveNodeContracts (r:1 w:1)
	// Storage: SmartContractModule ContractIDByNodeIDAndHash (r:0 w:2)
	fn update_node_contract() -> Weight {
		// Minimum execution time: 49_586 nanoseconds.
		Weight::from_ref_time(50_752_000)
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
	// Storage: SmartContractModule Contracts (r:1 w:1)
	// Storage: TfgridModule Twins (r:1 w:0)
	// Storage: SmartContractModule ActiveNodeContracts (r:1 w:1)
	// Storage: SmartContractModule BillingFrequency (r:1 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: SmartContractModule ContractLock (r:1 w:1)
	// Storage: TfgridModule PricingPolicies (r:1 w:0)
	// Storage: SmartContractModule ContractBillingInformationByID (r:1 w:1)
	// Storage: TfgridModule Nodes (r:1 w:0)
	// Storage: SmartContractModule NodeContractResources (r:1 w:1)
	// Storage: SmartContractModule ActiveRentContractForNode (r:1 w:0)
	// Storage: SmartContractModule ContractIDByNodeIDAndHash (r:0 w:1)
	fn cancel_contract() -> Weight {
		// Minimum execution time: 109_934 nanoseconds.
		Weight::from_ref_time(123_338_000)
			.saturating_add(RocksDbWeight::get().reads(11))
			.saturating_add(RocksDbWeight::get().writes(6))
	}
	// Storage: TfgridModule TwinIdByAccountID (r:1 w:0)
	// Storage: SmartContractModule ContractIDByNameRegistration (r:1 w:1)
	// Storage: SmartContractModule ContractID (r:1 w:1)
	// Storage: SmartContractModule BillingFrequency (r:1 w:0)
	// Storage: SmartContractModule ContractsToBillAt (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: SmartContractModule Contracts (r:0 w:1)
	// Storage: SmartContractModule ContractLock (r:0 w:1)
	fn create_name_contract() -> Weight {
		// Minimum execution time: 48_722 nanoseconds.
		Weight::from_ref_time(49_772_000)
			.saturating_add(RocksDbWeight::get().reads(6))
			.saturating_add(RocksDbWeight::get().writes(5))
	}
	// Storage: SmartContractModule Contracts (r:1 w:1)
	// Storage: TfgridModule Twins (r:1 w:0)
	// Storage: SmartContractModule BillingFrequency (r:1 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: SmartContractModule ContractLock (r:1 w:1)
	// Storage: TfgridModule PricingPolicies (r:1 w:0)
	// Storage: TFTPriceModule AverageTftPrice (r:1 w:0)
	// Storage: TFTPriceModule MinTftPrice (r:1 w:0)
	// Storage: TFTPriceModule MaxTftPrice (r:1 w:0)
	// Storage: SmartContractModule ContractBillingInformationByID (r:1 w:1)
	// Storage: SmartContractModule ContractIDByNameRegistration (r:0 w:1)
	fn cancel_name_contract() -> Weight {
		// Minimum execution time: 98_590 nanoseconds.
		Weight::from_ref_time(127_355_000)
			.saturating_add(RocksDbWeight::get().reads(10))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
	// Storage: TfgridModule TwinIdByAccountID (r:1 w:0)
	// Storage: TfgridModule NodeIdByTwinID (r:1 w:0)
	// Storage: TfgridModule Nodes (r:1 w:0)
	// Storage: TfgridModule Farms (r:1 w:0)
	// Storage: TfgridModule PricingPolicies (r:1 w:0)
	// Storage: SmartContractModule Contracts (r:1 w:0)
	// Storage: SmartContractModule ContractBillingInformationByID (r:1 w:1)
	fn add_nru_reports() -> Weight {
		// Minimum execution time: 65_944 nanoseconds.
		Weight::from_ref_time(67_476_000)
			.saturating_add(RocksDbWeight::get().reads(7))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: TfgridModule TwinIdByAccountID (r:1 w:0)
	// Storage: TfgridModule NodeIdByTwinID (r:1 w:0)
	// Storage: SmartContractModule Contracts (r:1 w:0)
	// Storage: SmartContractModule NodeContractResources (r:0 w:1)
	fn report_contract_resources() -> Weight {
		// Minimum execution time: 42_133 nanoseconds.
		Weight::from_ref_time(43_238_000)
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: SmartContractModule ActiveRentContractForNode (r:1 w:1)
	// Storage: TfgridModule Nodes (r:1 w:0)
	// Storage: TfgridModule Farms (r:1 w:0)
	// Storage: TfgridModule NodePower (r:1 w:0)
	// Storage: SmartContractModule ActiveNodeContracts (r:1 w:0)
	// Storage: TfgridModule TwinIdByAccountID (r:1 w:0)
	// Storage: SmartContractModule ContractID (r:1 w:1)
	// Storage: SmartContractModule BillingFrequency (r:1 w:0)
	// Storage: SmartContractModule ContractsToBillAt (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: SmartContractModule Contracts (r:0 w:1)
	// Storage: SmartContractModule ContractLock (r:0 w:1)
	fn create_rent_contract() -> Weight {
		// Minimum execution time: 63_783 nanoseconds.
		Weight::from_ref_time(72_172_000)
			.saturating_add(RocksDbWeight::get().reads(10))
			.saturating_add(RocksDbWeight::get().writes(5))
	}
	// Storage: SmartContractModule Contracts (r:1 w:1)
	// Storage: TfgridModule Twins (r:1 w:0)
	// Storage: SmartContractModule ActiveNodeContracts (r:1 w:0)
	// Storage: SmartContractModule BillingFrequency (r:1 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: SmartContractModule ContractLock (r:1 w:1)
	// Storage: TfgridModule PricingPolicies (r:1 w:0)
	// Storage: TfgridModule Nodes (r:1 w:0)
	// Storage: TFTPriceModule AverageTftPrice (r:1 w:0)
	// Storage: TFTPriceModule MinTftPrice (r:1 w:0)
	// Storage: TFTPriceModule MaxTftPrice (r:1 w:0)
	// Storage: SmartContractModule ContractBillingInformationByID (r:1 w:1)
	// Storage: SmartContractModule ActiveRentContractForNode (r:0 w:1)
	fn cancel_rent_contract() -> Weight {
		// Minimum execution time: 103_720 nanoseconds.
		Weight::from_ref_time(106_347_000)
			.saturating_add(RocksDbWeight::get().reads(12))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
	// Storage: SmartContractModule SolutionProviderID (r:1 w:1)
	// Storage: SmartContractModule SolutionProviders (r:0 w:1)
	fn create_solution_provider() -> Weight {
		// Minimum execution time: 23_905 nanoseconds.
		Weight::from_ref_time(26_479_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	// Storage: SmartContractModule SolutionProviders (r:1 w:1)
	fn approve_solution_provider() -> Weight {
		// Minimum execution time: 27_670 nanoseconds.
		Weight::from_ref_time(30_376_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: SmartContractModule Contracts (r:1 w:0)
	// Storage: TfgridModule Twins (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: SmartContractModule BillingFrequency (r:1 w:0)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: SmartContractModule ContractLock (r:1 w:1)
	// Storage: TfgridModule PricingPolicies (r:1 w:0)
	// Storage: SmartContractModule ContractBillingInformationByID (r:1 w:1)
	// Storage: TfgridModule Nodes (r:1 w:0)
	// Storage: SmartContractModule NodeContractResources (r:1 w:0)
	// Storage: SmartContractModule ActiveRentContractForNode (r:1 w:0)
	// Storage: TFTPriceModule AverageTftPrice (r:1 w:0)
	// Storage: TFTPriceModule MinTftPrice (r:1 w:0)
	// Storage: TFTPriceModule MaxTftPrice (r:1 w:0)
	// Storage: Balances Locks (r:1 w:1)
	fn bill_contract_for_block() -> Weight {
		// Minimum execution time: 110_792 nanoseconds.
		Weight::from_ref_time(114_740_000)
			.saturating_add(RocksDbWeight::get().reads(15))
			.saturating_add(RocksDbWeight::get().writes(4))
	}
	// Storage: TfgridModule TwinIdByAccountID (r:2 w:0)
	// Storage: SmartContractModule ServiceContractID (r:1 w:1)
	// Storage: SmartContractModule ServiceContracts (r:0 w:1)
	fn service_contract_create() -> Weight {
		// Minimum execution time: 34_632 nanoseconds.
		Weight::from_ref_time(35_528_000)
			.saturating_add(RocksDbWeight::get().reads(3))
			.saturating_add(RocksDbWeight::get().writes(2))
	}
	// Storage: TfgridModule TwinIdByAccountID (r:1 w:0)
	// Storage: SmartContractModule ServiceContracts (r:1 w:1)
	fn service_contract_set_metadata() -> Weight {
		// Minimum execution time: 31_482 nanoseconds.
		Weight::from_ref_time(35_295_000)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: TfgridModule TwinIdByAccountID (r:1 w:0)
	// Storage: SmartContractModule ServiceContracts (r:1 w:1)
	fn service_contract_set_fees() -> Weight {
		// Minimum execution time: 31_024 nanoseconds.
		Weight::from_ref_time(35_066_000)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: TfgridModule TwinIdByAccountID (r:1 w:0)
	// Storage: SmartContractModule ServiceContracts (r:1 w:1)
	fn service_contract_approve() -> Weight {
		// Minimum execution time: 31_558 nanoseconds.
		Weight::from_ref_time(33_168_000)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: TfgridModule TwinIdByAccountID (r:1 w:0)
	// Storage: SmartContractModule ServiceContracts (r:1 w:1)
	fn service_contract_reject() -> Weight {
		// Minimum execution time: 35_446 nanoseconds.
		Weight::from_ref_time(36_650_000)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: TfgridModule TwinIdByAccountID (r:1 w:0)
	// Storage: SmartContractModule ServiceContracts (r:1 w:1)
	fn service_contract_cancel() -> Weight {
		// Minimum execution time: 34_093 nanoseconds.
		Weight::from_ref_time(35_933_000)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: TfgridModule TwinIdByAccountID (r:1 w:0)
	// Storage: SmartContractModule ServiceContracts (r:1 w:1)
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: TfgridModule Twins (r:2 w:0)
	// Storage: System Account (r:1 w:0)
	fn service_contract_bill() -> Weight {
		// Minimum execution time: 53_157 nanoseconds.
		Weight::from_ref_time(61_498_000)
			.saturating_add(RocksDbWeight::get().reads(6))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
	// Storage: SmartContractModule BillingFrequency (r:1 w:1)
	fn change_billing_frequency() -> Weight {
		// Minimum execution time: 33_324 nanoseconds.
		Weight::from_ref_time(33_940_000)
			.saturating_add(RocksDbWeight::get().reads(1))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
}