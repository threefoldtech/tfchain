#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as SmartContractModule;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::{
    assert_ok,
    traits::{OnFinalize, OnInitialize},
};
use frame_system::{EventRecord, Pallet as System, RawOrigin};
use pallet_balances::Pallet as Balances;
use pallet_tfgrid::{
    types::{self as tfgrid_types, LocationInput},
    CityNameInput, CountryNameInput, DocumentHashInput, DocumentLinkInput, LatitudeInput,
    LongitudeInput, Pallet as Tfgrid, PkInput, RelayInput, ResourcesInput,
};
use pallet_timestamp::Pallet as Timestamp;
use sp_runtime::traits::{Bounded, One, StaticLookup};
use sp_std::{
    convert::{TryFrom, TryInto},
    fmt::Debug,
    vec,
};
use tfchain_support::{
    resources::Resources,
    types::{FarmCertification, NodeCertification, IP4},
};

const GIGABYTE: u64 = 1024 * 1024 * 1024;
const TIMESTAMP_INIT_MILLISECS: u64 = 1628082000 * 1000;

benchmarks! {
    where_clause {
        where
        <T as pallet_timestamp::Config>::Moment: TryFrom<u64>,
        <<T as pallet_timestamp::Config>::Moment as  TryFrom<u64>>::Error: Debug,
    }

    create_node_contract {
        let farmer: T::AccountId = account("Alice", 0, 0);
        prepare_farm_with_node::<T>(farmer);

        let caller: T::AccountId = whitelisted_caller();
        create_twin::<T>(caller.clone());
    }: _(RawOrigin::Signed(
        caller.clone()),
        1,
        b"858f8fb2184b15ecb8c0be8b95398c81".to_vec().try_into().unwrap(),
        b"some_data".to_vec().try_into().unwrap(),
        1,
        None
    )
    verify {
        assert!(SmartContractModule::<T>::contracts(1).is_some());
        let contract = SmartContractModule::<T>::contracts(1).unwrap();
        assert_eq!(
            contract.contract_id, 1
        );
        assert_last_event::<T>(Event::ContractCreated(contract).into());
    }

    add_nru_reports {
        let farmer: T::AccountId = account("Alice", 0, 0);
        prepare_farm_with_node::<T>(farmer.clone());

        let caller: T::AccountId = whitelisted_caller();
        create_twin::<T>(caller.clone());
        create_contract::<T>(caller.clone());
        let contract_id = 1;

        let report = types::NruConsumption {
            contract_id: contract_id,
            timestamp: TIMESTAMP_INIT_MILLISECS,
            window: 1000,
            nru: 10 * GIGABYTE,
        };
        let mut reports = Vec::new();
        reports.push(report.clone());

    }: _(RawOrigin::Signed(farmer.clone()), reports)
    verify {
        assert!(SmartContractModule::<T>::contracts(1).is_some());
        let contract = SmartContractModule::<T>::contracts(contract_id).unwrap();
        assert_eq!(
            contract.contract_id, contract_id
        );
        assert_last_event::<T>(Event::NruConsumptionReportReceived(report).into());
    }

    bill_contract_for_block {
        let farmer: T::AccountId = account("Alice", 0, 0);
        prepare_farm_with_node::<T>(farmer.clone());

        let caller: T::AccountId = whitelisted_caller();
        let caller_lookup = T::Lookup::unlookup(caller.clone());
        let balance_init_amount = <T as pallet_balances::Config>::Balance::saturated_from(100000000 as u128);
        Balances::<T>::set_balance(RawOrigin::Root.into(), caller_lookup, balance_init_amount, balance_init_amount).unwrap();
        create_twin::<T>(caller.clone());
        create_contract::<T>(caller.clone());
        let contract_id = 1;

        push_contract_used_resources_report::<T>(farmer.clone());
        push_contract_nru_consumption_report::<T>(farmer.clone());

    }: _(RawOrigin::Signed(farmer.clone()), contract_id)
    verify {
        assert!(SmartContractModule::<T>::contracts(contract_id).is_some());
        let contract = SmartContractModule::<T>::contracts(contract_id).unwrap();
        assert_eq!(
            contract.contract_id, contract_id
        );
        let amount_billed = 3175199 as u128;
        let contract_bill = types::ContractBill {
            contract_id,
            timestamp: <Timestamp<T>>::get().saturated_into::<u64>() / 1000,
            discount_level: types::DiscountLevel::None,
            amount_billed,
        };
        assert_last_event::<T>(Event::ContractBilled(contract_bill).into());
        assert_eq!((Balances::<T>::free_balance(&caller) - Balances::<T>::usable_balance(&caller)).saturated_into::<u128>(), amount_billed);
    }

    // Calling the `impl_benchmark_test_suite` macro inside the `benchmarks`
    // block will generate one #[test] function per benchmark
    impl_benchmark_test_suite! (SmartContractModule, crate::mock::new_test_ext(), crate::mock::TestRuntime)
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    let events = System::<T>::events();
    let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}

