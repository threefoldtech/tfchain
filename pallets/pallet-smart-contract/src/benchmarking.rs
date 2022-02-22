// This file is part of Substrate.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Vesting pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, whitelisted_caller, account};
use frame_system::RawOrigin;
use frame_support::traits::{Vec, Box};
use crate::Module as SmartContractModule;
use sp_std::vec;

benchmarks! {
	create_node_contract {
		let caller: T::AccountId = whitelisted_caller();
	}: _ (RawOrigin::Signed(caller.clone()), 1, "some_data".as_bytes().to_vec(), "hash".as_bytes().to_vec(), 1)
	verify {
        let contract = SmartContractModule::<T>::contracts(1);
        assert_eq!(
            contract.contract_id, 1  
        );
	}

    add_reports {
		let caller: T::AccountId = whitelisted_caller();

        let mut reports = Vec::new();

        for i in 0..10 {
            let gigabyte = 1000 * 1000 * 1000;
            reports.push(types::Consumption {
                contract_id: 1,
                cru: 2,
                hru: 0,
                mru: 8 * gigabyte,
                sru: 25 * gigabyte,
                nru: 0,
                timestamp: 0,
            });
        }

	}: _ (RawOrigin::Signed(caller.clone()), reports)
	verify {
        let contract = SmartContractModule::<T>::contracts(1);
        assert_eq!(
            contract.contract_id, 1
        );
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::assert_ok;
    use crate::mock::{TestRuntime, new_test_ext};

	#[test]
	fn test_benchmarks() {
		new_test_ext().execute_with(|| {
            {
                let a1 = account("Alice", 0, 0);
                crate::tests::_prepare_farm_and_node(a1);
                let caller = whitelisted_caller();
                crate::tests::create_twin_and_node(caller);
                assert_ok!(test_benchmark_create_node_contract::<TestRuntime>());
            }
            
            {
                // crate::tests::prepare_twins();
                // crate::tests::prepare_farm();
                // let caller = whitelisted_caller();
                // crate::tests::create_twin_and_node(caller);
                // crate::tests::create_contract_bob();
                // assert_ok!(test_benchmark_add_reports::<TestRuntime>());
            }
		});
	}
}