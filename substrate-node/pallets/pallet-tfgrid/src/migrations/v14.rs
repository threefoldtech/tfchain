use crate::{
    ip::{GW4, GW6, IP4, IP6},
    types,
    types::StorageVersion,
    Config, Error, FarmInfoOf, Farms, InterfaceOf, LocationOf, Nodes, PalletVersion, PubConfigOf,
    PublicIpOf, SerialNumberOf, TFGRID_NODE_VERSION,
};
use frame_support::{
    pallet_prelude::Weight, traits::ConstU32, traits::Get, traits::OnRuntimeUpgrade, BoundedVec,
};
use log::{debug, info};
use sp_std::marker::PhantomData;
use tfchain_support::{
    traits::PublicIpModifier,
    types::{Farm, Node},
};

use valip::ip4::{Ip as IPv4, CIDR as IPv4Cidr};
use valip::ip6::{Ip as IPv6, CIDR as IPv6Cidr};

#[cfg(feature = "try-runtime")]
use frame_support::traits::OnRuntimeUpgradeHelpersExt;

pub struct FixPublicIP<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for FixPublicIP<T>
where
    IP4<T>: From<<T as Config>::IP4>,
    GW4<T>: From<<T as Config>::GW4>,
    IP6<T>: From<<T as Config>::IP6>,
    GW6<T>: From<<T as Config>::GW6>,
{
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        assert!(PalletVersion::<T>::get() <= types::StorageVersion::V13Struct);

        info!("current pallet version: {:?}", PalletVersion::<T>::get());
        let nodes_count: u64 = Nodes::<T>::iter_keys().count() as u64;
        Self::set_temp_storage(nodes_count, "pre_node_count");
        log::info!(
            "🔎 FixPublicIPV13 pre migration: Number of existing nodes {:?}",
            nodes_count
        );
        Ok(())
    }

    fn on_runtime_upgrade() -> Weight {
        if PalletVersion::<T>::get() == StorageVersion::V13Struct {
            migrate_nodes::<T>() + migrate_farms::<T>()
        } else {
            info!(" >>> Unused migration");
            0
        }
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V14);
        // Check number of nodes against pre-check result
        let pre_nodes_count = Self::get_temp_storage("pre_node_count").unwrap_or(0u64);
        assert_eq!(
            Nodes::<T>::iter().count() as u64,
            pre_nodes_count,
            "Number of nodes migrated does not match"
        );

        info!(
            "👥  FixPublicIPV13 post migration: migration to {:?} passes POST migrate checks ✅",
            PalletVersion::<T>::get()
        );

        Ok(())
    }
}

