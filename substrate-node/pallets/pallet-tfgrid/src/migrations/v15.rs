use crate::*;
use frame_support::{traits::Get, traits::OnRuntimeUpgrade, weights::Weight};
use log::{debug, info};
use scale_info::prelude::string::String;
use sp_core::H256;
use sp_std::{marker::PhantomData, vec::Vec};

#[cfg(feature = "try-runtime")]
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "try-runtime")]
use sp_std::vec;

pub struct MigrateTwinsV15<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateTwinsV15<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V14Struct);

        let twins_count: u64 = Twins::<T>::iter().count() as u64;
        log::info!(
            "🔎 MigrateTwinsV15 pre migration: Number of existing twins {:?}",
            twins_count
        );

        info!("👥  TFGrid pallet to v14 passes PRE migrate checks ✅",);
        Ok(twins_count.encode())
    }

    fn on_runtime_upgrade() -> Weight {
        if PalletVersion::<T>::get() == types::StorageVersion::V14Struct {
            migrate_twins::<T>()
        } else {
            info!(" >>> Unused TFGrid pallet V15 migration");
            Weight::zero()
        }
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(pre_twins_count: Vec<u8>) -> Result<(), &'static str> {
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V15Struct);

        // Check number of twins against pre-check result
        let pre_twins_count: u64 = Decode::decode(&mut pre_twins_count.as_slice())
            .expect("the state parameter should be something that was generated by pre_upgrade");
        assert_eq!(
            Twins::<T>::iter().count() as u64,
            pre_twins_count,
            "Number of twins migrated does not match"
        );

        info!(
            "👥  TFGrid pallet migration to {:?} passes POST migrate checks ✅",
            Pallet::<T>::pallet_version()
        );

        Ok(())
    }
}

pub fn migrate_twins<T: Config>() -> frame_support::weights::Weight {
    info!(" >>> Migrating twin storage...");

    let mut read_writes = 0;

    Twins::<T>::translate::<super::types::v14::Twin<Vec<u8>, AccountIdOf<T>>, _>(|k, twin| {
        debug!("migrated twin: {:?}", k);

        let new_twin = types::Twin {
            id: twin.id,
            account_id: twin.account_id,
            relay: None,
            entities: twin.entities,
            pk: None,
        };

        read_writes += 1;
        Some(new_twin)
    });

    // Update pallet storage version
    PalletVersion::<T>::set(types::StorageVersion::V15Struct);
    info!(" <<< Twin migration success, storage version upgraded");

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(read_writes, read_writes + 1)
}

pub struct CheckStorageState<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for CheckStorageState<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        assert!(PalletVersion::<T>::get() == types::StorageVersion::V15Struct);

        check_pallet_tfgrid::<T>();

        Ok(vec![])
    }
}

pub fn check_pallet_tfgrid<T: Config>() {
    info!("💥💥💥💥💥 CHECKING PALLET TFGRID STORAGE 💥💥💥💥💥");
    check_farms::<T>();
    check_nodes_by_farm_id::<T>();
    check_farm_id_by_name::<T>();
    check_farm_payout_v2_address_by_farm_id::<T>();
    check_nodes::<T>();
    check_node_id_by_twin_id::<T>();
    // check_entities::<T>();
    // check_entity_id_by_account_id::<T>();
    // check_entity_id_by_name::<T>();
    check_twins::<T>();
    check_twin_id_by_account_id::<T>();
    check_twin_bounded_account_id::<T>();
    check_pricing_policies::<T>();
    check_pricing_policy_id_by_name::<T>();
    check_farming_policies_map::<T>();
    check_users_terms_and_conditions::<T>();
}

