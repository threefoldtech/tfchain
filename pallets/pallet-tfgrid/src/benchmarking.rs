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

use crate::Module as TfgridModule;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

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

    create_farm {
        let name = "some_name".as_bytes().to_vec();

        let caller: T::AccountId = whitelisted_caller();
        prepare_twin::<T>(caller.clone());
    }: _ (RawOrigin::Signed(caller.clone()), name, Vec::new())
    verify {
        let farm = TfgridModule::<T>::farms(1);
        assert_eq!(
            farm.id, 1
        )
    }

    create_node {
        let a1: T::AccountId = account("Alice", 0, 0);
        prepare_twin::<T>(a1.clone());
        prepare_farm::<T>(a1.clone());

        let country = "Belgium".as_bytes().to_vec();
        let city = "Ghent".as_bytes().to_vec();
    }: _ (
        RawOrigin::Signed(a1.clone()),
        1,
        types::Resources::default(),
        types::Location::default(),
        country,
        city,
        Vec::new(),
        false,
        false,
        "some_serial_2".as_bytes().to_vec()
    )
    verify {
        let node1 = TfgridModule::<T>::nodes(1);
        assert_eq!(
            node1.id, 1
        );
    }

    update_node {
        let a1: T::AccountId = account("Alice", 0, 0);
        prepare_farm_and_node::<T>(a1.clone());

        let country = "Belgium".as_bytes().to_vec();
        let city = "Ghent".as_bytes().to_vec();
    }: _ (
        RawOrigin::Signed(a1.clone()),
        1,
        1,
        types::Resources::default(),
        types::Location::default(),
        country,
        city,
        Vec::new(),
        false,
        false,
        "some_serial_2".as_bytes().to_vec()
    )
    verify {
        let node1 = TfgridModule::<T>::nodes(1);
        assert_eq!(
            node1.serial_number, "some_serial_2".as_bytes().to_vec()
        );
    }

    report_uptime {
        let a1: T::AccountId = account("Alice", 0, 0);
        prepare_farm_and_node::<T>(a1.clone());


        let country = "Belgium".as_bytes().to_vec();
        let city = "Ghent".as_bytes().to_vec();
    }: _ (
        RawOrigin::Signed(a1.clone()),
        0
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{new_test_ext, TestRuntime};
    use frame_support::assert_ok;

    #[test]
    fn test_benchmarks() {
        new_test_ext().execute_with(|| {
            assert_ok!(test_benchmark_create_twin::<TestRuntime>());
            assert_ok!(test_benchmark_create_farm::<TestRuntime>());
            assert_ok!(test_benchmark_create_node::<TestRuntime>());
            assert_ok!(test_benchmark_update_node::<TestRuntime>());
            assert_ok!(test_benchmark_report_uptime::<TestRuntime>());
        });
    }
}

pub fn prepare_twin<T: Config>(source: T::AccountId) {
    let document = "some_link".as_bytes().to_vec();
    let hash = "some_hash".as_bytes().to_vec();

    TfgridModule::<T>::user_accept_tc(
        RawOrigin::Signed(source.clone()).into(),
        document.clone(),
        hash.clone(),
    )
    .unwrap();
    let ip = "10.2.3.3";
    TfgridModule::<T>::create_twin(RawOrigin::Signed(source).into(), ip.as_bytes().to_vec())
        .unwrap();
}

pub fn prepare_farm_and_node<T: Config>(source: T::AccountId) {
    prepare_twin::<T>(source.clone());
    prepare_farm::<T>(source.clone());

    // random location
    let location = types::Location {
        longitude: "12.233213231".as_bytes().to_vec(),
        latitude: "32.323112123".as_bytes().to_vec(),
    };

    let resources = types::Resources {
        hru: 1,
        sru: 1,
        cru: 1,
        mru: 1,
    };

    let country = "Belgium".as_bytes().to_vec();
    let city = "Ghent".as_bytes().to_vec();
    TfgridModule::<T>::create_node(
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
    pub_ips.push(types::PublicIP {
        ip: "1.1.1.0".as_bytes().to_vec(),
        gateway: "1.1.1.1".as_bytes().to_vec(),
        contract_id: 0,
    });

    let su_policy = types::Policy {
        value: 194400,
        unit: types::Unit::Gigabytes,
    };
    let nu_policy = types::Policy {
        value: 50000,
        unit: types::Unit::Gigabytes,
    };
    let cu_policy = types::Policy {
        value: 305600,
        unit: types::Unit::Gigabytes,
    };
    let ipu_policy = types::Policy {
        value: 69400,
        unit: types::Unit::Gigabytes,
    };
    let unique_name_policy = types::Policy {
        value: 13900,
        unit: types::Unit::Gigabytes,
    };
    let domain_name_policy = types::Policy {
        value: 27800,
        unit: types::Unit::Gigabytes,
    };

    let x1 = account("ferdie", 0, 2);
    let x2 = account("eve", 0, 3);

    TfgridModule::<T>::create_pricing_policy(
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

    TfgridModule::<T>::create_farm(
        RawOrigin::Signed(source).into(),
        farm_name.as_bytes().to_vec(),
        pub_ips.clone(),
    )
    .unwrap();
}
