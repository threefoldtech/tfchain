use super::*;
use frame_support::weights::Weight;

pub mod deprecated {
    use crate::Config;
    use frame_support::decl_module;
    use sp_std::prelude::*;

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::Origin { }
    }
}

pub fn migrate_node_contracts<T: Config>() -> frame_support::weights::Weight {
    frame_support::debug::RuntimeLogger::init();

    frame_support::debug::info!(" >>> Starting migration");

    // save number of read writes
    let mut read_writes = 0;

    let last_contract = ContractID::get();
    for ctr in 0..last_contract {
        let contract = Contracts::get(ctr);
        if !contract.is_state_delete() {
            continue
        }

        frame_support::debug::info!(" >>> removing contract: {:?}", ctr);
        ContractBillingInformationByID::remove(ctr);
        ContractLastBilledAt::remove(ctr);
        Contracts::remove(ctr);
        read_writes+=3;
    };

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(read_writes as Weight + 1, read_writes as Weight + 1)
}
