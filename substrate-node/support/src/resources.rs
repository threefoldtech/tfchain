use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

/// A resources capacity that countains HRU, SRU, CRU and MRU in integer values.
#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo, MaxEncodedLen,
)]
pub struct Resources {
    pub hru: u64,
    pub sru: u64,
    pub cru: u64,
    pub mru: u64,
}

pub const ONE_THOUSAND: u128 = 1_000;
pub const GIGABYTE: u128 = 1024 * 1024 * 1024;

impl Resources {
    pub fn add(mut self, other: &Resources) -> Resources {
        self.cru += other.cru;
        self.sru += other.sru;
        self.hru += other.hru;
        self.mru += other.mru;
        self
    }

    pub fn validate_hru(&self) -> bool {
        // No constraint, always valid
        true
    }

    pub fn validate_sru(&self) -> bool {
        // Convert SRU to bytes

        // minimum of 500 GB
        true
    }

    pub fn validate_cru(&self) -> bool {
        // Convert SRU to vCPU
        // minimum of 1 vCPU
        true
    }

    pub fn validate_mru(&self) -> bool {
        // Convert MRU to bytes
        // minimum of 2GB
        true
    }

    pub fn get_cu(&self) -> u64 {
        let cu = self.calc_cu();
        let calculated_cu = 2 * (cu as u128 / GIGABYTE / ONE_THOUSAND);
        calculated_cu as u64
    }

    fn calc_cu(&self) -> u64 {
        let cru_min = self.cru as u128 * 2 * GIGABYTE * ONE_THOUSAND;
        let mru_min =
            ((self.mru as u128).checked_sub(1).unwrap_or(0) * GIGABYTE) * ONE_THOUSAND / 4;
        let sru_min = self.sru as u128 * ONE_THOUSAND / 50;

        if cru_min < mru_min && cru_min < sru_min {
            cru_min as u64
        } else if mru_min < cru_min && mru_min < sru_min {
            mru_min as u64
        } else if sru_min < cru_min && sru_min < mru_min {
            sru_min as u64
        } else {
            0
        }
    }

    pub fn get_su(&self) -> u64 {
        let su = self.hru as u128 * ONE_THOUSAND / 1200 + self.sru as u128 * ONE_THOUSAND / 250;
        let calculated_su = su / GIGABYTE;
        let result = calculated_su as u128 / ONE_THOUSAND;
        result as u64
    }

    pub fn get_node_weight(&self) -> u64 {
        let cu = self.get_cu();
        let su = self.get_su();
        cu * 2 + su
    }
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
}
