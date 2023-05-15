#![cfg(feature = "runtime-benchmarks")]

use super::*;

use crate::Pallet as SmartContractModule;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use sp_std::{
    convert::{TryFrom, TryInto},
    fmt::Debug,
    vec,
};

use pallet_tfgrid::{
    types::{self as pallet_tfgrid_types, LocationInput},
    CityNameInput, CountryNameInput, DocumentHashInput, DocumentLinkInput, LatitudeInput,
    LongitudeInput, PkInput, RelayInput, ResourcesInput,
};
use tfchain_support::{resources::Resources, types::IP4};
const GIGABYTE: u64 = 1024 * 1024 * 1024;

benchmarks! {
    where_clause {
        where
        <T as pallet_timestamp::Config>::Moment: TryFrom<u64>,
        <<T as pallet_timestamp::Config>::Moment as  TryFrom<u64>>::Error: Debug,
    }

    create_node_contract {
        let a1: T::AccountId = account("Alice", 0, 0);
        prepare_farm_and_node::<T>(a1);

        let caller: T::AccountId = whitelisted_caller();
        create_twin::<T>(caller.clone());
    }: _ (RawOrigin::Signed(
        caller.clone()),
        1,
        "858f8fb2184b15ecb8c0be8b95398c81".as_bytes().to_vec().try_into().unwrap(),
        "some_data".as_bytes().to_vec().try_into().unwrap(),
        1,
        None
    )
    verify {
        let contract = SmartContractModule::<T>::contracts(1).unwrap();
        assert_eq!(
            contract.contract_id, 1
        );
    }

    add_nru_reports {
        let stamp: u64 = 1628082000 * 1000;
        pallet_timestamp::Pallet::<T>::set_timestamp(stamp.try_into().unwrap());

        let a1: T::AccountId = account("Alice", 0, 0);
        prepare_farm_and_node::<T>(a1.clone());

        let caller: T::AccountId = whitelisted_caller();
        create_twin::<T>(caller.clone());
        create_contract::<T>(caller.clone());

        let mut reports = Vec::new();

        reports.push(types::NruConsumption {
            contract_id: 1,
            timestamp: 1628082000 * 1000,
            window: 1000,
            nru: 10 * GIGABYTE,
        });

    }: _ (RawOrigin::Signed(a1.clone()), reports)
    verify {
        let contract = SmartContractModule::<T>::contracts(1).unwrap();
        assert_eq!(
            contract.contract_id, 1
        );
    }

    bill_contract_for_block {
        let stamp: u64 = 1628082000 * 1000;
        pallet_timestamp::Pallet::<T>::set_timestamp(stamp.try_into().unwrap());
        let a1: T::AccountId = account("Alice", 0, 0);
        prepare_farm_and_node::<T>(a1.clone());

        let caller: T::AccountId = whitelisted_caller();
        create_twin::<T>(caller.clone());
        create_contract::<T>(caller.clone());

        let mut reports = Vec::new();

        reports.push(types::Consumption {
            contract_id: 1,
            cru: 2,
            hru: 0,
            mru: 8 * GIGABYTE,
            sru: 25 * GIGABYTE,
            nru: 0,
            timestamp: 0,
        });

        push_contract_resources::<T>(a1.clone());

        let stamp: u64 = 1628082000 * 1000 * 10 * 6000;
        pallet_timestamp::Pallet::<T>::set_timestamp(stamp.try_into().unwrap());
        // run_to_block::<T>(10);
    }: _ (RawOrigin::Signed(a1.clone()), 0, 1) // Update here
    verify {
        let contract = SmartContractModule::<T>::contracts(1).unwrap();
        assert_eq!(
            contract.contract_id, 1
        );
    }
}

pub fn create_twin<T: Config>(source: T::AccountId) {
    assert_ok!(pallet_tfgrid::Pallet::<T>::user_accept_tc(
        RawOrigin::Signed(source.clone()).into(),
        get_document_link_input(b"some_link"),
        get_document_hash_input(b"some_hash"),
    ));

    let relay = get_relay_input(b"somerelay.io");
    let pk =
        get_public_key_input(b"0x6c8fd181adc178cea218e168e8549f0b0ff30627c879db9eac4318927e87c901");
    assert_ok!(pallet_tfgrid::Pallet::<T>::create_twin(
        RawOrigin::Signed(source).into(),
        relay,
        pk
    ));
}

