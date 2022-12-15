use crate::{
    types, types::StorageVersion, Config, Error, FarmInfoOf, Farms, InterfaceOf, LocationOf, Nodes,
    PalletVersion, SerialNumberOf, TFGRID_NODE_VERSION,
};
use frame_support::{
    pallet_prelude::Weight, traits::ConstU32, traits::Get, traits::OnRuntimeUpgrade, BoundedVec,
};
use log::{debug, info};
use sp_std::{marker::PhantomData, vec};
use tfchain_support::{
    traits::PublicIpModifier,
    types::{Farm, Node, PublicIP, IP4},
};

#[cfg(feature = "try-runtime")]
use frame_support::traits::OnRuntimeUpgradeHelpersExt;

pub struct FixPublicIP<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for FixPublicIP<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        assert!(PalletVersion::<T>::get() <= types::StorageVersion::V12Struct);

        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        let nodes_count: u64 = Nodes::<T>::iter_keys().count() as u64;
        Self::set_temp_storage(nodes_count, "pre_node_count");
        log::info!(
            "🔎 FixPublicIP pre migration: Number of existing nodes {:?}",
            nodes_count
        );

        info!("👥  TFGrid pallet to V13 passes PRE migrate checks ✅",);
        Ok(())
    }

    fn on_runtime_upgrade() -> Weight {
        if PalletVersion::<T>::get() == StorageVersion::V12Struct {
            migrate_nodes::<T>() + migrate_farms::<T>()
        } else {
            info!(" >>> Unused migration");
            0
        }
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V13);
        // Check number of nodes against pre-check result
        let pre_nodes_count = Self::get_temp_storage("pre_node_count").unwrap_or(0u64);
        assert_eq!(
            Nodes::<T>::iter().count() as u64,
            pre_nodes_count,
            "Number of nodes migrated does not match"
        );

        info!(
            "👥  FixPublicIP post migration: migration to {:?} passes POST migrate checks ✅",
            PalletVersion::<T>::get()
        );

        Ok(())
    }
}

pub fn migrate_nodes<T: Config>() -> frame_support::weights::Weight {
    info!(
        " >>> Starting tfgrid pallet migration, pallet version: {:?}",
        PalletVersion::<T>::get()
    );

    let mut migrated_count = 0;

    Nodes::<T>::translate::<
        super::types::v12::Node<LocationOf<T>, InterfaceOf<T>, SerialNumberOf<T>>,
        _,
    >(|k, n| {
        // By default initialize the public config to None
        let mut public_config = None;
        if let Some(config) = n.public_config {
            // If the config is valid, keep it, otherwise discard
            match config.is_valid() {
                Ok(_) => public_config = Some(config),
                Err(_) => {
                    debug!("resetting pub config of node: {:?}", k);
                    public_config = None;
                }
            }
        }

        let new_node = Node {
            version: TFGRID_NODE_VERSION,
            id: n.id,
            farm_id: n.farm_id,
            twin_id: n.twin_id,
            resources: n.resources,
            location: n.location,
            public_config,
            created: n.created,
            farming_policy_id: n.farming_policy_id,
            interfaces: n.interfaces,
            certification: n.certification,
            secure_boot: n.secure_boot,
            virtualized: n.virtualized,
            serial_number: n.serial_number,
            connection_price: n.connection_price,
        };
        debug!("Node: {:?} succesfully migrated", k);
        migrated_count += 1;
        Some(new_node)
    });

    info!(
        " <<< Node storage updated! Migrated {} Nodes ✅",
        migrated_count
    );

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(migrated_count as Weight + 1, migrated_count as Weight + 1)
}

pub fn migrate_farms<T: Config>() -> frame_support::weights::Weight {
    info!(" >>> Migrating farms storage...");

    let mut migrated_count = 0;
    // We transform the storage values from the old into the new format.
    Farms::<T>::translate::<FarmInfoOf<T>, _>(|k, farm| {
        let mut public_ips: BoundedVec<PublicIP, ConstU32<256>> = vec![].try_into().unwrap();

        match validate_public_ips::<T>(&farm) {
            Ok(ips) => {
                public_ips = ips;
            }
            Err(e) => {
                debug!(
                    "failed to parse public ips for farm: {:?}, error: {:?}",
                    k, e
                )
            }
        }

        let new_farm = Farm {
            version: 4,
            id: farm.id,
            name: farm.name,
            twin_id: farm.twin_id,
            pricing_policy_id: farm.pricing_policy_id,
            certification: farm.certification,
            public_ips,
            dedicated_farm: farm.dedicated_farm,
            farming_policy_limits: farm.farming_policy_limits,
        };

        migrated_count += 1;

        debug!("Farm: {:?} succesfully migrated", k);
        Some(new_farm)
    });

    info!(
        " <<< Farm storage updated! Migrated {} Farms ✅",
        migrated_count
    );

    // Update pallet storage version
    PalletVersion::<T>::set(types::StorageVersion::V14);
    info!(" <<< Storage version upgraded");

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(migrated_count as Weight + 1, migrated_count as Weight + 1)
}

fn validate_public_ips<T: Config>(
    farm: &FarmInfoOf<T>,
) -> Result<BoundedVec<PublicIP, ConstU32<256>>, Error<T>> {
    let mut parsed_public_ips: BoundedVec<PublicIP, ConstU32<256>> = vec![].try_into().unwrap();

    for pub_ip in farm.public_ips.clone().into_iter() {
        let ip4 = IP4 {
            ip: pub_ip.ip.clone(),
            gw: pub_ip.gateway.clone(),
        };

        match ip4.is_valid() {
            Ok(_) => {
                let _ = parsed_public_ips.try_push(pub_ip);
            }
            Err(_) => {
                debug!("resetting farm ip for farm {:?}", farm.id);
                if pub_ip.contract_id != 0 {
                    T::PublicIpModifier::ip_removed(&pub_ip)
                }

                continue;
            }
        }
    }

    Ok(parsed_public_ips)
}
