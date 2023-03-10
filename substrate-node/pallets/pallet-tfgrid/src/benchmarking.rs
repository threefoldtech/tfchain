#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as TfgridModule;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::{assert_ok, BoundedVec};
use frame_system::{EventRecord, Pallet as System, RawOrigin};
// use hex;
use pallet_timestamp::Pallet as Timestamp;
use scale_info::prelude::format;
use sp_core::{ed25519, Pair, Public};
// use sp_core::*;
// use sp_core::crypto::Public;
// use sp_core::ed25519;
use sp_core::sr25519;
// use sp_core::Pair;
// use sp_core::Public;
use sp_runtime::{
    traits::{Bounded, IdentifyAccount, Verify},
    AccountId32, MultiSignature,
};
use sp_std::{
    convert::{TryFrom, TryInto},
    fmt::Debug,
    vec,
};
use tfchain_support::types::{FarmCertification, NodeCertification, PublicConfig, IP4, IP6};

type AccountPublic = <MultiSignature as Verify>::Signer;
type AccountId = <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;

const GIGABYTE: u64 = 1024 * 1024 * 1024;
const TIMESTAMP_INIT_SECS: u64 = 1628082000;

benchmarks! {
    where_clause {
        where
        <T as pallet_timestamp::Config>::Moment: TryFrom<u64>,
        <<T as pallet_timestamp::Config>::Moment as TryFrom<u64>>::Error: Debug,
        <T as frame_system::Config>::AccountId: From<AccountId32>,
    }

    // set_storage_version()
    set_storage_version {
        let version = types::StorageVersion::default();
    }: _ (RawOrigin::Root, version.clone())
    verify {
        assert_eq!(TfgridModule::<T>::pallet_version(), version);
    }

    // create_farm()
    create_farm {
        let caller: T::AccountId = whitelisted_caller();
        _create_twin::<T>(caller.clone());
        let name = b"farm_name".to_vec();
        let pub_ips = Vec::new();
    }: _ (
        RawOrigin::Signed(caller),
        name.try_into().unwrap(),
        pub_ips.try_into().unwrap()
    )
    verify {
        let farm_id = 1;
        assert!(TfgridModule::<T>::farms(farm_id).is_some());
        let farm = TfgridModule::<T>::farms(farm_id).unwrap();
        assert_last_event::<T>(Event::FarmStored(farm).into());
    }

    // update_farm()
    update_farm {
        let caller: T::AccountId = whitelisted_caller();
        _prepare_farm::<T>(caller.clone());
        let farm_id = 1;
        let name = b"new_farm_name".to_vec();
    }: _ (RawOrigin::Signed(caller), farm_id, name.try_into().unwrap())
    verify {
        assert!(TfgridModule::<T>::farms(farm_id).is_some());
        let farm = TfgridModule::<T>::farms(farm_id).unwrap();
        assert_last_event::<T>(Event::FarmUpdated(farm).into());
    }

    // add_stellar_payout_v2address()
    add_stellar_payout_v2address {
        let caller: T::AccountId = whitelisted_caller();
        _prepare_farm::<T>(caller.clone());
        let farm_id = 1;
        let stellar_address = b"some_address".to_vec();
    }: _ (RawOrigin::Signed(caller), farm_id, stellar_address.clone())
    verify {
        assert_eq!(TfgridModule::<T>::farm_payout_address_by_farm_id(farm_id), stellar_address);
        assert_last_event::<T>(Event::FarmPayoutV2AddressRegistered(
            farm_id,
            stellar_address,
        ).into());
    }

    // set_farm_certification()
    set_farm_certification {
        let caller: T::AccountId = whitelisted_caller();
        _prepare_farm::<T>(caller);
        let farm_id = 1;
        let certification = FarmCertification::Gold;
    }: _ (RawOrigin::Root, farm_id, certification)
    verify {
        assert_last_event::<T>(Event::FarmCertificationSet(farm_id, certification).into());
    }

    // add_farm_ip()
    add_farm_ip {
        let caller: T::AccountId = whitelisted_caller();
        _prepare_farm::<T>(caller.clone());
        let farm_id = 1;
        let ip = get_public_ip_ip_input(b"185.206.122.125/16");
        let gw = get_public_ip_gw_input(b"185.206.122.1");
    }: _ (RawOrigin::Signed(caller), farm_id, ip, gw)
    verify {
        assert!(TfgridModule::<T>::farms(farm_id).is_some());
        let farm = TfgridModule::<T>::farms(farm_id).unwrap();
        assert_last_event::<T>(Event::FarmUpdated(farm).into());
    }

    // remove_farm_ip()
    remove_farm_ip {
        let caller: T::AccountId = whitelisted_caller();
        _prepare_farm::<T>(caller.clone());
        let farm_id = 1;
        let ip = get_public_ip_ip_input(b"185.206.122.33/24");
    }: _ (RawOrigin::Signed(caller), farm_id, ip)
    verify {
        assert!(TfgridModule::<T>::farms(farm_id).is_some());
        let farm = TfgridModule::<T>::farms(farm_id).unwrap();
        assert_last_event::<T>(Event::FarmUpdated(farm).into());
    }

    // create_node()
    create_node {
        let caller: T::AccountId = whitelisted_caller();
        _prepare_farm::<T>(caller.clone());
        let farm_id = 1;

        let resources = ResourcesInput {
            hru: 1024 * GIGABYTE,
            sru: 512 * GIGABYTE,
            cru: 8,
            mru: 16 * GIGABYTE,
        };

        let location = LocationInput {
            city: get_city_name_input(b"Ghent"),
            country: get_country_name_input(b"Belgium"),
            latitude: get_latitude_input(b"12.233213231"),
            longitude: get_longitude_input(b"32.323112123"),
        };
        let interfaces = Vec::new();
        let secure_boot = false;
        let virtualized= false;
        let serial_number = None;
    }: _ (
        RawOrigin::Signed(caller),
        farm_id,
        resources,
        location,
        interfaces.try_into().unwrap(),
        secure_boot,
        virtualized,
        serial_number
    )
    verify {
        let node_id = 1;
        assert!(TfgridModule::<T>::nodes(node_id).is_some());
        let node = TfgridModule::<T>::nodes(node_id).unwrap();
        assert_last_event::<T>(Event::NodeStored(node).into());
    }

    // update_node()
    update_node {
        let caller: T::AccountId = whitelisted_caller();
        _prepare_farm_with_node::<T>(caller.clone());
        let node_id = 1;
        let node = TfgridModule::<T>::nodes(node_id).unwrap();

        let resources = ResourcesInput {
            hru: node.resources.hru,
            sru: node.resources.sru * 2,
            cru: node.resources.cru,
            mru: node.resources.mru * 2,
        };
        let location = LocationInput {
            city: get_city_name_input(b"Rio de Janeiro"),
            country: get_country_name_input(b"Brazil"),
            latitude: get_latitude_input(b"43.1868"),
            longitude: get_longitude_input(b"22.9694"),
        };
        let interfaces = Vec::new();
        let secure_boot = true;
        let virtualized = true;
        let serial_number = b"some_serial".to_vec();
    }: _ (
        RawOrigin::Signed(caller),
        node_id,
        node.farm_id,
        resources,
        location,
        interfaces.try_into().unwrap(),
        secure_boot,
        virtualized,
        Some(serial_number.try_into().unwrap())
    )
    verify {
        let node_id = 1;
        assert!(TfgridModule::<T>::nodes(node_id).is_some());
        let node = TfgridModule::<T>::nodes(node_id).unwrap();
        assert_last_event::<T>(Event::NodeUpdated(node).into());
    }

    // set_node_certification()
    set_node_certification {
        let caller: T::AccountId = whitelisted_caller();
        _prepare_farm_with_node::<T>(caller);
        let node_id = 1;
        let node_certification = NodeCertification::Certified;
    }: _ (
        RawOrigin::Root, node_id, node_certification)
    verify {
        assert_last_event::<T>(Event::NodeCertificationSet(
            node_id,
            node_certification
        ).into());
    }

    // report_uptime()
    report_uptime {
        let caller: T::AccountId = whitelisted_caller();
        _prepare_farm_with_node::<T>(caller.clone());
        let node_id = 1;

        let now: u64 = TIMESTAMP_INIT_SECS;
        Timestamp::<T>::set_timestamp((now * 1000).try_into().unwrap());

        let uptime = 500;
    }: _ (RawOrigin::Signed(caller), uptime)
    verify {
        assert_last_event::<T>(Event::NodeUptimeReported(node_id, now, uptime).into());
    }

    // add_node_public_config()
    add_node_public_config {
        let caller: T::AccountId = whitelisted_caller();
        _prepare_farm_with_node::<T>(caller.clone());
        let farm_id = 1;
        let node_id = 1;

        let public_config = PublicConfig {
            ip4: IP4 {
                ip: get_pub_config_ip4_input(b"185.206.122.33/24"),
                gw: get_pub_config_gw4_input(b"185.206.122.1"),
            },
            ip6: Some(IP6 {
                ip: get_pub_config_ip6_input(b"2a10:b600:1::0cc4:7a30:65b5/64"),
                gw: get_pub_config_gw6_input(b"2a10:b600:1::1"),
            }),
            domain: Some(get_pub_config_domain_input(b"some-domain")),
        };
    }: _ (RawOrigin::Signed(caller), farm_id, node_id, Some(public_config.clone()))
    verify {
        assert!(TfgridModule::<T>::nodes(node_id).is_some());
        let node = TfgridModule::<T>::nodes(node_id).unwrap();
        assert_eq!(node.public_config, Some(public_config));
        assert_last_event::<T>(Event::NodePublicConfigStored(
            node_id,
            node.public_config
        ).into());
    }

    // delete_node()
    delete_node {
        let caller: T::AccountId = whitelisted_caller();
        _prepare_farm_with_node::<T>(caller.clone());
        let node_id = 1;
    }: _ (RawOrigin::Signed(caller), node_id)
    verify {
        assert!(TfgridModule::<T>::nodes(node_id).is_none());
        assert_last_event::<T>(Event::NodeDeleted(node_id).into());
    }

    // create_entity()
    create_entity {
        let caller: T::AccountId = whitelisted_caller();
        let target: T::AccountId = _test_ed25519().try_into().unwrap();
        let name = b"entity_name".to_vec();
        let country = get_country_name_input(b"Belgium");
        let city = get_city_name_input(b"Ghent");
        let signature = _sign_create_entity(name.clone(), country.to_vec(), city.to_vec());
    }: _ (RawOrigin::Signed(caller), target.clone(), name.clone(), country, city, signature)
    verify {
        let entity_id = 1;
        assert_eq!(TfgridModule::<T>::entities_by_pubkey_id(target), Some(entity_id));
        assert_eq!(TfgridModule::<T>::entities_by_name_id(name), entity_id);
        assert!(TfgridModule::<T>::entities(entity_id).is_some());
        let entity = TfgridModule::<T>::entities(entity_id).unwrap();
        assert_last_event::<T>(Event::EntityStored(entity).into());
    }

    // update_entity()
    update_entity {
        let caller: T::AccountId = whitelisted_caller();
        _create_entity::<T>(caller);

        let entity_id = 1;
        let entity = TfgridModule::<T>::entities(entity_id).unwrap();

        let name = b"new_entity_name".to_vec();
        let country = get_country_name_input(b"Brazil");
        let city = get_city_name_input(b"Rio de Janeiro");
    }: _ (RawOrigin::Signed(entity.account_id), name.clone(), country, city)
    verify {
        assert_eq!(TfgridModule::<T>::entities_by_name_id(name), entity_id);
        assert!(TfgridModule::<T>::entities(entity_id).is_some());
        let entity = TfgridModule::<T>::entities(entity_id).unwrap();
        assert_last_event::<T>(Event::EntityUpdated(entity).into());
    }

    // delete_entity()
    delete_entity {
        let caller: T::AccountId = whitelisted_caller();
        _create_entity::<T>(caller);
        let entity_id = 1;
        let entity = TfgridModule::<T>::entities(entity_id).unwrap();
    }: _ (RawOrigin::Signed(entity.account_id.clone()))
    verify {
        assert!(TfgridModule::<T>::entities_by_pubkey_id(entity.account_id).is_none());
        assert_eq!(TfgridModule::<T>::entities_by_name_id(entity.name), 0 as u32);
        assert!(TfgridModule::<T>::entities(entity_id).is_none());
        assert_last_event::<T>(Event::EntityDeleted(entity_id).into());
    }

    // create_twin()
    create_twin {
        let caller: T::AccountId = whitelisted_caller();
        TfgridModule::<T>::user_accept_tc(
            RawOrigin::Signed(caller.clone()).into(),
            get_document_link_input(b"some_link"),
            get_document_hash_input(b"some_hash"),
        ).unwrap();
    }: _ (
        RawOrigin::Signed(caller),
        get_relay_input(b"somerelay.io"),
        get_public_key_input(b"0x6c8fd181adc178cea218e168e8549f0b0ff30627c879db9eac4318927e87c901")
    )
    verify {
        let twin_id = 1;
        assert!(TfgridModule::<T>::twins(twin_id).is_some());
        let twin = TfgridModule::<T>::twins(twin_id).unwrap();
        assert_last_event::<T>(Event::TwinStored(twin).into());
    }

    // update_twin()
    // add_twin_entity()
    // delete_twin_entity()
    // create_pricing_policy()
    // update_pricing_policy()
    // create_farming_policy()

    // user_accept_tc()
    user_accept_tc {
        let caller: T::AccountId = whitelisted_caller();
    }: _ (
        RawOrigin::Signed(caller.clone()),
        get_document_link_input(b"some_link"),
        get_document_hash_input(b"some_hash")
    )
    verify {
        assert!(TfgridModule::<T>::users_terms_and_condition(caller).is_some());
    }

    // delete_node_farm()
    // set_farm_dedicated()
    // force_reset_farm_ip()
    // set_connection_price()
    // add_node_certifier()
    // remove_node_certifier()
    // update_farming_policy()
    // attach_policy_to_farm()
    // set_zos_version()

    // Calling the `impl_benchmark_test_suite` macro inside the `benchmarks`
    // block will generate one #[test] function per benchmark
    impl_benchmark_test_suite! (TfgridModule, crate::mock::new_test_ext(), crate::mock::TestRuntime)
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    let events = System::<T>::events();
    let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}

