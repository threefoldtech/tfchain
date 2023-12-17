#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as SmartContractModule;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::{
    assert_ok,
    traits::{OnFinalize, OnInitialize},
    BoundedVec,
};
use frame_system::{pallet_prelude::BlockNumberFor, EventRecord, Pallet as System, RawOrigin};
use pallet_balances::Pallet as Balances;
use pallet_tfgrid::{
    types::{self as tfgrid_types, LocationInput},
    CityNameInput, CountryNameInput, DocumentHashInput, DocumentLinkInput, Gw4Input, Ip4Input,
    LatitudeInput, LongitudeInput, Pallet as TfgridModule, PkInput, RelayInput, ResourcesInput,
};
use sp_runtime::{
    traits::{Bounded, One, StaticLookup},
    SaturatedConversion,
};
use sp_std::{
    convert::{TryFrom, TryInto},
    fmt::Debug,
    vec,
    vec::Vec,
};
use tfchain_support::{
    resources::Resources,
    types::{FarmCertification, NodeCertification, IP4},
};

const GIGABYTE: u64 = 1024 * 1024 * 1024;

benchmarks! {
    where_clause {
        where
        <T as pallet_timestamp::Config>::Moment: TryFrom<u64>,
        <<T as pallet_timestamp::Config>::Moment as TryFrom<u64>>::Error: Debug,
        T: pallet_balances::Config<Balance = BalanceOf<T>>,
    }

    // create_node_contract()
    create_node_contract {
        let farmer: T::AccountId = account("Alice", 0, 0);
        _prepare_farm_with_node::<T>(farmer);
        let node_id = 1;

        let caller: T::AccountId = whitelisted_caller();
        _create_twin::<T>(caller.clone());
    }: _(
        RawOrigin::Signed(caller.clone()),
        node_id,
        get_deployment_hash_input(b"858f8fb2184b15ecb8c0be8b95398c81"),
        get_deployment_data_input::<T>(b"some_data"),
        1,
        None
    )
    verify {
        let contract_id = 1;
        assert!(SmartContractModule::<T>::contracts(contract_id).is_some());
        let contract = SmartContractModule::<T>::contracts(contract_id).unwrap();
        assert_eq!(
            contract.contract_id, contract_id
        );
        assert_last_event::<T>(Event::ContractCreated(contract).into());
    }

    // update_node_contract()
    update_node_contract {
        let farmer: T::AccountId = account("Alice", 0, 0);
        _prepare_farm_with_node::<T>(farmer.clone());

        let caller: T::AccountId = whitelisted_caller();
        _create_twin::<T>(caller.clone());
        _create_node_contract::<T>(caller.clone());
        let contract_id = 1;
        let new_deployment_hash = get_deployment_hash_input(b"858f8fb2184b15ecb8c0be8b95398c82");
    }: _(
        RawOrigin::Signed(caller.clone()),
        contract_id,
        new_deployment_hash,
        get_deployment_data_input::<T>(b"some_data")
    )
    verify {
        assert!(SmartContractModule::<T>::contracts(contract_id).is_some());
        let contract = SmartContractModule::<T>::contracts(contract_id).unwrap();
        assert_eq!(
            contract.contract_id, contract_id
        );
        assert_last_event::<T>(Event::ContractUpdated(contract).into());
        let node_id = 1;
        assert_eq!(SmartContractModule::<T>::node_contract_by_hash(node_id, new_deployment_hash), contract_id);
        let old_deployment_hash = get_deployment_hash_input(b"858f8fb2184b15ecb8c0be8b95398c81");
        assert_eq!(SmartContractModule::<T>::node_contract_by_hash(node_id, old_deployment_hash), 0);
    }

    // cancel_contract()
    cancel_contract {
        let farmer: T::AccountId = account("Alice", 0, 0);
        _prepare_farm_with_node::<T>(farmer.clone());

        let caller: T::AccountId = whitelisted_caller();
        _create_twin::<T>(caller.clone());
        _create_node_contract::<T>(caller.clone());
        let contract_id = 1;

    }: _(RawOrigin::Signed(caller.clone()), contract_id)
    verify {
        assert!(SmartContractModule::<T>::contracts(contract_id).is_none());
        let node_id = 1;
        let twin_id = 2;
        assert_last_event::<T>(Event::NodeContractCanceled {
            contract_id,
            node_id,
            twin_id,
         }.into());
    }

    // create_name_contract()
    create_name_contract {
        let caller: T::AccountId = whitelisted_caller();
        _create_twin::<T>(caller.clone());
    }: _(RawOrigin::Signed(caller.clone()), b"foobar".to_vec())
    verify {
        let contract_id = 1;
        assert!(SmartContractModule::<T>::contracts(contract_id).is_some());
        let contract = SmartContractModule::<T>::contracts(contract_id).unwrap();
        assert_eq!(
            contract.contract_id, contract_id
        );
        assert_last_event::<T>(Event::ContractCreated(contract).into());
    }

    cancel_name_contract {
        _create_pricing_policy::<T>();
        let caller: T::AccountId = whitelisted_caller();
        _create_twin::<T>(caller.clone());
        _create_name_contract::<T>(caller.clone());
        let contract_id = 1;
    }: cancel_contract(RawOrigin::Signed(caller.clone()), contract_id)
    verify {
        assert!(SmartContractModule::<T>::contracts(contract_id).is_none());
        assert_last_event::<T>(Event::NameContractCanceled {
            contract_id,
         }.into());
    }

    // add_nru_reports()
    add_nru_reports {
        let farmer: T::AccountId = account("Alice", 0, 0);
        _prepare_farm_with_node::<T>(farmer.clone());

        let user: T::AccountId = account("Bob", 0, 1);
        _create_twin::<T>(user.clone());
        _create_node_contract::<T>(user.clone());
        let contract_id = 1;

        let report = types::NruConsumption {
            contract_id: contract_id,
            timestamp: SmartContractModule::<T>::get_current_timestamp_in_secs(),
            window: 1000,
            nru: 10 * GIGABYTE,
        };
        let mut reports = Vec::new();
        reports.push(report.clone());
    }: _(RawOrigin::Signed(farmer), reports)
    verify {
        assert!(SmartContractModule::<T>::contracts(contract_id).is_some());
        let contract = SmartContractModule::<T>::contracts(contract_id).unwrap();
        assert_eq!(
            contract.contract_id, contract_id
        );
        assert_last_event::<T>(Event::NruConsumptionReportReceived(report).into());
    }

    // report_contract_resources()
    report_contract_resources {
        let farmer: T::AccountId = account("Alice", 0, 0);
        _prepare_farm_with_node::<T>(farmer.clone());

        let user: T::AccountId = account("Bob", 0, 1);
        _create_twin::<T>(user.clone());
        _create_node_contract::<T>(user.clone());
        let contract_id = 1;

        let contract_resource = types::ContractResources {
            contract_id,
            used: Resources {
                sru: 150 * GIGABYTE,
                cru: 16,
                mru: 8 * GIGABYTE,
                hru: 0,
            }
        };
        let contract_resources = vec![contract_resource.clone()];
    }: _(RawOrigin::Signed(farmer), contract_resources)
    verify {
        assert_eq!(
            SmartContractModule::<T>::node_contract_resources(contract_id),
            contract_resource
        );
        assert_last_event::<T>(Event::UpdatedUsedResources(contract_resource).into());
    }

    // create_rent_contract()
    create_rent_contract {
        let farmer: T::AccountId = account("Alice", 0, 0);
        _prepare_farm_with_node::<T>(farmer.clone());
        let node_id = 1;

        let caller: T::AccountId = whitelisted_caller();
        _create_twin::<T>(caller.clone());
    }: _(RawOrigin::Signed(caller.clone()), node_id, None)
    verify {
        let contract_id = 1;
        assert!(SmartContractModule::<T>::contracts(contract_id).is_some());
        let contract = SmartContractModule::<T>::contracts(contract_id).unwrap();
        assert_eq!(
            contract.contract_id, contract_id
        );
        assert_last_event::<T>(Event::ContractCreated(contract).into());
    }

    cancel_rent_contract {
        let farmer: T::AccountId = account("Alice", 0, 0);
        _prepare_farm_with_node::<T>(farmer.clone());

        let caller: T::AccountId = whitelisted_caller();
        _create_twin::<T>(caller.clone());
        _create_rent_contract::<T>(caller.clone());
        let contract_id = 1;
    }: cancel_contract(RawOrigin::Signed(caller.clone()), contract_id)
    verify {
        assert!(SmartContractModule::<T>::contracts(contract_id).is_none());
        assert_last_event::<T>(Event::RentContractCanceled {
            contract_id,
         }.into());
    }

    // create_solution_provider()
    create_solution_provider {
        let provider1 = super::types::Provider {
            take: 10,
            who: account("Alice", 0, 0),
        };
        let provider2 = super::types::Provider {
            take: 10,
            who: account("Bob", 0, 1),
        };
        let providers = vec![provider1, provider2];

        let caller: T::AccountId = whitelisted_caller();
        _create_twin::<T>(caller.clone());
    }: _(
        RawOrigin::Signed(caller.clone()),
        b"some_description".to_vec(),
        b"some_link".to_vec(),
        providers.clone()
    )
    verify {
        let solution_provider_id = 1;
        assert!(SmartContractModule::<T>::solution_providers(solution_provider_id).is_some());
        let solution_provider = SmartContractModule::<T>::solution_providers(solution_provider_id).unwrap();
        assert_eq!(solution_provider.providers, providers);
        assert_last_event::<T>(Event::SolutionProviderCreated(solution_provider).into());
    }

    // approve_solution_provider()
    approve_solution_provider {
        let provider: T::AccountId = account("Alice", 0, 0);
        _create_twin::<T>(provider.clone());
        _create_solution_provider::<T>(provider.clone());
        let solution_provider_id = 1;
        let approve = true;

    }: _(RawOrigin::Root, solution_provider_id, approve)
    verify {
        assert!(SmartContractModule::<T>::solution_providers(solution_provider_id).is_some());
        let solution_provider = SmartContractModule::<T>::solution_providers(solution_provider_id).unwrap();
        assert_eq!(solution_provider.approved, approve);
        assert_last_event::<T>(Event::SolutionProviderApproved(solution_provider_id, approve).into());
    }

    // bill_contract_for_block()
    bill_contract_for_block {
        let farmer: T::AccountId = account("Alice", 0, 0);
        _prepare_farm_with_node::<T>(farmer.clone());

        let user: T::AccountId = account("Bob", 0, 1);
        let user_lookup = T::Lookup::unlookup(user.clone());
        let balance_init_amount = <T as pallet_balances::Config>::Balance::saturated_from(100000000 as u128);
        Balances::<T>::force_set_balance(RawOrigin::Root.into(), user_lookup, balance_init_amount).unwrap();
        _create_twin::<T>(user.clone());
        _create_node_contract::<T>(user.clone());
        let contract_id = 1;

        let now = SmartContractModule::<T>::get_current_timestamp_in_secs();
        let elapsed_seconds = 5; // need to be < 6 secs to bill at same block!
        let then: u64 = now + elapsed_seconds;
        pallet_timestamp::Pallet::<T>::set_timestamp((then * 1000).try_into().unwrap());

        _push_contract_used_resources_report::<T>(farmer.clone());
        _push_contract_nru_consumption_report::<T>(farmer.clone(), then, elapsed_seconds);

        let contract = SmartContractModule::<T>::contracts(contract_id).unwrap();
        // Get contract cost before billing to take into account nu
        let (cost, _) = contract.calculate_contract_cost_tft(balance_init_amount, elapsed_seconds).unwrap();
    }: _(RawOrigin::Signed(farmer), contract_id)
    verify {
        let lock = SmartContractModule::<T>::contract_number_of_cylces_billed(contract_id);
        assert_eq!(lock.amount_locked, cost);
        let contract_bill = types::ContractBill {
            contract_id,
            timestamp: SmartContractModule::<T>::get_current_timestamp_in_secs(),
            discount_level: types::DiscountLevel::Gold,
            amount_billed: cost.saturated_into::<u128>(),
        };
        assert_last_event::<T>(Event::ContractBilled(contract_bill).into());
    }

    // service_contract_create()
    service_contract_create {
        let service: T::AccountId = account("Alice", 0, 0);
        _create_twin::<T>(service.clone());
        let consumer: T::AccountId = account("Bob", 0, 1);
        _create_twin::<T>(consumer.clone());
    }: _(RawOrigin::Signed(service.clone()), service.clone(), consumer)
    verify {
        let contract_id = 1;
        assert!(SmartContractModule::<T>::service_contracts(contract_id).is_some());
        let contract = SmartContractModule::<T>::service_contracts(contract_id).unwrap();
        assert_last_event::<T>(Event::ServiceContractCreated(contract).into());
    }

    // service_contract_set_metadata()
    service_contract_set_metadata {
        let service: T::AccountId = account("Alice", 0, 0);
        let consumer: T::AccountId = account("Bob", 0, 1);
        _create_service_contract::<T>(service.clone(), consumer.clone());
        let contract_id = 1;
    }: _(RawOrigin::Signed(service.clone()), contract_id, b"some_metadata".to_vec())
    verify {
        assert!(SmartContractModule::<T>::service_contracts(contract_id).is_some());
        let contract = SmartContractModule::<T>::service_contracts(contract_id).unwrap();
        assert_last_event::<T>(Event::ServiceContractMetadataSet(contract).into());
    }

    // service_contract_set_fees()
    service_contract_set_fees {
        let service: T::AccountId = account("Alice", 0, 0);
        let consumer: T::AccountId = account("Bob", 0, 1);
        _create_service_contract::<T>(service.clone(), consumer.clone());
        let contract_id = 1;
        let base_fee = 1000;
        let variable_fee = 1000;
    }: _(RawOrigin::Signed(service.clone()), contract_id, base_fee, variable_fee)
    verify {
        assert!(SmartContractModule::<T>::service_contracts(contract_id).is_some());
        let contract = SmartContractModule::<T>::service_contracts(contract_id).unwrap();
        assert_last_event::<T>(Event::ServiceContractFeesSet(contract).into());
    }

    // service_contract_approve()
    service_contract_approve {
        let service: T::AccountId = account("Alice", 0, 0);
        let consumer: T::AccountId = account("Bob", 0, 1);
        _prepare_service_contract::<T>(service.clone(), consumer);
        let contract_id = 1;
    }: _(RawOrigin::Signed(service), contract_id)
    verify {
        assert!(SmartContractModule::<T>::service_contracts(contract_id).is_some());
        let contract = SmartContractModule::<T>::service_contracts(contract_id).unwrap();
        assert_last_event::<T>(Event::ServiceContractApproved(contract).into());
    }

    // service_contract_reject()
    service_contract_reject {
        let service: T::AccountId = account("Alice", 0, 0);
        let consumer: T::AccountId = account("Bob", 0, 1);
        _prepare_service_contract::<T>(service.clone(), consumer);
        let contract_id = 1;
    }: _(RawOrigin::Signed(service),contract_id)
    verify {
        assert!(SmartContractModule::<T>::service_contracts(contract_id).is_none());
        assert_last_event::<T>(Event::ServiceContractCanceled {
            service_contract_id: contract_id,
            cause: types::Cause::CanceledByUser,
        }.into());
    }

    // service_contract_cancel()
    service_contract_cancel {
        let service: T::AccountId = account("Alice", 0, 0);
        let consumer: T::AccountId = account("Bob", 0, 1);
        _prepare_and_approve_service_contract::<T>(service.clone(), consumer);
        let contract_id = 1;
    }: _(RawOrigin::Signed(service), contract_id)
    verify {
        assert!(SmartContractModule::<T>::service_contracts(contract_id).is_none());
        assert_last_event::<T>(Event::ServiceContractCanceled {
            service_contract_id: contract_id,
            cause: types::Cause::CanceledByUser,
        }.into());
    }

    // service_contract_bill()
    service_contract_bill {
        let service: T::AccountId = account("Alice", 0, 0);
        let consumer: T::AccountId = account("Bob", 0, 1);
        let consumer_lookup = T::Lookup::unlookup(consumer.clone());
        let balance_init_amount = <T as pallet_balances::Config>::Balance::saturated_from(100000000 as u128);
        Balances::<T>::force_set_balance(RawOrigin::Root.into(), consumer_lookup, balance_init_amount).unwrap();
        _prepare_and_approve_service_contract::<T>(service.clone(), consumer);
        let contract_id = 1;
        let variable_amount = 0;
        let metadata = b"bill_metadata".to_vec();

    }: _(RawOrigin::Signed(service), contract_id, variable_amount, metadata.clone())
    verify {
        assert!(SmartContractModule::<T>::service_contracts(contract_id).is_some());
        let contract = SmartContractModule::<T>::service_contracts(contract_id).unwrap();
        let bill = types::ServiceContractBill {
            variable_amount,
            window: 0,
            metadata: BoundedVec::try_from(metadata).unwrap(),
        };
        let amount = contract
            .calculate_bill_cost_tft::<T>(bill.clone())
            .unwrap();
        assert_last_event::<T>(Event::ServiceContractBilled {
            service_contract: contract,
            bill,
            amount,
        }.into());
    }

    // change_billing_frequency()
    change_billing_frequency {
        let new_frequency = 900;
    }: _(RawOrigin::Root, new_frequency)
    verify {
        assert_eq!(SmartContractModule::<T>::billing_frequency(), new_frequency);
        assert_last_event::<T>(Event::BillingFrequencyChanged(new_frequency).into());
    }

    // attach_solution_provider_id()
    attach_solution_provider_id {
        let farmer: T::AccountId = whitelisted_caller();
        _prepare_farm_with_node::<T>(farmer.clone());

        let user: T::AccountId = account("Ferdie", 0, 2);
        _create_twin::<T>(user.clone());
        _create_node_contract::<T>(user.clone());
        let contract_id = 1;

        let provider: T::AccountId = account("Alice", 0, 0);
        _create_twin::<T>(provider.clone());
        _create_and_approve_solution_provider::<T>(provider);
        let solution_provider_id = 1;

    }: _(RawOrigin::Signed(user), contract_id, solution_provider_id)
    verify {
        assert!(SmartContractModule::<T>::contracts(contract_id).is_some());
        let contract = SmartContractModule::<T>::contracts(contract_id).unwrap();
        assert_eq!(contract.solution_provider_id, Some(solution_provider_id));
    }

    // set_dedicated_node_extra_fee
    set_dedicated_node_extra_fee {
        let farmer: T::AccountId = whitelisted_caller();
        _prepare_farm_with_node::<T>(farmer.clone());
        let node_id = 1;
        let extra_fee = 10000;

    }: _(RawOrigin::Signed(farmer), node_id, extra_fee)
    verify {
        assert_eq!(
            SmartContractModule::<T>::dedicated_nodes_extra_fee(node_id),
            extra_fee
        );
    }

    // cancel_contract_collective()
    cancel_contract_collective {
        let farmer: T::AccountId = account("Alice", 0, 0);
        _prepare_farm_with_node::<T>(farmer.clone());

        let user: T::AccountId = whitelisted_caller();
        _create_twin::<T>(user.clone());
        _create_node_contract::<T>(user.clone());
        let contract_id = 1;

    }: _(RawOrigin::Root, contract_id)
    verify {
        assert!(SmartContractModule::<T>::contracts(contract_id).is_none());
        let node_id = 1;
        let twin_id = 2;
        assert_last_event::<T>(Event::NodeContractCanceled {
            contract_id,
            node_id,
            twin_id,
         }.into());
    }

    // Calling the `impl_benchmark_test_suite` macro inside the `benchmarks`
    // block will generate one #[test] function per benchmark
    impl_benchmark_test_suite!(SmartContractModule, crate::mock::new_test_ext(), crate::mock::TestRuntime)
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    let events = System::<T>::events();
    let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}

