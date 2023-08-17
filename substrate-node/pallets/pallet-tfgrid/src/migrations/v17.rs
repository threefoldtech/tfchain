use crate::*;
use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};
use log::{debug, info};
use sp_core::Get;
use sp_runtime::Saturating;
use sp_std::marker::PhantomData;

#[cfg(feature = "try-runtime")]
use sp_std::{vec, vec::Vec};

pub struct CleanStorageState<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for CleanStorageState<T> {
    fn on_runtime_upgrade() -> Weight {
        if PalletVersion::<T>::get() == types::StorageVersion::V16Struct {
            clean_pallet_tfgrid::<T>()
        } else {
            info!("⛔ Unused TFGrid pallet V16 storage cleaning");
            Weight::zero()
        }
    }
}

pub fn clean_pallet_tfgrid<T: Config>() -> frame_support::weights::Weight {
    info!("🧼 Cleaning TFGrid pallet storagem");
    let mut weight = clean_farm_id_by_name::<T>();
    weight.saturating_accrue(clean_farm_payout_v2_address_by_farm_id::<T>());
    weight.saturating_accrue(clean_node_id_by_twin_id::<T>());

    PalletVersion::<T>::put(types::StorageVersion::V17Struct);
    info!("🔔 Ending TFGrid pallet storage cleaning");
    weight.saturating_add(T::DbWeight::get().writes(1))
}

// Farms
pub fn clean_farms<T: Config>() -> frame_support::weights::Weight {
    debug!(
        "🧼  TFGrid pallet {:?} cleaning Farms storage map START",
        PalletVersion::<T>::get()
    );

    let mut r = 0u64;
    let mut w = 0u64;

    let to_remove: Vec<u32> = Farms::<T>::iter()
        .filter(|(_, farm)| {
            r.saturating_accrue(2);
            Twins::<T>::get(farm.twin_id).is_none()
        })
        .map(|(id, _)| id)
        .collect();

    for farm_id in to_remove {
        Farms::<T>::remove(farm_id);
        w.saturating_inc();
    }

    debug!(
        "✨  TFGrid pallet {:?} cleaning Farms storage map END",
        PalletVersion::<T>::get()
    );

    T::DbWeight::get().reads_writes(r.saturating_add(2), w)
}

// FarmIdByName
pub fn clean_farm_id_by_name<T: Config>() -> frame_support::weights::Weight {
    debug!(
        "🧼  TFGrid pallet {:?} cleaning FarmIdByName storage map START",
        PalletVersion::<T>::get()
    );

    let mut r = 0u64;
    let mut w = 0u64;

    let mut to_remove = vec![];
    let mut to_reinsert = vec![];

    for (farm_name, farm_id) in FarmIdByName::<T>::iter() {
        r.saturating_inc();
        if let Some(f) = Farms::<T>::get(farm_id) {
            if f.name.clone().into() != farm_name {
                to_remove.push(farm_name);
                to_reinsert.push((f.name, farm_id));
            }
        } else {
            to_remove.push(farm_name);
        }
    }

    // 1. Remove obsolete entries
    for farm_name in to_remove {
        FarmIdByName::<T>::remove(farm_name);
        w.saturating_inc();
    }

    // 2. Re-insert entries with same name as set in farm
    for (farm_name, farm_id) in to_reinsert {
        FarmIdByName::<T>::insert(farm_name.into(), farm_id);
        w.saturating_inc();
    }

    debug!(
        "✨  TFGrid pallet {:?} cleaning FarmIdByName storage map END",
        PalletVersion::<T>::get()
    );

    T::DbWeight::get().reads_writes(r.saturating_add(2), w)
}

// FarmPayoutV2AddressByFarmID
pub fn clean_farm_payout_v2_address_by_farm_id<T: Config>() -> frame_support::weights::Weight {
    debug!(
        "🧼  TFGrid pallet {:?} cleaning FarmPayoutV2AddressByFarmID storage map START",
        PalletVersion::<T>::get()
    );

    let mut r = 0u64;
    let mut w = 0u64;

    let to_remove: Vec<u32> = FarmPayoutV2AddressByFarmID::<T>::iter()
        .filter(|(farm_id, _)| {
            r.saturating_accrue(2);
            Farms::<T>::get(farm_id).is_none()
        })
        .map(|(id, _)| id)
        .collect();

    for farm_id in to_remove {
        FarmPayoutV2AddressByFarmID::<T>::remove(farm_id);
        w.saturating_inc();
    }

    debug!(
        "✨  TFGrid pallet {:?} cleaning FarmPayoutV2AddressByFarmID storage map END",
        PalletVersion::<T>::get()
    );

    T::DbWeight::get().reads_writes(r.saturating_add(2), w)
}

// NodeIdByTwinID
pub fn clean_node_id_by_twin_id<T: Config>() -> frame_support::weights::Weight {
    debug!(
        "🧼  TFGrid pallet {:?} cleaning NodeIdByTwinID storage map START",
        PalletVersion::<T>::get()
    );

    let mut r = 0u64;
    let mut w = 0u64;

    let to_remove: Vec<u32> = NodeIdByTwinID::<T>::iter()
        .filter(|(_, node_id)| {
            r.saturating_accrue(2);
            Nodes::<T>::get(node_id).is_none()
        })
        .map(|(id, _)| id)
        .collect();

    for twin_id in to_remove {
        NodeIdByTwinID::<T>::remove(twin_id);
        w.saturating_inc();
    }

    debug!(
        "✨  TFGrid pallet {:?} cleaning NodeIdByTwinID storage map END",
        PalletVersion::<T>::get()
    );

    T::DbWeight::get().reads_writes(r.saturating_add(2), w)
}

pub struct CheckStorageState<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for CheckStorageState<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        assert!(PalletVersion::<T>::get() == types::StorageVersion::V17Struct);

        check_pallet_tfgrid::<T>();

        Ok(vec![])
    }
}

pub fn check_pallet_tfgrid<T: Config>() {
    migrations::v16::check_pallet_tfgrid::<T>();
}