// Farms
pub fn check_farms<T: Config>() {
    debug!(
        "🔎  TFGrid pallet {:?} checking Farms storage map START",
        PalletVersion::<T>::get()
    );

    let farm_id_range = 1..=FarmID::<T>::get();

    for (farm_id, farm) in Farms::<T>::iter() {
        if farm_id != farm.id {
            debug!(" ⚠️    Farms[id: {}]: wrong id ({})", farm_id, farm.id);
        }
        if !farm_id_range.contains(&farm_id) {
            debug!(
                " ⚠️    Farms[id: {}]: id not in range {:?}",
                farm_id, farm_id_range
            );
        }

        // FarmIdByName
        if !FarmIdByName::<T>::contains_key(farm.name.clone().into()) {
            debug!(
                " ⚠️    Farm[id: {}]: farm (name: {}) not found",
                farm_id,
                String::from_utf8_lossy(&farm.name.into())
            );
        }

        // Twins
        if !Twins::<T>::contains_key(farm.twin_id) {
            debug!(
                " ⚠️    Farm[id: {}]: twin (twin_id: {}) not found",
                farm_id, farm.twin_id
            );
        }

        // PricingPolicies
        if !PricingPolicies::<T>::contains_key(farm.pricing_policy_id) {
            debug!(
                " ⚠️    Farm[id: {}]: pricing policy (pricing_policy_id: {}) not found",
                farm_id, farm.pricing_policy_id
            );
        }

        // FarmingPoliciesMap
        if let Some(limits) = farm.farming_policy_limits {
            if !FarmingPoliciesMap::<T>::contains_key(limits.farming_policy_id) {
                debug!(
                    " ⚠️    Farm[id: {}]: farming policy (farming_policy_id: {}) not found",
                    farm_id, limits.farming_policy_id
                );
            }
        }
    }

    debug!(
        "🏁  TFGrid pallet {:?} checking Farms storage map END",
        PalletVersion::<T>::get()
    );
}

// NodesByFarmID
pub fn check_nodes_by_farm_id<T: Config>() {
    debug!(
        "🔎  TFGrid pallet {:?} checking NodesByFarmID storage map START",
        PalletVersion::<T>::get()
    );

    for (farm_id, nodes) in NodesByFarmID::<T>::iter() {
        if Farms::<T>::get(farm_id).is_none() {
            debug!(" ⚠️    NodesByFarmID[farm: {}]: farm not exists", farm_id);
        }

        for node_id in nodes {
            if Nodes::<T>::get(node_id).is_none() {
                debug!(
                    " ⚠️    NodesByFarmID[farm: {}]: node {} not exists",
                    farm_id, node_id
                );
            }
        }
    }

    debug!(
        "🏁  TFGrid pallet {:?} checking NodesByFarmID storage map END",
        PalletVersion::<T>::get()
    );
}

// FarmIdByName
pub fn check_farm_id_by_name<T: Config>() {
    debug!(
        "🔎  TFGrid pallet {:?} checking FarmIdByName storage map START",
        PalletVersion::<T>::get()
    );

    for (farm_name, farm_id) in FarmIdByName::<T>::iter() {
        if let Some(farm) = Farms::<T>::get(farm_id) {
            if farm_name != farm.name.clone().into() {
                debug!(
                    " ⚠️    FarmIdByName[name: {}]: name ({:?}) on farm {} not matching",
                    String::from_utf8_lossy(&farm_name),
                    String::from_utf8_lossy(&farm.name.into()),
                    farm_id,
                );
            }
        } else {
            debug!(
                " ⚠️    FarmIdByName[name: {}]: farm {} not exists",
                String::from_utf8_lossy(&farm_name),
                farm_id
            );
        }
    }

    debug!(
        "🏁  TFGrid pallet {:?} checking FarmIdByName storage map END",
        PalletVersion::<T>::get()
    );
}

// FarmPayoutV2AddressByFarmID
pub fn check_farm_payout_v2_address_by_farm_id<T: Config>() {
    debug!(
        "🔎  TFGrid pallet {:?} checking FarmPayoutV2AddressByFarmID storage map START",
        PalletVersion::<T>::get()
    );

    for (farm_id, _stellar_address) in FarmPayoutV2AddressByFarmID::<T>::iter() {
        if Farms::<T>::get(farm_id).is_none() {
            debug!(
                " ⚠️    FarmPayoutV2AddressByFarmID[farm id: {}]: farm not exists",
                farm_id
            );
        }
    }

    debug!(
        "🏁  TFGrid pallet {:?} checking FarmPayoutV2AddressByFarmID storage map END",
        PalletVersion::<T>::get()
    );
}

// Nodes
pub fn check_nodes<T: Config>() {
    debug!(
        "🔎  TFGrid pallet {:?} checking Nodes storage map START",
        PalletVersion::<T>::get()
    );

    let node_id_range = 1..=NodeID::<T>::get();

    for (node_id, node) in Nodes::<T>::iter() {
        if node_id != node.id {
            debug!(" ⚠️    Nodes[id: {}]: wrong id ({})", node_id, node.id);
        }
        if !node_id_range.contains(&node_id) {
            debug!(
                " ⚠️    Nodes[id: {}]: id not in range {:?}",
                node_id, node_id_range
            );
        }

        // Farms
        if !Farms::<T>::contains_key(node.farm_id) {
            debug!(
                " ⚠️    Nodes[id: {}]: farm (farm_id: {}) not found",
                node_id, node.farm_id
            );
        }

        // Twins
        if !Twins::<T>::contains_key(node.twin_id) {
            debug!(
                " ⚠️    Nodes[id: {}]: twin (twin_id: {}) not found",
                node_id, node.twin_id
            );
        }

        // FarmingPoliciesMap
        if !FarmingPoliciesMap::<T>::contains_key(node.farming_policy_id) {
            debug!(
                " ⚠️    Node[id: {}]: farming policy (farming_policy_id: {}) not found",
                node_id, node.farming_policy_id
            );
        }
    }

    debug!(
        "🏁  TFGrid pallet {:?} checking Nodes storage map END",
        PalletVersion::<T>::get()
    );
}

