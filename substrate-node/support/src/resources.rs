use super::types::Resources;

pub const ONE_THOUSAND: u128 = 1_000;
pub const GIB: u128 = 1024 * 1024 * 1024;

pub fn get_cu(resources: Resources) -> u64 {
    let cru_min = resources.cru as u128 * 2 * GIB * ONE_THOUSAND;
    let mru_min = (resources.mru as u128 - 1 * GIB) * ONE_THOUSAND / 4;
    let sru_min = resources.sru as u128 * ONE_THOUSAND / 50;

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

pub fn get_su(resources: Resources) -> u64 {
    let su =
        resources.hru as u128 * ONE_THOUSAND / 1200 + resources.sru as u128 * ONE_THOUSAND / 250;
    let calculated_su = su / GIB;
    calculated_su as u64
}