pub fn migrate_nodes<T: Config>() -> frame_support::weights::Weight
where
    IP4<T>: From<<T as Config>::IP4>,
    GW4<T>: From<<T as Config>::GW4>,
    IP6<T>: From<<T as Config>::IP6>,
    GW6<T>: From<<T as Config>::GW6>,
{
    info!(
        " >>> Starting tfgrid pallet migration, pallet version: {:?}",
        PalletVersion::<T>::get()
    );

    let mut migrated_count = 0;

    Nodes::<T>::translate::<
        super::types::v13::Node<LocationOf<T>, PubConfigOf<T>, InterfaceOf<T>, SerialNumberOf<T>>,
        _,
    >(|k, n| {
        migrated_count += 1;

        // By default initialize the public config to None
        let mut public_config = None;
        // If the node has a valid public config we can assign it
        if let Some(config) = n.public_config {
            match validate_public_config_ip4::<T>(config) {
                Ok(config) => {
                    if matches!(config, None) {
                        info!("resetting pub config of node: {:?}", k);
                    }
                    public_config = config;
                }
                Err(e) => {
                    public_config = None;
                    info!(
                        "failed to parse pub config for node: {:?}, error: {:?}",
                        k, e
                    );
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
            power: n.power,
            // optional public config
            public_config: public_config,
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
        Some(new_node)
    });

    info!(
        " <<< Node storage updated! Migrated {} Nodes ✅",
        migrated_count
    );

    // Update pallet storage version
    PalletVersion::<T>::set(StorageVersion::V13Struct);
    info!(" <<< Storage version upgraded");

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(migrated_count as Weight + 1, migrated_count as Weight + 1)
}

use sp_std::vec;

pub fn migrate_farms<T: Config>() -> frame_support::weights::Weight
where
    IP4<T>: From<<T as Config>::IP4>,
    GW4<T>: From<<T as Config>::GW4>,
{
    info!(" >>> Migrating farms storage...");

    let mut migrated_count = 0;
    // We transform the storage values from the old into the new format.
    Farms::<T>::translate::<FarmInfoOf<T>, _>(|k, farm| {
        let mut public_ips: BoundedVec<PublicIpOf<T>, ConstU32<256>> = vec![].try_into().unwrap();

        match validate_public_ips::<T>(&farm) {
            Ok(ips) => {
                public_ips = ips;
            }
            Err(e) => {
                info!(
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

fn validate_public_config_ip4<T: Config>(
    config: PubConfigOf<T>,
) -> Result<Option<PubConfigOf<T>>, Error<T>>
where
    IP4<T>: From<<T as Config>::IP4>,
    GW4<T>: From<<T as Config>::GW4>,
    IP6<T>: From<<T as Config>::IP6>,
    GW6<T>: From<<T as Config>::GW6>,
{
    let config_ip4: IP4<T> = config
        .clone()
        .ip4
        .ip
        .try_into()
        .map_err(|_| Error::<T>::InvalidPublicIP)?;

    let config_gw4: GW4<T> = config
        .clone()
        .ip4
        .gw
        .try_into()
        .map_err(|_| Error::<T>::InvalidPublicIP)?;

    let gw4 = IPv4::parse(&config_gw4.0).map_err(|_| Error::<T>::InvalidPublicIP)?;
    let ip4 = IPv4Cidr::parse(&config_ip4.0).map_err(|_| Error::<T>::InvalidPublicIP)?;

    if gw4.is_public()
        && gw4.is_unicast()
        && ip4.is_public()
        && ip4.is_unicast()
        && ip4.contains(gw4)
    {
        if let Some(ipv6) = config.clone().ip6 {
            let ip6: IP6<T> = ipv6
                .ip
                .try_into()
                .map_err(|_| Error::<T>::InvalidPublicIP)?;
            let gw6: GW6<T> = ipv6
                .gw
                .try_into()
                .map_err(|_| Error::<T>::InvalidPublicIP)?;

            let ip6_parsed = IPv6Cidr::parse(&ip6.0).map_err(|_| Error::<T>::InvalidIP6)?;
            let gw6_parsed = IPv6::parse(&gw6.0).map_err(|_| Error::<T>::InvalidIP6)?;
            if ip6_parsed.is_public() && ip6_parsed.is_unicast() && gw6_parsed.is_public()
            // && gw6_parsed.is_unicast()
            // && gw6_parsed.contains(ip6_parsed)
            {
                return Ok(Some(config));
            } else {
                return Ok(None);
            }
        };

        return Ok(Some(config));
    } else {
        return Ok(None);
    }
}

fn validate_public_ips<T: Config>(
    farm: &FarmInfoOf<T>,
) -> Result<BoundedVec<PublicIpOf<T>, ConstU32<256>>, Error<T>>
where
    IP4<T>: From<<T as Config>::IP4>,
    GW4<T>: From<<T as Config>::GW4>,
{
    let mut parsed_public_ips: BoundedVec<PublicIpOf<T>, ConstU32<256>> =
        vec![].try_into().unwrap();

    for pub_ip in farm.public_ips.clone().into_iter() {
        let parsed_ip: IP4<T> = pub_ip
            .clone()
            .ip
            .try_into()
            .map_err(|_| Error::<T>::InvalidPublicIP)?;

        let parsed_gw: GW4<T> = pub_ip
            .clone()
            .gateway
            .try_into()
            .map_err(|_| Error::<T>::InvalidPublicIP)?;

        let gw4 = IPv4::parse(&parsed_gw.0).map_err(|_| Error::<T>::InvalidPublicIP)?;
        let ip4 = IPv4Cidr::parse(&parsed_ip.0).map_err(|_| Error::<T>::InvalidPublicIP)?;

        if gw4.is_public()
            && gw4.is_unicast()
            && ip4.is_public()
            && ip4.is_unicast()
            && ip4.contains(gw4)
        {
            let _ = parsed_public_ips.try_push(pub_ip);
        } else {
            if pub_ip.contract_id != 0 {
                T::PublicIpModifier::ip_removed(&pub_ip)
            }

            continue;
        }
    }

    Ok(parsed_public_ips)
}