// NodeIdByTwinID
pub fn check_node_id_by_twin_id<T: Config>() {
    debug!(
        "🔎  TFGrid pallet {:?} checking NodeIdByTwinID storage map START",
        PalletVersion::<T>::get()
    );

    for (twin_id, node_id) in NodeIdByTwinID::<T>::iter() {
        if Twins::<T>::get(twin_id).is_none() {
            debug!(
                " ⚠️    NodeIdByTwinID[twin_id: {}]: twin not exists",
                twin_id
            );
        }

        if Nodes::<T>::get(node_id).is_none() {
            debug!(
                " ⚠️    NodeIdByTwinID[twin_id: {}]: node {} not exists",
                twin_id, node_id
            );
        }
    }

    debug!(
        "🏁  TFGrid pallet {:?} checking NodeIdByTwinID storage map END",
        PalletVersion::<T>::get()
    );
}

// Entities
// pub type EntityIdByAccountID
// pub type EntityIdByName

// Twins
pub fn check_twins<T: Config>() {
    debug!(
        "🔎  TFGrid pallet {:?} checking Twins storage map START",
        PalletVersion::<T>::get()
    );

    let twin_id_range = 1..=TwinID::<T>::get();

    for (twin_id, twin) in Twins::<T>::iter() {
        if twin_id != twin.id {
            debug!(" ⚠️    Twins[id: {}]: wrong id ({})", twin_id, twin.id);
        }
        if !twin_id_range.contains(&twin_id) {
            debug!(
                " ⚠️    Twins[id: {}]: id not in range {:?}",
                twin_id, twin_id_range
            );
        }

        // TwinIdByAccountID
        if !TwinIdByAccountID::<T>::contains_key(&twin.account_id) {
            debug!(
                " ⚠️    Twins[id: {}]: account (account_id: {:?}) not found",
                twin_id, &twin.account_id
            );
        }

        // UsersTermsAndConditions
        if !UsersTermsAndConditions::<T>::contains_key(&twin.account_id) {
            debug!(
                " ⚠️    Twins[id: {}]: users terms and conditions (account_id: {:?}) not found",
                twin_id, &twin.account_id
            );
        }
    }

    debug!(
        "🏁  TFGrid pallet {:?} checking Twins storage map END",
        PalletVersion::<T>::get()
    );
}

// TwinIdByAccountID
pub fn check_twin_id_by_account_id<T: Config>() {
    debug!(
        "🔎  TFGrid pallet {:?} checking TwinIdByAccountID storage map START",
        PalletVersion::<T>::get()
    );

    for (account_id, twin_id) in TwinIdByAccountID::<T>::iter() {
        if let Some(twin) = Twins::<T>::get(twin_id) {
            if account_id != twin.account_id {
                debug!(
                    " ⚠️    TwinIdByAccountID[account_id: {:?}]: account ({:?}) on twin {} not matching",
                    &account_id, &twin.account_id, twin_id,
                );
            }
        } else {
            debug!(
                " ⚠️    TwinIdByAccountID[account_id: {:?}]: twin {} not exists",
                &account_id, twin_id
            );
        }
    }

    debug!(
        "🏁  TFGrid pallet {:?} checking TwinIdByAccountID storage map END",
        PalletVersion::<T>::get()
    );
}

// TwinBoundedAccountID
pub fn check_twin_bounded_account_id<T: Config>() {
    debug!(
        "🔎  TFGrid pallet {:?} checking TwinBoundedAccountID storage map START",
        PalletVersion::<T>::get()
    );

    for (twin_id, _bounded_account_id) in TwinBoundedAccountID::<T>::iter() {
        if Twins::<T>::get(twin_id).is_none() {
            debug!(
                " ⚠️    TwinBoundedAccountID[twin_id: {}]: twin not exists",
                twin_id
            );
        }
    }

    debug!(
        "🏁  TFGrid pallet {:?} checking TwinBoundedAccountID storage map END",
        PalletVersion::<T>::get()
    );
}

