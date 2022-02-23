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

use crate::Module as SmartContractModule;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::{Box, Vec};
use frame_system::RawOrigin;
use sp_std::vec;

use pallet_tfgrid;

benchmarks! {
    create_node_contract {
        let a1: T::AccountId = account("Alice", 0, 0);
        prepare_farm_and_node::<T>(a1);

        let caller: T::AccountId = whitelisted_caller();
        create_twin::<T>(caller.clone());
    }: _ (RawOrigin::Signed(caller.clone()), 1, "some_data".as_bytes().to_vec(), "hash".as_bytes().to_vec(), 1)
    verify {
        let contract = SmartContractModule::<T>::contracts(1);
        assert_eq!(
            contract.contract_id, 1
        );
    }

    add_reports {
        let a1: T::AccountId = account("Alice", 0, 0);
        prepare_farm_and_node::<T>(a1.clone());

        let caller: T::AccountId = whitelisted_caller();
        create_twin::<T>(caller.clone());
        create_contract::<T>(caller.clone());

        let mut reports = Vec::new();

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

    }: _ (RawOrigin::Signed(a1.clone()), reports)
    verify {
        let contract = SmartContractModule::<T>::contracts(1);
        assert_eq!(
            contract.contract_id, 1
        );
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
    )
    .unwrap();

    pallet_tfgrid::Module::<T>::create_farm(
        RawOrigin::Signed(source).into(),
        farm_name.as_bytes().to_vec(),
        pub_ips.clone(),
    )
    .unwrap();
}

pub fn create_contract<T: Config>(source: T::AccountId) {
    SmartContractModule::<T>::create_node_contract(
        RawOrigin::Signed(source).into(),
        1,
        "some_data123".as_bytes().to_vec(),
        "hash123".as_bytes().to_vec(),
        0
    ).unwrap()
}