pub fn _prepare_farm_with_node<T: Config>(source: T::AccountId) {
    _prepare_farm::<T>(source.clone());
    _create_node::<T>(source);
}

pub fn _prepare_farm<T: Config>(source: T::AccountId) {
    _create_farming_policy::<T>();
    _create_twin::<T>(source.clone());
    _create_farm::<T>(source);
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
        <T as frame_system::Config>::BlockNumber::max_value(),
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

    let location = LocationInput {
        city: get_city_name_input(b"Ghent"),
        country: get_country_name_input(b"Belgium"),
        latitude: get_latitude_input(b"12.233213231"),
        longitude: get_longitude_input(b"32.323112123"),
    };

    assert_ok!(TfgridModule::<T>::create_node(
        RawOrigin::Signed(source).into(),
        1,
        resources,
        location,
        Vec::new().try_into().unwrap(),
        false,
        false,
        None,
    ));
}

fn _create_entity<T: Config>(source: T::AccountId)
where
    <T as frame_system::Config>::AccountId: From<AccountId32>,
{
    let target: T::AccountId = _test_ed25519().try_into().unwrap();
    let name = b"entity_name".to_vec();
    let country = get_country_name_input(b"Belgium");
    let city = get_city_name_input(b"Ghent");
    let signature = _sign_create_entity(name.clone(), country.to_vec(), city.to_vec());

    assert_ok!(TfgridModule::<T>::create_entity(
        RawOrigin::Signed(source).into(),
        target,
        name,
        country,
        city,
        signature,
    ));
}

