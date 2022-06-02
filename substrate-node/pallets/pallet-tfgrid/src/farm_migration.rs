use super::*;
use frame_support::weights::Weight;
use tfchain_support::types::Certification;

pub mod deprecated {
    use codec::{Decode, Encode};
    use crate::Config;
    use frame_support::{decl_module};
    use sp_std::prelude::*;
    use tfchain_support::types::{PublicIP, CertificationType};

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
    pub struct FarmV2 {
        pub version: u32,
        pub id: u32,
        pub name: Vec<u8>,
        pub twin_id: u32,
        pub pricing_policy_id: u32,
        pub certification_type: CertificationType,
        pub public_ips: Vec<PublicIP>,
        pub dedicated_farm: bool
    }    

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::Origin { }
    }
}

pub fn rework_farm_certification<T: Config>() -> frame_support::weights::Weight {
    frame_support::debug::RuntimeLogger::init();

    if PalletVersion::get() == types::StorageVersion::V3Struct {
        frame_support::debug::info!(" >>> Starting migration, pallet version: {:?}", PalletVersion::get());
        let count = Farms::iter().count();
        frame_support::debug::info!(" >>> Updating Farms storage. Migrating {} Farms...", count);
    
        let mut migrated_count = 0;
        // We transform the storage values from the old into the new format.
        Farms::translate::<deprecated::FarmV2, _>(
            |k, farm| {
                frame_support::debug::info!("     Migrated node for {:?}...", k);
    
                let new_farm = Farm {
                    version: 3,
                    id: farm.id,
                    name: farm.name,
                    twin_id: farm.twin_id,
                    pricing_policy_id: farm.pricing_policy_id,
                    certification: Certification::NotCertified,
                    public_ips: farm.public_ips,
                    dedicated_farm: farm.dedicated_farm,
                    farming_policy_limits: None
                };
    
                migrated_count+=1;
    
                Some(new_farm)
            }
        );
        frame_support::debug::info!(" <<< Node storage updated! Migrated {} farms ✅", migrated_count);
    
        // Update pallet storage version
        PalletVersion::set(types::StorageVersion::V4Struct);
        frame_support::debug::info!(" <<< Storage version upgraded");
    
        // Return the weight consumed by the migration.
        T::DbWeight::get().reads_writes(migrated_count as Weight + 1, migrated_count as Weight + 1)
    }  else {
        frame_support::debug::info!(" >>> Unused migration");
        return 0;
    }
}