pub fn run_to_block<T: Config>(n: BlockNumberFor<T>) {
    while System::<T>::block_number() < n {
        crate::Pallet::<T>::on_finalize(System::<T>::block_number());
        System::<T>::on_finalize(System::<T>::block_number());
        System::<T>::set_block_number(System::<T>::block_number() + One::one());
        System::<T>::on_initialize(System::<T>::block_number());
        crate::Pallet::<T>::on_initialize(System::<T>::block_number());
    }
}

pub fn _prepare_farm_with_node<T: Config>(source: T::AccountId) {
    _create_pricing_policy::<T>();
    _create_farming_policy::<T>();
    _create_twin::<T>(source.clone());
    _create_farm::<T>(source.clone());
    _create_node::<T>(source.clone());
}

fn _create_pricing_policy<T: Config>() {
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

    assert_ok!(TfgridModule::<T>::create_pricing_policy(
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

fn _create_farming_policy<T: Config>() {
    assert_ok!(TfgridModule::<T>::create_farming_policy(
        RawOrigin::Root.into(),
        b"fp".to_vec(),
        12,
        15,
        10,
        8,
        9999,
        BlockNumberFor::<T>::max_value(),
        true,
        true,
        NodeCertification::Diy,
        FarmCertification::NotCertified,
    ));
}

fn _create_twin<T: Config>(source: T::AccountId) {
    assert_ok!(TfgridModule::<T>::user_accept_tc(
        RawOrigin::Signed(source.clone()).into(),
        get_document_link_input(b"some_link"),
        get_document_hash_input(b"some_hash"),
    ));

    assert_ok!(TfgridModule::<T>::create_twin(
        RawOrigin::Signed(source).into(),
        get_relay_input(b"somerelay.io"),
        get_public_key_input(b"0x6c8fd181adc178cea218e168e8549f0b0ff30627c879db9eac4318927e87c901"),
    ));
}

fn _create_farm<T: Config>(source: T::AccountId) {
    let mut pub_ips = Vec::new();
    pub_ips.push(IP4 {
        ip: get_public_ip_ip_input(b"185.206.122.33/24"),
        gw: get_public_ip_gw_input(b"185.206.122.1"),
    });
    pub_ips.push(IP4 {
        ip: get_public_ip_ip_input(b"185.206.122.34/24"),
        gw: get_public_ip_gw_input(b"185.206.122.1"),
    });

    assert_ok!(TfgridModule::<T>::create_farm(
        RawOrigin::Signed(source).into(),
        b"testfarm".to_vec().try_into().unwrap(),
        pub_ips.try_into().unwrap(),
    ));
}

fn _create_node<T: Config>(source: T::AccountId) {
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

    assert_ok!(TfgridModule::<T>::create_node(
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

fn _create_node_contract<T: Config>(source: T::AccountId) {
    assert_ok!(SmartContractModule::<T>::create_node_contract(
        RawOrigin::Signed(source).into(),
        1,
        get_deployment_hash_input(b"858f8fb2184b15ecb8c0be8b95398c81"),
        get_deployment_data_input::<T>(b"some_data123"),
        0,
        None,
    ));
}

fn _push_contract_used_resources_report<T: Config>(source: T::AccountId) {
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

fn _push_contract_nru_consumption_report<T: Config>(
    source: T::AccountId,
    stamp_in_seconds: u64,
    elapsed_seconds: u64,
) {
    let nru_consumption = types::NruConsumption {
        contract_id: 1,
        timestamp: stamp_in_seconds,
        window: elapsed_seconds,
        nru: 10 * GIGABYTE,
    };

    let mut reports = Vec::new();
    reports.push(nru_consumption.clone());

    assert_ok!(SmartContractModule::<T>::add_nru_reports(
        RawOrigin::Signed(source).into(),
        reports,
    ));
}

fn _create_name_contract<T: Config>(source: T::AccountId) {
    assert_ok!(SmartContractModule::<T>::create_name_contract(
        RawOrigin::Signed(source).into(),
        b"foobar".to_vec()
    ));
}

fn _create_rent_contract<T: Config>(source: T::AccountId) {
    assert_ok!(SmartContractModule::<T>::create_rent_contract(
        RawOrigin::Signed(source).into(),
        1,
        None
    ));
}

fn _create_solution_provider<T: Config>(source: T::AccountId) {
    let provider1 = super::types::Provider {
        take: 10,
        who: account("Alice", 0, 0),
    };
    let provider2 = super::types::Provider {
        take: 10,
        who: account("Bob", 0, 1),
    };
    let providers = vec![provider1, provider2];

    assert_ok!(SmartContractModule::<T>::create_solution_provider(
        RawOrigin::Signed(source).into(),
        b"some_description".to_vec(),
        b"some_link".to_vec(),
        providers
    ));
}

fn _create_and_approve_solution_provider<T: Config>(source: T::AccountId) {
    _create_solution_provider::<T>(source);
    let solution_provider_id = 1;

    assert_ok!(SmartContractModule::<T>::approve_solution_provider(
        RawOrigin::Root.into(),
        solution_provider_id,
        true
    ));
}

fn _create_service_contract<T: Config>(service: T::AccountId, consumer: T::AccountId) {
    _create_twin::<T>(service.clone());
    _create_twin::<T>(consumer.clone());

    assert_ok!(SmartContractModule::<T>::service_contract_create(
        RawOrigin::Signed(service.clone()).into(),
        service,
        consumer,
    ));
}

fn _prepare_service_contract<T: Config>(service: T::AccountId, consumer: T::AccountId) {
    _create_service_contract::<T>(service.clone(), consumer);
    let contract_id = 1;
    let base_fee = 1000;
    let variable_fee = 1000;

    assert_ok!(SmartContractModule::<T>::service_contract_set_metadata(
        RawOrigin::Signed(service.clone()).into(),
        contract_id,
        b"some_metadata".to_vec(),
    ));

    assert_ok!(SmartContractModule::<T>::service_contract_set_fees(
        RawOrigin::Signed(service).into(),
        contract_id,
        base_fee,
        variable_fee,
    ));
}

fn _prepare_and_approve_service_contract<T: Config>(service: T::AccountId, consumer: T::AccountId) {
    _prepare_service_contract::<T>(service.clone(), consumer.clone());
    let contract_id = 1;

    // Service approves
    assert_ok!(SmartContractModule::<T>::service_contract_approve(
        RawOrigin::Signed(service).into(),
        contract_id,
    ));
    // Consumer approves
    assert_ok!(SmartContractModule::<T>::service_contract_approve(
        RawOrigin::Signed(consumer).into(),
        contract_id,
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

pub(crate) fn get_public_ip_ip_input(public_ip_ip_input: &[u8]) -> Ip4Input {
    BoundedVec::try_from(public_ip_ip_input.to_vec()).expect("Invalid public ip (ip) input.")
}

pub(crate) fn get_public_ip_gw_input(public_ip_gw_input: &[u8]) -> Gw4Input {
    BoundedVec::try_from(public_ip_gw_input.to_vec()).expect("Invalid public ip (gw) input.")
}

pub(crate) fn get_deployment_hash_input(deployment_hash_input: &[u8]) -> types::HexHash {
    deployment_hash_input
        .to_vec()
        .try_into()
        .expect("Invalid deployment hash input.")
}

pub(crate) fn get_deployment_data_input<T: pallet::Config>(
    deployment_data_input: &[u8],
) -> DeploymentDataInput<T> {
    BoundedVec::try_from(deployment_data_input.to_vec()).expect("Invalid deployment data input.")
}
