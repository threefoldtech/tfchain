use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::Percent;

/// A resources capacity that countains HRU, SRU, CRU and MRU in integer values.
#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Encode,
    Decode,
    Default,
    Debug,
    TypeInfo,
    MaxEncodedLen,
    Copy,
)]
pub struct Resources {
    pub hru: u64,
    pub sru: u64,
    pub cru: u64,
    pub mru: u64,
}

pub const ONE_THOUSAND: u128 = 1_000;
pub const ONE_MILL: u128 = 1_000_000;
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
        // No HRU minimun requirement
        true
    }

    pub fn validate_sru(&self) -> bool {
        // SRU minimum of 100 GB
        self.sru as u128 >= 100 * GIGABYTE
    }

    pub fn validate_cru(&self) -> bool {
        // CRU minimum of 1 vCPU
        self.cru >= 1
    }

    pub fn validate_mru(&self) -> bool {
        // MRU minimum of 2GB
        self.mru as u128 >= 2 * GIGABYTE
    }

    /// Compute the CU, SU and NU for the node. The result is expressed in a "permill" way. So the
    /// actual CU, SU and NU are obtained by dividing the results by 1_000_000.
    ///
    /// In order for this to be accurate, the data about network and IP usage needs to already have
    /// been aggregated on the node object.
    ///
    /// Calculation taken from [the
    /// wiki](https://library.threefold.me/info/threefold#/tfgrid/farming/threefold__resource_units_calc_cloudunits)
    /// on 31-01-2022 as follows:
    ///   CU: MIN(cru * 4 / 2, (mru - 1) / 4, sru / 50)
    ///   SU: hru / 1200 + sru * 0.8 / 200
    pub fn cloud_units_permill(&self) -> (u64, u64) {
        // Calculate CU first. Mru and sru are in bytes, but are expressed in GB in the formula.
        // Rather than converting mru and sru to GB (by dividing first), we multiply cru first
        // instead (to put the values at same scale), then take the MIN, and finally divide the
        // returning values at the end of function. This eliminates the issue of rounding errors
        // _BEFORE_ the MIN. Also multiply by 1000000 so we have precision without working with
        // floats.
        // MIN is associative.
        let cu_intermediate = sp_std::cmp::min(
            self.cru as u128 * 2 * GIGABYTE * ONE_MILL,
            (self.mru as u128 - 1 * GIGABYTE) * ONE_MILL / 4,
        );
        let cu = sp_std::cmp::min(cu_intermediate, self.sru as u128 * ONE_MILL / 50);
        let su = self.hru as u128 * ONE_MILL / 1200 + self.sru as u128 * ONE_MILL / 250;
        ((cu / GIGABYTE) as u64, (su / GIGABYTE) as u64)
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

    pub fn get_cu_su(&self) -> (u64, u64) {
        (self.get_cu(), self.get_su())
    }

    pub fn get_node_weight(&self) -> u64 {
        let cu = self.get_cu();
        let su = self.get_su();
        cu * 2 + su
    }

    pub fn has_changed(
        resources_before: &Resources,
        resources_after: &Resources,
        tolerance: u8,
    ) -> bool {
        let wiggle = |a: u64, b: u64| -> bool {
            let p = Percent::from_percent(tolerance) * a;
            let diff = (a as i64 - b as i64).abs() as u64;
            if diff > p {
                return true;
            }
            return false;
        };

        return wiggle(resources_before.cru, resources_after.cru)
            || wiggle(resources_before.sru, resources_after.sru)
            || wiggle(resources_before.hru, resources_after.hru)
            || wiggle(resources_before.mru, resources_after.mru);
    }

    pub fn is_empty(&self) -> bool {
        return self.cru == 0 && self.hru == 0 && self.mru == 0 && self.hru == 0;
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
}