pub fn run_to_block<T: Config>(n: T::BlockNumber) {
    while System::<T>::block_number() < n {
        crate::Pallet::<T>::on_finalize(System::<T>::block_number());
        System::<T>::on_finalize(System::<T>::block_number());
        System::<T>::set_block_number(System::<T>::block_number() + One::one());
        System::<T>::on_initialize(System::<T>::block_number());
        crate::Pallet::<T>::on_initialize(System::<T>::block_number());
    }
}

pub fn prepare_farm_with_node<T: Config>(source: T::AccountId) {
    create_pricing_policy::<T>();
    create_farming_policy::<T>();
    create_twin::<T>(source.clone());
    create_farm::<T>(source.clone());
    create_node::<T>(source.clone());
}

fn create_pricing_policy<T: Config>() {
    let su_policy = tfgrid_types::Policy {
        value: 194400,
        unit: tfgrid_types::Unit::Gigabytes,
    };
    let nu_policy = tfgrid_types::Policy {
        value: 50000,
        unit: tfgrid_types::Unit::Gigabytes,
    };
    let cu_policy = tfgrid_types::Policy {
        value: 305600,
        unit: tfgrid_types::Unit::Gigabytes,
    };
    let ipu_policy = tfgrid_types::Policy {
        value: 69400,
        unit: tfgrid_types::Unit::Gigabytes,
    };
    let unique_name_policy = tfgrid_types::Policy {
        value: 13900,
        unit: tfgrid_types::Unit::Gigabytes,
    };
    let domain_name_policy = tfgrid_types::Policy {
        value: 27800,
        unit: tfgrid_types::Unit::Gigabytes,
    };

    let x1 = account("Ferdie", 0, 2);
    let x2 = account("Eve", 0, 3);

    assert_ok!(Tfgrid::<T>::create_pricing_policy(
        RawOrigin::Root.into(),
        b"policy_1".to_vec(),
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
}

fn create_farming_policy<T: Config>() {
    let name = b"fp".to_vec();
    assert_ok!(Tfgrid::<T>::create_farming_policy(
        RawOrigin::Root.into(),
        name,
        12,
        15,
        10,
        8,
        9999,
        <T as frame_system::Config>::BlockNumber::max_value(),
        true,
        true,
        NodeCertification::Diy,
        FarmCertification::NotCertified,
    ));
}

pub fn create_twin<T: Config>(source: T::AccountId) {
    assert_ok!(Tfgrid::<T>::user_accept_tc(
        RawOrigin::Signed(source.clone()).into(),
        get_document_link_input(b"some_link"),
        get_document_hash_input(b"some_hash"),
    ));

    let relay = get_relay_input(b"somerelay.io");
    let pk =
        get_public_key_input(b"0x6c8fd181adc178cea218e168e8549f0b0ff30627c879db9eac4318927e87c901");

    assert_ok!(Tfgrid::<T>::create_twin(
        RawOrigin::Signed(source).into(),
        relay,
        pk
    ));
}

pub fn create_farm<T: Config>(source: T::AccountId) {
    let farm_name = b"testfarm";
    let mut pub_ips = Vec::new();
    pub_ips.push(IP4 {
        ip: b"185.206.122.33/24".to_vec().try_into().unwrap(),
        gw: b"185.206.122.1".to_vec().try_into().unwrap(),
    });
    pub_ips.push(IP4 {
        ip: b"185.206.122.34/24".to_vec().try_into().unwrap(),
        gw: b"185.206.122.1".to_vec().try_into().unwrap(),
    });

    assert_ok!(pallet_tfgrid::Pallet::<T>::create_farm(
        RawOrigin::Signed(source).into(),
        farm_name.to_vec().try_into().unwrap(),
        pub_ips.clone().try_into().unwrap(),
    ));
}

pub fn create_node<T: Config>(source: T::AccountId) {
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

pub fn create_contract<T: Config>(source: T::AccountId) {
    assert_ok!(SmartContractModule::<T>::create_node_contract(
        RawOrigin::Signed(source).into(),
        1,
        b"858f8fb2184b15ecb8c0be8b95398c81"
            .to_vec()
            .try_into()
            .unwrap(),
        "some_data123".as_bytes().to_vec().try_into().unwrap(),
        0,
        None,
    ));
}

pub fn push_contract_used_resources_report<T: Config>(source: T::AccountId) {
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

pub fn push_contract_nru_consumption_report<T: Config>(source: T::AccountId) {
    let nru_consumption = types::NruConsumption {
        contract_id: 1,
        timestamp: TIMESTAMP_INIT_MILLISECS,
        window: BillingFrequency::<T>::get() * 6,
        nru: 10 * GIGABYTE,
    };

    let mut reports = Vec::new();
    reports.push(nru_consumption.clone());

    assert_ok!(SmartContractModule::<T>::add_nru_reports(
        RawOrigin::Signed(source).into(),
        reports,
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
    Some(BoundedVec::try_from(pk_input.to_vec()).expect("Invalid document public key input."))
}