// PricingPolicies
pub fn check_pricing_policies<T: Config>() {
    debug!(
        "🔎  TFGrid pallet {:?} checking PricingPolicies storage map START",
        PalletVersion::<T>::get()
    );

    let pricing_policy_id_range = 1..=PricingPolicyID::<T>::get();

    for (pricing_policy_id, pricing_policy) in PricingPolicies::<T>::iter() {
        if pricing_policy_id != pricing_policy.id {
            debug!(
                " ⚠️    PricingPolicies[id: {}]: wrong id ({})",
                pricing_policy_id, pricing_policy.id
            );
        }
        if !pricing_policy_id_range.contains(&pricing_policy_id) {
            debug!(
                " ⚠️    PricingPolicies[id: {}]: id not in range {:?}",
                pricing_policy_id, pricing_policy_id_range
            );
        }

        // PricingPolicyIdByName
        if !PricingPolicyIdByName::<T>::contains_key(&pricing_policy.name) {
            debug!(
                " ⚠️    PricingPolicies[id: {}]: : pricing policy (name: {}) not found",
                pricing_policy_id,
                String::from_utf8_lossy(&pricing_policy.name),
            );
        }
    }

    debug!(
        "🏁  TFGrid pallet {:?} checking PricingPolicies storage map END",
        PalletVersion::<T>::get()
    );
}

// PricingPolicyIdByName
pub fn check_pricing_policy_id_by_name<T: Config>() {
    debug!(
        "🔎  TFGrid pallet {:?} checking PricingPolicyIdByName storage map START",
        PalletVersion::<T>::get()
    );

    for (pricing_policy_name, pricing_policy_id) in PricingPolicyIdByName::<T>::iter() {
        if let Some(pricing_policy) = PricingPolicies::<T>::get(pricing_policy_id) {
            if pricing_policy_name != pricing_policy.name {
                debug!(
                    " ⚠️    PricingPolicyIdByName[name: {}]: name ({:?}) on pricing policy {} not matching",
                    String::from_utf8_lossy(&pricing_policy_name),
                    String::from_utf8_lossy(&pricing_policy.name),
                    pricing_policy_id,
                );
            }
        } else {
            debug!(
                " ⚠️    PricingPolicyIdByName[name: {}]: pricing policy {} not exists",
                String::from_utf8_lossy(&pricing_policy_name),
                pricing_policy_id
            );
        }
    }

    debug!(
        "🏁  TFGrid pallet {:?} checking Farms PricingPolicyIdByName map END",
        PalletVersion::<T>::get()
    );
}

// FarmingPoliciesMap
pub fn check_farming_policies_map<T: Config>() {
    debug!(
        "🔎  TFGrid pallet {:?} checking FarmingPoliciesMap storage map START",
        PalletVersion::<T>::get()
    );

    let farming_policy_id_range = 1..=FarmingPolicyID::<T>::get();

    for (farming_policy_id, farming_policy) in FarmingPoliciesMap::<T>::iter() {
        if farming_policy_id != farming_policy.id {
            debug!(
                " ⚠️    FarmingPoliciesMap[id: {}]: wrong id ({})",
                farming_policy_id, farming_policy.id
            );
        }
        if !farming_policy_id_range.contains(&farming_policy_id) {
            debug!(
                " ⚠️    FarmingPoliciesMap[id: {}]: id not in range {:?}",
                farming_policy_id, farming_policy_id_range
            );
        }

        // Nothing to add here
    }

    debug!(
        "🏁  TFGrid pallet {:?} checking FarmingPoliciesMap storage map END",
        PalletVersion::<T>::get()
    );
}

// UsersTermsAndConditions
pub fn check_users_terms_and_conditions<T: Config>() {
    debug!(
        "🔎  TFGrid pallet {:?} checking UsersTermsAndConditions storage map START",
        PalletVersion::<T>::get()
    );

    // Nothing to do here

    debug!(
        "🏁  TFGrid pallet {:?} checking UsersTermsAndConditions storage map END",
        PalletVersion::<T>::get()
    );
}

// NodePower
pub fn check_node_power<T: Config>() {
    debug!(
        "🔎  TFGrid pallet {:?} checking NodePower storage map START",
        PalletVersion::<T>::get()
    );

    for (twin_id, _node_power) in NodePower::<T>::iter() {
        if Twins::<T>::get(twin_id).is_none() {
            debug!(" ⚠️    NodePower[twin_id: {}]: twin not exists", twin_id);
        }
    }

    debug!(
        "🏁  TFGrid pallet {:?} checking NodePower storage map END",
        PalletVersion::<T>::get()
    );
}
