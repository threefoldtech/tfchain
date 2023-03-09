#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as DaoModule;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::{assert_ok, BoundedVec};
use frame_system::{Call as SystemCall, EventRecord, Pallet as System, RawOrigin};
use pallet_membership::Pallet as CouncilMembership;
use pallet_tfgrid::{
    types::LocationInput, CityNameInput, CountryNameInput, DocumentHashInput, DocumentLinkInput,
    Gw4Input, Ip4Input, LatitudeInput, LongitudeInput, Pallet as Tfgrid, PkInput, RelayInput,
    ResourcesInput,
};
use sp_runtime::traits::StaticLookup;
use sp_std::convert::{TryFrom, TryInto};
use tfchain_support::types::IP4;

const GIGABYTE: u64 = 1024 * 1024 * 1024;

benchmarks! {
    // propose()
    propose {
        let caller: T::AccountId = whitelisted_caller();
        assert_ok!(_add_council_member::<T>(caller.clone()));
        let threshold = 1;
        let proposal: T::Proposal = SystemCall::<T>::remark { remark: b"remark".to_vec() }.into();
        let description = b"some_description".to_vec();
        let link = b"some_link".to_vec();
    }: _ (
        RawOrigin::Signed(caller.clone()),
        threshold,
        Box::new(proposal.clone()),
        description,
        link,
        None
    )
    verify {
        let proposal_count = DaoModule::<T>::proposal_count();
        assert_eq!(proposal_count, 1);
        let proposal_index = 0;
        assert_eq!(DaoModule::<T>::proposals_list_hashes().len(), 1);
        let proposal_hash = DaoModule::<T>::proposals_list_hashes().into_iter().next().unwrap();
        assert!(DaoModule::<T>::proposal_list(proposal_hash).is_some());
        assert!(DaoModule::<T>::proposal_of(proposal_hash).is_some());
        assert!(DaoModule::<T>::voting(&proposal_hash).is_some());
        assert_last_event::<T>(Event::Proposed { account: caller, proposal_index, proposal_hash, threshold }.into());
    }

    // vote()
    vote {
        let farmer: T::AccountId = account("Alice", 0, 0);
        _prepare_farm_with_node::<T>(farmer.clone());
        let farm_id = 1;

        let caller: T::AccountId = whitelisted_caller();
        let proposal_hash = _create_proposal::<T>(caller.clone());

        let approve = true;
    }: _ (RawOrigin::Signed(farmer.clone()), farm_id, proposal_hash, approve)
    verify {
        assert!(DaoModule::<T>::proposal_list(proposal_hash).is_some());
        assert!(DaoModule::<T>::proposal_of(proposal_hash).is_some());
        assert!(DaoModule::<T>::voting(&proposal_hash).is_some());
        let voting = DaoModule::<T>::voting(&proposal_hash).unwrap();
        assert_eq!(voting.ayes.len(), 1);
        assert_eq!(voting.nays.len(), 0);
        assert_last_event::<T>(Event::Voted {
            account: farmer,
            proposal_hash,
            voted: approve,
            yes: 1 as u32,
            no: 0 as u32
        }.into());
    }

    // veto()
    veto {
        let farmer: T::AccountId = account("Alice", 0, 0);
        _prepare_farm_with_node::<T>(farmer.clone());
        let farm_id = 1;

        let caller: T::AccountId = whitelisted_caller();
        let proposal_hash = _create_proposal::<T>(caller.clone());
    }: _ (RawOrigin::Signed(caller.clone()), proposal_hash)
    verify {
        assert!(DaoModule::<T>::proposal_list(proposal_hash).is_some());
        assert!(DaoModule::<T>::proposal_of(proposal_hash).is_some());
        assert!(DaoModule::<T>::voting(proposal_hash).is_some());
        let voting = DaoModule::<T>::voting(&proposal_hash).unwrap();
        assert_eq!(voting.vetos, vec![caller.clone()]);
        assert_last_event::<T>(Event::CouncilMemberVeto { proposal_hash, who: caller }.into());
    }

    // close()
    close {
        let farmer: T::AccountId = account("Alice", 0, 0);
        _prepare_farm_with_node::<T>(farmer.clone());
        let farm_id = 1;

        let caller: T::AccountId = whitelisted_caller();
        let proposal_hash = _create_proposal::<T>(caller.clone());
        let proposal_index = 0;

        let approve = false;
        DaoModule::<T>::vote(RawOrigin::Signed(farmer.clone()).into(), farm_id, proposal_hash, approve).unwrap();
    }: _ (RawOrigin::Signed(caller.clone()), proposal_hash, proposal_index)
    verify {
        assert!(DaoModule::<T>::proposal_list(proposal_hash).is_none());
        assert!(DaoModule::<T>::proposal_of(proposal_hash).is_none());
        assert!(DaoModule::<T>::voting(proposal_hash).is_none());
        assert_last_event::<T>(Event::Disapproved { proposal_hash }.into());
    }

    // Calling the `impl_benchmark_test_suite` macro inside the `benchmarks`
    // block will generate one #[test] function per benchmark
    impl_benchmark_test_suite! (DaoModule, crate::mock::new_test_ext(), crate::mock::TestRuntime)
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    let events = System::<T>::events();
    let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}

pub fn _prepare_farm_with_node<T: Config>(source: T::AccountId) {
    _create_twin::<T>(source.clone());
    _create_farm::<T>(source.clone());
    _create_node::<T>(source.clone());
}

fn _create_twin<T: Config>(source: T::AccountId) {
    assert_ok!(Tfgrid::<T>::user_accept_tc(
        RawOrigin::Signed(source.clone()).into(),
        get_document_link_input(b"some_link"),
        get_document_hash_input(b"some_hash"),
    ));

    assert_ok!(Tfgrid::<T>::create_twin(
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

    assert_ok!(pallet_tfgrid::Pallet::<T>::create_farm(
        RawOrigin::Signed(source).into(),
        b"testfarm".to_vec().try_into().unwrap(),
        pub_ips.clone().try_into().unwrap(),
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

pub fn _create_proposal<T: Config>(source: T::AccountId) -> T::Hash {
    assert_ok!(_add_council_member::<T>(source.clone()));

    let threshold = 1;
    let proposal: T::Proposal = SystemCall::<T>::remark {
        remark: b"remark".to_vec(),
    }
    .into();

    DaoModule::<T>::propose(
        RawOrigin::Signed(source).into(),
        threshold,
        Box::new(proposal.clone()),
        b"some_description".to_vec(),
        b"some_link".to_vec(),
        None,
    )
    .unwrap();

    T::Hashing::hash_of(&proposal)
}

fn _add_council_member<T: Config>(source: T::AccountId) -> Result<(), DispatchError> {
    let source_lookup = T::Lookup::unlookup(source.clone());
    CouncilMembership::<T, _>::add_member(RawOrigin::Root.into(), source_lookup)
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
