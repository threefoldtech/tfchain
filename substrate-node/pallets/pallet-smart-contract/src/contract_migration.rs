use super::*;
use frame_support::weights::Weight;

pub mod deprecated {
    use crate::Config;
    use codec::{Decode, Encode};
    use frame_support::decl_module;
    use sp_std::prelude::*;

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
    pub struct ContractV3 {
        pub version: u32,
        pub state: super::types::ContractState,
        pub contract_id: u64,
        pub twin_id: u32,
        pub contract_type: super::types::ContractData,
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::Origin { }
    }
}

pub fn add_solution_provider_to_contract<T: Config>() -> frame_support::weights::Weight {
    if PalletVersion::<T>::get() == types::StorageVersion::V3 {
        frame_support::log::info!(
            " >>> Starting migration, pallet version: {:?}",
            PalletVersion::<T>::get()
        );
        let count = Contracts::<T>::iter().count();
        frame_support::log::info!(" >>> Updating Contracts storage. Migrating {} Contracts...", count);

        let mut migrated_count = 0;
        // We transform the storage values from the old into the new format.
        Contracts::<T>::translate::<deprecated::ContractV3, _>(|k, ctr| {
            frame_support::log::info!("     Migrated contract for {:?}...", k);

            let new_contract = types::Contract {
                version: 4,
                state: ctr.state,
                contract_id: ctr.contract_id,
                twin_id: ctr.twin_id,
                contract_type: ctr.contract_type,
                solution_provider_id: None
            };

            migrated_count += 1;

            Some(new_contract)
        });
        frame_support::log::info!(
            " <<< Contracts storage updated! Migrated {} Contracts ✅",
            migrated_count
        );

        // Update pallet storage version
        PalletVersion::<T>::set(types::StorageVersion::V4);
        frame_support::log::info!(" <<< Storage version upgraded");

        // Return the weight consumed by the migration.
        T::DbWeight::get().reads_writes(migrated_count as Weight + 1, migrated_count as Weight + 1)
    } else {
        frame_support::log::info!(" >>> Unused migration");
        return 0;
    }
}