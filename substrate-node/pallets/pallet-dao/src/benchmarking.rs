// This file is part of Substrate.

// Copyright (C) 2022 Threefold Tech
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

use crate::Module as DaoModule;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::{Box, Vec};
use sp_runtime::traits::Bounded;
use frame_system::RawOrigin;
use sp_std::vec;

use frame_system::Call as SystemCall;
use frame_system::Module as System;

use tfchain_support::types::{Location, Resources};

benchmarks! {
    propose {
        let caller: T::AccountId = whitelisted_caller();
        add_council_member::<T>(caller.clone());

        let remark = "remark".as_bytes().to_vec();
        let proposal: T::Proposal = SystemCall::<T>::remark(remark).into();
        let threshold = 1;

        let description = "some_description".as_bytes().to_vec();
        let link = "some_link".as_bytes().to_vec();
    }: _ (RawOrigin::Signed(caller.clone()), 1, Box::new(proposal.clone()), description, link)
    verify {
        assert_eq!(DaoModule::<T>::proposals_list_hashes().len(), 1);
    }

    vote {
        let a1: T::AccountId = account("Alice", 0, 0);
        prepare_farm_and_node::<T>(a1.clone());

        let caller: T::AccountId = whitelisted_caller();
        add_council_member::<T>(caller.clone());

        let hash = create_proposal::<T>(caller.clone());
    }: _ (RawOrigin::Signed(a1.clone()), 1, hash, true)
    verify {
        let voting = DaoModule::<T>::voting(&hash).ok_or("Proposal missing")?;
        assert_eq!(voting.ayes.len(), 1);
    }

    close {
        let a1: T::AccountId = account("Alice", 0, 0);
        prepare_farm_and_node::<T>(a1.clone());

        let caller: T::AccountId = whitelisted_caller();
        add_council_member::<T>(caller.clone());

        let hash = create_proposal::<T>(caller.clone());
        vote_proposal::<T>(a1.clone(), 1, hash, true);

        System::<T>::set_block_number(<T as frame_system::Config>::BlockNumber::max_value());
    }: _ (RawOrigin::Signed(a1.clone()), hash, 0)
    verify {

    }
}

#[cfg(test)]
mod benchmarktests {
    use super::*;
    use crate::mock::{new_test_ext, TestRuntime};
    use frame_support::assert_ok;

    #[test]
    fn test_benchmarks() {
        new_test_ext().execute_with(|| {
            assert_ok!(test_benchmark_propose::<TestRuntime>());
            assert_ok!(test_benchmark_vote::<TestRuntime>());
        });
    }
}

pub fn create_twin<T: Config>(source: T::AccountId) {
    let document = "some_link".as_bytes().to_vec();
    let hash = "some_hash".as_bytes().to_vec();

    pallet_tfgrid::Module::<T>::user_accept_tc(
        RawOrigin::Signed(source.clone()).into(),
        document.clone(),
        hash.clone(),
    )
    .unwrap();
    let ip = "10.2.3.3";
    pallet_tfgrid::Module::<T>::create_twin(
        RawOrigin::Signed(source).into(),
        ip.as_bytes().to_vec(),
    )
    .unwrap();
}

pub fn prepare_farm_and_node<T: Config>(source: T::AccountId) {
    create_twin::<T>(source.clone());
    prepare_farm::<T>(source.clone());

    // random location
    let location = Location {
        longitude: "12.233213231".as_bytes().to_vec(),
        latitude: "32.323112123".as_bytes().to_vec(),
    };

    let resources = Resources {
        hru: 9001778946048,
        sru: 512110190592,
        cru: 64,
        mru: 202802909184,
    };

    let country = "Belgium".as_bytes().to_vec();
    let city = "Ghent".as_bytes().to_vec();
    pallet_tfgrid::Module::<T>::create_node(
        RawOrigin::Signed(source).into(),
        1,
        resources,
        location,
        country,
        city,
        Vec::new(),
        false,
        false,
        "some_serial".as_bytes().to_vec(),
    )
    .unwrap();
}

pub fn prepare_farm<T: Config>(source: T::AccountId) {
    let farm_name = "test_farm";
    pallet_tfgrid::Module::<T>::create_farm(
        RawOrigin::Signed(source).into(),
        farm_name.as_bytes().to_vec(),
        Vec::new(),
    )
    .unwrap();
}

pub fn create_proposal<T: Config>(source: T::AccountId) -> T::Hash {
    let remark = "remark".as_bytes().to_vec();
    let proposal: T::Proposal = SystemCall::<T>::remark(remark).into();
    let threshold = 1;
    let description = "some_description".as_bytes().to_vec();
    let link = "some_link".as_bytes().to_vec();
    let hash = T::Hashing::hash_of(&proposal);

    DaoModule::<T>::propose(
        RawOrigin::Signed(source).into(),
        threshold,
        Box::new(proposal),
        description,
        link
    ).unwrap();

    hash
}

pub fn vote_proposal<T: Config>(source: T::AccountId, farm_id: u32, hash: T::Hash, approve: bool) {
    DaoModule::<T>::vote(
        RawOrigin::Signed(source).into(),
        farm_id,
        hash,
        approve
    ).unwrap();
}

fn add_council_member<T: Config>(source: T::AccountId) {
    pallet_membership::Module::<T, _>::add_member(RawOrigin::Root.into(), source).unwrap();
}
