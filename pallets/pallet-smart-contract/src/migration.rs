use super::*;
use frame_support::weights::Weight;
use codec::{Decode, Encode};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct ContractV4 {
    pub version: u32,
    pub state: ContractState,
    pub contract_id: u64,
    pub twin_id: u32,
    pub contract_type: super::types::ContractData,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug)]
pub enum ContractState {
    Created,
    Deleted(Cause),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug)]
pub enum Cause {
    CanceledByUser,
    OutOfFunds,
}

impl Default for ContractState {
    fn default() -> ContractState {
        ContractState::Created
    }
}


pub mod deprecated {
    use crate::Config;
    use frame_support::{decl_module};
    use sp_std::prelude::*;

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::Origin { }
    }
}

pub fn migrate_node_contracts<T: Config>() -> frame_support::weights::Weight {
    frame_support::debug::RuntimeLogger::init();

    let version = PalletVersion::get();
    frame_support::debug::info!(" >>> Version: {:?}", version);

    if version != types::PalletStorageVersion::V3 {
        frame_support::debug::info!(" >>> Unused migration!");
        return 0
    }

    frame_support::debug::info!(" >>> Starting migration");

    // save number of read writes
    let mut read_writes = 0;

    Contracts::translate::<ContractV4, _>(
        |k, ctr| {
            frame_support::debug::info!("     Migrated contract for {:?}...", k);

            let new_state = match ctr.state {
                ContractState::Created => super::types::ContractState::Created,
                ContractState::Deleted(Cause::CanceledByUser) => super::types::ContractState::Deleted(super::types::Cause::CanceledByUser),
                ContractState::Deleted(Cause::OutOfFunds) => super::types::ContractState::Deleted(super::types::Cause::OutOfFunds),
            };

            let new_contract = super::types::Contract {
                version: 2,
                state: new_state,
                contract_id: ctr.contract_id,
                twin_id: ctr.twin_id,
                contract_type: ctr.contract_type
            };

            read_writes+=1;
            Some(new_contract)
    });

    frame_support::debug::info!(" >>> Starting clean up of contract billing infos");
    for (ctr_id, mut contract) in Contracts::iter() {
        if matches!(contract.state, types::ContractState::Deleted(_)) {
            frame_support::debug::info!(" >>> Cleaning up contract billing information of deleted contract: {:?}", ctr_id);
            contract.state = types::ContractState::Killed;
            ContractBillingInformationByID::remove(contract.contract_id);
            ContractLastBilledAt::remove(contract.contract_id);
            Contracts::insert(contract.contract_id, contract.clone());
            read_writes+=5;
        }
    }

    // Update pallet version to V4
    PalletVersion::put(types::PalletStorageVersion::V4);

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(read_writes as Weight + 1, read_writes as Weight + 1)
} 