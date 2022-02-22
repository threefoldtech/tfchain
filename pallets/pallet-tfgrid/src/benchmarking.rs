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

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use crate::Module as TfgridModule;

benchmarks! {
	create_twin {
        let document = "some_link".as_bytes().to_vec();
        let hash = "some_hash".as_bytes().to_vec();
    
		let caller: T::AccountId = whitelisted_caller();
        TfgridModule::<T>::user_accept_tc(
            RawOrigin::Signed(caller.clone()).into(),
            document,
            hash,
        ).unwrap();
        let ip = "10.2.3.3";

	}: _ (RawOrigin::Signed(caller.clone()), ip.as_bytes().to_vec())
	verify {
        let twin = TfgridModule::<T>::twins(1);
        assert_eq!(
            twin.id, 1  
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
			assert_ok!(test_benchmark_create_twin::<TestRuntime>());
		});
	}
}