pub fn prepare_farm_and_node<T: Config>(source: T::AccountId) {
    create_twin::<T>(source.clone());
    prepare_farm::<T>(source.clone());

    let resources = ResourcesInput {
        hru: 1024 * GIGABYTE,
        sru: 512 * GIGABYTE,
        cru: 8,
        mru: 16 * GIGABYTE,
    };

    // random location
    let location = LocationInput {
        city: get_city_name_input(b"Ghent"),
        country: get_country_name_input(b"Belgium"),
        latitude: get_latitude_input(b"12.233213231"),
        longitude: get_longitude_input(b"32.323112123"),
    };

    assert_ok!(pallet_tfgrid::Pallet::<T>::create_node(
        RawOrigin::Signed(source.clone()).into(),
        1,
        resources,
        location,
        Vec::new().try_into().unwrap(),
        false,
        false,
        None,
    ));
}

pub fn prepare_farm<T: Config>(source: T::AccountId) {
    let farm_name = "testfarm";
    let mut pub_ips = Vec::new();
    pub_ips.push(IP4 {
        ip: "185.206.122.33/24".as_bytes().to_vec().try_into().unwrap(),
        gw: "185.206.122.1".as_bytes().to_vec().try_into().unwrap(),
    });
    pub_ips.push(IP4 {
        ip: "185.206.122.34/24".as_bytes().to_vec().try_into().unwrap(),
        gw: "185.206.122.1".as_bytes().to_vec().try_into().unwrap(),
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

    assert_ok!(pallet_tfgrid::Pallet::<T>::create_pricing_policy(
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
    ));

    assert_ok!(pallet_tfgrid::Pallet::<T>::create_farm(
        RawOrigin::Signed(source).into(),
        farm_name.as_bytes().to_vec().try_into().unwrap(),
        pub_ips.clone().try_into().unwrap(),
    ));
}

pub fn create_contract<T: Config>(source: T::AccountId) {
    assert_ok!(SmartContractModule::<T>::create_node_contract(
        RawOrigin::Signed(source).into(),
        1,
        "858f8fb2184b15ecb8c0be8b95398c81"
            .as_bytes()
            .to_vec()
            .try_into()
            .unwrap(),
        "some_data123".as_bytes().to_vec().try_into().unwrap(),
        0,
        None,
    ));
}

pub fn push_contract_resources<T: Config>(source: T::AccountId) {
    let contract_resources = vec![types::ContractResources {
        contract_id: 1,
        used: Resources {
            sru: 150 * GIGABYTE,
            cru: 16,
            mru: 8 * GIGABYTE,
            hru: 0,
        },
    }];

    assert_ok!(SmartContractModule::<T>::report_contract_resources(
        RawOrigin::Signed(source).into(),
        contract_resources,
    ));
}

pub(crate) fn get_city_name_input(city_input: &[u8]) -> CityNameInput {
    BoundedVec::try_from(city_input.to_vec()).expect("Invalid city name input.")
}

pub(crate) fn get_country_name_input(country_input: &[u8]) -> CountryNameInput {
    BoundedVec::try_from(country_input.to_vec()).expect("Invalid country name input.")
}

pub(crate) fn get_latitude_input(latitude_input: &[u8]) -> LatitudeInput {
    BoundedVec::try_from(latitude_input.to_vec()).expect("Invalid latitude input.")
}

pub(crate) fn get_longitude_input(longitude_input: &[u8]) -> LongitudeInput {
    BoundedVec::try_from(longitude_input.to_vec()).expect("Invalid longitude input.")
}

pub(crate) fn get_document_link_input(document_link_input: &[u8]) -> DocumentLinkInput {
    BoundedVec::try_from(document_link_input.to_vec()).expect("Invalid document link input.")
}

pub(crate) fn get_document_hash_input(document_hash_input: &[u8]) -> DocumentHashInput {
    BoundedVec::try_from(document_hash_input.to_vec()).expect("Invalid document hash input.")
}

pub(crate) fn get_relay_input(relay_input: &[u8]) -> RelayInput {
    Some(BoundedVec::try_from(relay_input.to_vec()).expect("Invalid relay input."))
}

pub(crate) fn get_public_key_input(pk_input: &[u8]) -> PkInput {
    Some(BoundedVec::try_from(pk_input.to_vec()).expect("Invalid public key input."))
}
