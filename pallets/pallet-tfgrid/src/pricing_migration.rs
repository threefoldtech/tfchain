use super::*;
use codec::{Decode, Encode};
use frame_support::weights::Weight;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct PricingPolicyV2<AccountId> {
    pub version: u32,
    pub id: u32,
    pub name: Vec<u8>,
    pub su: Policy,
    pub cu: Policy,
    pub nu: Policy,
    pub ipu: Policy,
    pub unique_name: Policy,
    pub domain_name: Policy,
    pub foundation_account: AccountId,
    pub certified_sales_account: AccountId
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct Policy {
    pub value: u32,
    pub unit: Unit,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug)]
pub enum Unit {
    Bytes,
    Kilobytes,
    Megabytes,
    Gigabytes,
    Terrabytes,
}

impl Default for Unit {
    fn default() -> Unit {
        Unit::Gigabytes
    }
}


pub mod deprecated {
    use crate::Config;
    use frame_support::decl_module;
    use sp_std::prelude::*;

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::Origin { }
    }
}

pub fn migrate_policies<T: Config>() -> frame_support::weights::Weight {
    frame_support::debug::RuntimeLogger::init();

    let policy_1 = PricingPolicies::<T>::get(1);
    if policy_1.version == 2 {
        frame_support::debug::info!(" >>> Unused migration!");
        return 0;
    }

    frame_support::debug::info!(" >>> Starting migration");

    // save number of read writes
    let mut read_writes = 0;

    PricingPolicies::<T>::translate::<PricingPolicyV2<T::AccountId>, _>(|k, policy| {
        frame_support::debug::info!("     Migrated pricingPolicy for {:?}...", k);

        let new_policy = super::types::PricingPolicy {
            version: 2,
            id: policy.id,
            name: policy.name,
            su: types::Policy{
                value: policy.su.value,
                unit: migration_unit_to_types_unit(policy.su.unit)
            },
            cu: types::Policy{
                value: policy.cu.value,
                unit: migration_unit_to_types_unit(policy.cu.unit)
            },
            nu: types::Policy{
                value: policy.nu.value,
                unit: migration_unit_to_types_unit(policy.nu.unit)
            },
            ipu: types::Policy{
                value: policy.ipu.value,
                unit: migration_unit_to_types_unit(policy.ipu.unit)
            },
            unique_name: types::Policy{
                value: policy.unique_name.value,
                unit: migration_unit_to_types_unit(policy.unique_name.unit)
            },
            domain_name: types::Policy{
                value: policy.domain_name.value,
                unit: migration_unit_to_types_unit(policy.domain_name.unit)
            },
            foundation_account: policy.foundation_account,
            certified_sales_account: policy.certified_sales_account,
            // set discount to 50%
            discount_for_dedication_nodes: 50
        };

        read_writes += 1;
        Some(new_policy)
    });

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(read_writes as Weight + 1, read_writes as Weight + 1)
}

fn migration_unit_to_types_unit(unit: pricing_migration::Unit) -> types::Unit {
    match unit {
        pricing_migration::Unit::Bytes => types::Unit::Bytes,
        pricing_migration::Unit::Kilobytes => types::Unit::Kilobytes,
        pricing_migration::Unit::Megabytes => types::Unit::Megabytes,
        pricing_migration::Unit::Gigabytes => types::Unit::Gigabytes,
        pricing_migration::Unit::Terrabytes => types::Unit::Terrabytes,
    }
}