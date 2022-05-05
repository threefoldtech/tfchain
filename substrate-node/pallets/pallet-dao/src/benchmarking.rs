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

use super::Event as DaoEvent;
use crate::Module as DaoModule;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::{Box, Vec};
use frame_system::EventRecord;
use frame_system::RawOrigin;
use sp_std::mem::size_of;
use sp_std::vec;

use frame_system::Call as SystemCall;
use frame_system::Module as System;

use pallet_tfgrid;

const MAX_BYTES: u32 = 1_024;

benchmarks! {
    propose {
        let b in 1 .. MAX_BYTES;
        let bytes_in_storage = b + size_of::<u32>() as u32;

        let caller: T::AccountId = whitelisted_caller();
        add_council_member::<T>(caller.clone());

        let proposal: T::Proposal = SystemCall::<T>::remark(vec![1; b as usize]).into();
        let threshold = 1;

        let description = "some_description".as_bytes().to_vec();
        let link = "some_link".as_bytes().to_vec();
    }: _ (RawOrigin::Signed(caller.clone()), 1, Box::new(proposal.clone()), description, link, bytes_in_storage)
    verify {
        let proposal_hash = T::Hashing::hash_of(&proposal);
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
            assert_ok!(test_benchmark_create_node_contract::<TestRuntime>());
            assert_ok!(test_benchmark_add_reports::<TestRuntime>());
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
    let location = pallet_tfgrid_types::Location {
        longitude: "12.233213231".as_bytes().to_vec(),
        latitude: "32.323112123".as_bytes().to_vec(),
    };

    let resources = pallet_tfgrid_types::Resources {
        hru: 1,
        sru: 1,
        cru: 1,
        mru: 1,
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
    let mut pub_ips = Vec::new();
    pub_ips.push(pallet_tfgrid_types::PublicIP {
        ip: "1.1.1.0".as_bytes().to_vec(),
        gateway: "1.1.1.1".as_bytes().to_vec(),
        contract_id: 0,
    });

    let su_policy = pallet_tfgrid_types::Policy {
        value: 194400,
        unit: pallet_tfgrid_types::Unit::Gigabytes,
    };
    let nu_policy = pallet_tfgrid_types::Policy {
        value: 50000,
        unit: pallet_tfgrid_types::Unit::Gigabytes,
    };
    let cu_policy = pallet_tfgrid_types::Policy {
        value: 305600,
        unit: pallet_tfgrid_types::Unit::Gigabytes,
    };
    let ipu_policy = pallet_tfgrid_types::Policy {
        value: 69400,
        unit: pallet_tfgrid_types::Unit::Gigabytes,
    };
    let unique_name_policy = pallet_tfgrid_types::Policy {
        value: 13900,
        unit: pallet_tfgrid_types::Unit::Gigabytes,
    };
    let domain_name_policy = pallet_tfgrid_types::Policy {
        value: 27800,
        unit: pallet_tfgrid_types::Unit::Gigabytes,
    };

    let x1 = account("ferdie", 0, 2);
    let x2 = account("eve", 0, 3);

    pallet_tfgrid::Module::<T>::create_pricing_policy(
        RawOrigin::Root.into(),
        "policy_1".as_bytes().to_vec(),
        su_policy,
        cu_policy,
        nu_policy,
        ipu_policy,
        unique_name_policy,
        domain_name_policy,
        x1,
        x2,
        80,
    )
    .unwrap();

    pallet_tfgrid::Module::<T>::create_farm(
        RawOrigin::Signed(source).into(),
        farm_name.as_bytes().to_vec(),
        pub_ips.clone(),
    )
    .unwrap();
}

fn add_council_member<T: Config>(source: T::AccountId) {
    pallet_membership::Module::<T, _>::add_member(RawOrigin::Root.into(), source);
}