pub fn _test_ed25519() -> AccountId {
    _get_account_id_from_seed_string::<ed25519::Public>(
        "industry dismiss casual gym gap music pave gasp sick owner dumb cost",
    )
}

fn _get_account_id_from_seed_string<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(_get_from_seed_string::<TPublic>(seed)).into_account()
}

fn _get_from_seed_string<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

pub fn _sign_create_entity(name: Vec<u8>, country: Vec<u8>, city: Vec<u8>) -> Vec<u8> {
    let seed =
        hex::decode("59336423ee7af732b2d4a76e440651e33e5ba51540e5633535b9030492c2a6f6").unwrap();
    let pair = ed25519::Pair::from_seed_slice(&seed).unwrap();

    let mut message = vec![];
    message.extend_from_slice(&name);
    message.extend_from_slice(&country);
    message.extend_from_slice(&city);

    let signature = pair.sign(&message);

    // hex encode signature
    hex::encode(signature.0.to_vec()).into()
}

pub fn _sign_create_entity_sr(name: Vec<u8>, country: Vec<u8>, city: Vec<u8>) -> Vec<u8> {
    let seed =
        hex::decode("59336423ee7af732b2d4a76e440651e33e5ba51540e5633535b9030492c2a6f6").unwrap();
    let pair = sr25519::Pair::from_seed_slice(&seed).unwrap();

    let mut message = vec![];
    message.extend_from_slice(&name);
    message.extend_from_slice(&country);
    message.extend_from_slice(&city);

    let signature = pair.sign(&message);

    // hex encode signature
    hex::encode(signature.0.to_vec()).into()
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

pub(crate) fn get_pub_config_ip4_input(ip4_input: &[u8]) -> Ip4Input {
    BoundedVec::try_from(ip4_input.to_vec()).expect("Invalid ip4 input.")
}

pub(crate) fn get_pub_config_gw4_input(gw4_input: &[u8]) -> Gw4Input {
    BoundedVec::try_from(gw4_input.to_vec()).expect("Invalid gw4 input.")
}

pub(crate) fn get_pub_config_ip6_input(ip6_input: &[u8]) -> Ip6Input {
    BoundedVec::try_from(ip6_input.to_vec()).expect("Invalid ip6 input.")
}

pub(crate) fn get_pub_config_gw6_input(gw6_input: &[u8]) -> Gw6Input {
    BoundedVec::try_from(gw6_input.to_vec()).expect("Invalid gw6 input.")
}

pub(crate) fn get_pub_config_domain_input(domain_input: &[u8]) -> DomainInput {
    BoundedVec::try_from(domain_input.to_vec()).expect("Invalid domain input.")
}
