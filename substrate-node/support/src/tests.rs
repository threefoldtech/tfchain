use crate::resources::{Resources, GIGABYTE};
use crate::types::{PublicIpError, IP4, IP6};
use frame_support::{assert_err, assert_ok, storage::bounded_vec::BoundedVec};

#[test]
fn test_calc_cu_falsy_values() {
    let resources = Resources {
        hru: 0,
        cru: 0,
        mru: 0,
        sru: 0,
    };

    let cu = resources.get_cu();
    assert_eq!(cu, 0);
}

#[test]
fn test_calc_cu() {
    let resources = Resources {
        hru: 4 * GIGABYTE as u64 * 1024,
        cru: 64,
        mru: 64 * GIGABYTE as u64 * 1024,
        sru: 12 * GIGABYTE as u64 * 1024,
    };

    let cu = resources.get_cu();
    assert_eq!(cu, 256);
}

#[test]
fn test_calc_cu_2() {
    let resources = Resources {
        hru: 4 * GIGABYTE as u64 * 1024,
        cru: 4,
        mru: 8,
        sru: 12 * GIGABYTE as u64 * 1024,
    };

    let cu = resources.get_cu();
    assert_eq!(cu, 2);
}

#[test]
fn test_calc_su() {
    let resources = Resources {
        hru: 4 * GIGABYTE as u64 * 1024,
        cru: 64,
        mru: 64,
        sru: 12 * GIGABYTE as u64 * 1024,
    };

    let su = resources.get_su();
    assert_eq!(su, 52);
}

#[test]
fn test_calc_su_2() {
    let resources = Resources {
        hru: 0,
        cru: 64,
        mru: 64,
        sru: 12 * GIGABYTE as u64 * 1024,
    };

    let su = resources.get_su();
    assert_eq!(su, 49);
}

#[test]
fn test_calc_su_3() {
    let resources = Resources {
        hru: 0,
        cru: 64,
        mru: 64,
        sru: 0,
    };

    let su = resources.get_su();
    assert_eq!(su, 0);
}

#[test]
fn test_calc_su_4() {
    let resources = Resources {
        hru: 4 * GIGABYTE as u64 * 1024,
        cru: 64,
        mru: 64,
        sru: 0,
    };

    let su = resources.get_su();
    assert_eq!(su, 3);
}

#[test]
fn test_resources_diff() {
    let resources = Resources {
        hru: 4 * GIGABYTE as u64 * 1024,
        cru: 64,
        mru: 64 * GIGABYTE as u64,
        sru: 0,
    };

    let new_resources = Resources {
        hru: 4 * GIGABYTE as u64 * 1024,
        cru: 64,
        mru: 64 * GIGABYTE as u64,
        sru: 0,
    };

    assert_eq!(Resources::has_changed(&resources, &new_resources, 1), false);

    let resources = Resources {
        hru: 4 * GIGABYTE as u64 * 1024,
        cru: 64,
        mru: 64 * GIGABYTE as u64,
        sru: 0,
    };

    let new_resources = Resources {
        hru: 4 * GIGABYTE as u64 * 1024,
        cru: 64,
        mru: 40 * GIGABYTE as u64,
        sru: 0,
    };

    assert_eq!(Resources::has_changed(&resources, &new_resources, 1), true);

    let resources = Resources {
        hru: 4 * GIGABYTE as u64 * 1024,
        cru: 64,
        mru: 64 * GIGABYTE as u64,
        sru: 1000 * GIGABYTE as u64,
    };

    let new_resources = Resources {
        hru: 4 * GIGABYTE as u64 * 1024,
        cru: 64,
        mru: 64 * GIGABYTE as u64,
        sru: 989 * GIGABYTE as u64,
    };

    assert_eq!(Resources::has_changed(&resources, &new_resources, 1), true);
}

#[test]
fn test_ip4_is_valid() {
    // Rerence
    let ip4 = IP4 {
        ip: BoundedVec::try_from(b"185.206.122.1/24".to_vec()).unwrap(),
        gw: BoundedVec::try_from(b"185.206.122.2".to_vec()).unwrap(),
    };
    assert_ok!(ip4.is_valid());
    // Same ip
    let ip4 = IP4 {
        ip: BoundedVec::try_from(b"185.206.122.1/24".to_vec()).unwrap(),
        gw: BoundedVec::try_from(b"185.206.122.1".to_vec()).unwrap(),
    };
    assert_err!(ip4.is_valid(), PublicIpError::InvalidPublicIp);
    // Different subset
    let ip4 = IP4 {
        ip: BoundedVec::try_from(b"185.206.122.1/24".to_vec()).unwrap(),
        gw: BoundedVec::try_from(b"185.206.121.2".to_vec()).unwrap(),
    };
    assert_err!(ip4.is_valid(), PublicIpError::InvalidPublicIp);
}

#[test]
fn test_ip6_is_valid() {
    // Rerence
    let ip6 = IP6 {
        ip: BoundedVec::try_from(b"2a10:b600:1::0cc4:7a30:65b5/64".to_vec()).unwrap(),
        gw: BoundedVec::try_from(b"2a10:b600:1::1".to_vec()).unwrap(),
    };
    assert_ok!(ip6.is_valid());
    // Same ip
    let ip6 = IP6 {
        ip: BoundedVec::try_from(b"2a10:b600:1::0cc4:7a30:65b5/64".to_vec()).unwrap(),
        gw: BoundedVec::try_from(b"2a10:b600:1::0cc4:7a30:65b5".to_vec()).unwrap(),
    };
    assert_err!(ip6.is_valid(), PublicIpError::InvalidPublicIp);
    // Different subset
    let ip6 = IP6 {
        ip: BoundedVec::try_from(b"2a10:b600:1::0cc4:7a30:65b5/64".to_vec()).unwrap(),
        gw: BoundedVec::try_from(b"2a10:b600:2::1".to_vec()).unwrap(),
    };
    assert_err!(ip6.is_valid(), PublicIpError::InvalidPublicIp);
}
