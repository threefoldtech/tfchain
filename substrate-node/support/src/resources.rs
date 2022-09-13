pub use super::types::Resources;

pub const ONE_THOUSAND: u128 = 1_000;
pub const GIB: u128 = 1024 * 1024 * 1024;

pub fn get_cu(resources: Resources) -> u64 {
    let cu = calc_cu(resources);
    let calculated_cu = 2 * (cu as u128 / GIB / ONE_THOUSAND);
    calculated_cu as u64
}

pub fn calc_cu(resources: Resources) -> u64 {
    let cu_cru = resources.cru as u128 * 2 * GIB * ONE_THOUSAND;
    let cu_mru = ((resources.mru as u128).checked_sub(1).unwrap_or(0) * GIB) * ONE_THOUSAND / 4;
    let cu_sru = resources.sru as u128 * ONE_THOUSAND / 50;
    cu_cru.min(cu_mru.min(cu_sru)) as u64
}

pub fn get_su(resources: Resources) -> u64 {
    let su =
        resources.hru as u128 * ONE_THOUSAND / 1200 + resources.sru as u128 * ONE_THOUSAND / 250;
    let calculated_su = su / GIB;
    let result = calculated_su as u128 / ONE_THOUSAND;
    result as u64
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_calc_cu_falsy_values() {
        let resources = Resources {
            hru: 0,
            cru: 0,
            mru: 0,
            sru: 0,
        };

        let cu = get_cu(resources);
        assert_eq!(cu, 0);
    }

    #[test]
    fn test_calc_cu() {
        let resources = Resources {
            hru: 4 * GIB as u64 * 1024,
            cru: 64,
            mru: 64 * GIB as u64 * 1024,
            sru: 12 * GIB as u64 * 1024,
        };

        let cu = get_cu(resources);
        assert_eq!(cu, 256);
    }

    #[test]
    fn test_calc_cu_2() {
        let resources = Resources {
            hru: 4 * GIB as u64 * 1024,
            cru: 4,
            mru: 8,
            sru: 12 * GIB as u64 * 1024,
        };

        let cu = get_cu(resources);
        assert_eq!(cu, 2);
    }

    #[test]
    fn test_calc_su() {
        let resources = Resources {
            hru: 4 * GIB as u64 * 1024,
            cru: 64,
            mru: 64,
            sru: 12 * GIB as u64 * 1024,
        };

        let su = get_su(resources);
        assert_eq!(su, 52);
    }

    #[test]
    fn test_calc_su_2() {
        let resources = Resources {
            hru: 0,
            cru: 64,
            mru: 64,
            sru: 12 * GIB as u64 * 1024,
        };

        let su = get_su(resources);
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

        let su = get_su(resources);
        assert_eq!(su, 0);
    }

    #[test]
    fn test_calc_su_4() {
        let resources = Resources {
            hru: 4 * GIB as u64 * 1024,
            cru: 64,
            mru: 64,
            sru: 0,
        };

        let su = get_su(resources);
        assert_eq!(su, 3);
    }
}
