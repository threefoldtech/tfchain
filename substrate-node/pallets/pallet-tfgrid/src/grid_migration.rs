use super::Config;
use super::PubConfigOf;
use super::*;
use super::{InterfaceIp, InterfaceOf, PublicIpOf};
use frame_support::{
    bounded_vec,
    traits::{ConstU32, Get},
    weights::Weight,
    BoundedVec,
};
use log::info;
use sp_std::collections::btree_map::BTreeMap;
use tfchain_support::types::{Farm, Interface, PublicConfig, PublicIP, IP};

pub mod deprecated {
    use crate::Config;
    use codec::{Decode, Encode};
    use core::cmp::{Ord, PartialOrd};
    use frame_support::decl_module;
    use scale_info::TypeInfo;
    use sp_std::prelude::*;
    use sp_std::vec::Vec;
    use tfchain_support::resources::Resources;
    use tfchain_support::types::{
        FarmCertification, FarmingPolicyLimit, Location, NodeCertification,
    };

    #[derive(PartialEq, Eq, Clone, Encode, Decode, Default, Debug, TypeInfo)]
    pub struct FarmV3 {
        pub version: u32,
        pub id: u32,
        pub name: Vec<u8>,
        pub twin_id: u32,
        pub pricing_policy_id: u32,
        pub certification: FarmCertification,
        pub public_ips: Vec<PublicIP>,
        pub dedicated_farm: bool,
        pub farming_policy_limits: Option<FarmingPolicyLimit>,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
    pub struct PublicIP {
        pub ip: Vec<u8>,
        pub gateway: Vec<u8>,
        pub contract_id: u64,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
    pub struct NodeV4 {
        pub version: u32,
        pub id: u32,
        pub farm_id: u32,
        pub twin_id: u32,
        pub resources: Resources,
        pub location: Location,
        pub country: Vec<u8>,
        pub city: Vec<u8>,
        // optional public config
        pub public_config: Option<PublicConfig>,
        pub created: u64,
        pub farming_policy_id: u32,
        pub interfaces: Vec<Interface>,
        pub certification: NodeCertification,
        pub secure_boot: bool,
        pub virtualized: bool,
        pub serial_number: Vec<u8>,
        pub connection_price: u32,
    }

    pub type IP = Vec<u8>;

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
    pub struct Interface {
        pub name: Vec<u8>,
        pub mac: Vec<u8>,
        pub ips: Vec<IP>,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
    pub struct PublicConfig {
        pub ipv4: Vec<u8>,
        pub ipv6: Vec<u8>,
        pub gw4: Vec<u8>,
        pub gw6: Vec<u8>,
        pub domain: Vec<u8>,
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::Origin { }
    }
}

pub mod v7 {
    use super::*;
    use crate::Config;

    use frame_support::{pallet_prelude::Weight, traits::OnRuntimeUpgrade};
    use sp_std::marker::PhantomData;
    pub struct GridMigration<T: Config>(PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for GridMigration<T> {
        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<(), &'static str> {
            assert!(PalletVersion::<T>::get() == types::StorageVersion::V5Struct);

            info!("👥  TFGrid pallet to v4 passes PRE migrate checks ✅",);
            Ok(())
        }

        fn on_runtime_upgrade() -> Weight {
            migrate_nodes::<T>() + migrate_farms::<T>()
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade() -> Result<(), &'static str> {
            assert!(PalletVersion::<T>::get() == types::StorageVersion::V7Struct);

            info!(
                "👥  TFGrid pallet migration to {:?} passes POST migrate checks ✅",
                Pallet::<T>::pallet_version()
            );

            Ok(())
        }
    }
}

pub fn migrate<T: Config>() -> frame_support::weights::Weight {
    if PalletVersion::<T>::get() == types::StorageVersion::V5Struct {
        migrate_nodes::<T>() + migrate_farms::<T>()
    } else {
        info!(" >>> Unused migration");
        return 0;
    }
}

pub fn migrate_nodes<T: Config>() -> frame_support::weights::Weight {
    info!(" >>> Migrating nodes storage...");

    let mut migrated_count = 0;
    let mut writes = 0;
    let mut farms_with_nodes: BTreeMap<u32, Vec<u32>> = BTreeMap::new();

    // We transform the storage values from the old into the new format.
    Nodes::<T>::translate::<deprecated::NodeV4, _>(|k, node| {
        info!("     Migrated node for {:?}...", k);

        // By default initialize the public config to None
        let mut public_config = None;
        // If the node has a valid public config we can assign it
        if let Some(config) = &node.public_config {
            match get_public_config::<T>(&config) {
                Ok(config) => {
                    public_config = Some(config);
                }
                Err(e) => {
                    info!(
                        "failed to parse pub config for node: {:?}, error: {:?}",
                        k, e
                    );
                }
            }
        }

        let mut interfaces = Vec::new();
        match get_interfaces::<T>(&node) {
            Ok(intfs) => {
                interfaces = intfs;
            }
            Err(e) => {
                info!(
                    "failed to parse interfaces for node: {:?}, error: {:?}",
                    k, e
                );
            }
        }

        let new_node = Node {
            version: 5,
            id: node.id,
            farm_id: node.farm_id,
            twin_id: node.twin_id,
            resources: node.resources,
            location: node.location,
            country: node.country,
            city: node.city,
            public_config,
            created: node.created,
            farming_policy_id: node.farming_policy_id,
            interfaces,
            certification: node.certification,
            secure_boot: node.secure_boot,
            virtualized: node.virtualized,
            serial_number: node.serial_number,
            connection_price: node.connection_price,
        };

        // Add index of farm - list (nodes)
        farms_with_nodes
            .entry(node.farm_id)
            .or_insert(vec![])
            .push(node.id);

        migrated_count += 1;

        Some(new_node)
    });
    info!(
        " <<< Node storage updated! Migrated {} nodes ✅",
        migrated_count
    );

    for (farm_id, nodes) in farms_with_nodes.iter() {
        NodesByFarmID::<T>::insert(farm_id, nodes);
        writes += 1;
    }

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(
        migrated_count as Weight + 1,
        migrated_count + writes as Weight + 1,
    )
}

pub fn migrate_farms<T: Config>() -> frame_support::weights::Weight {
    info!(" >>> Migrating farms storage...");

    let mut migrated_count = 0;
    // We transform the storage values from the old into the new format.
    Farms::<T>::translate::<deprecated::FarmV3, _>(|k, farm| {
        info!("     Migrated farm for {:?}...", k);

        let mut public_ips: BoundedVec<PublicIpOf<T>, ConstU32<256>> = bounded_vec![];

        match get_public_ips::<T>(&farm) {
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

        let mut farm_name = farm.name;
        let truncated = farm_name.len() > <T as Config>::MaxFarmNameLength::get() as usize;
        if truncated {
            info!("farm name length exceeded, truncating farm name");
            farm_name.truncate(<T as Config>::MaxFarmNameLength::get() as usize);
        }

        let replaced_farm_name = farm::replace_farm_name_invalid_characters(&farm_name);
        let name = match <T as Config>::FarmName::try_from(replaced_farm_name.clone()) {
            Ok(n) => n,
            Err(_) => {
                info!("invalid farm name, skipping updating farm {:?} ...", k);
                return None;
            }
        };

        if replaced_farm_name != farm_name || truncated {
            info!("farm name changed, reworking storage");
            FarmIdByName::<T>::remove(&farm_name);
            FarmIdByName::<T>::insert(replaced_farm_name, farm.id);
        }

        let new_farm = Farm {
            version: 4,
            id: farm.id,
            name,
            twin_id: farm.twin_id,
            pricing_policy_id: farm.pricing_policy_id,
            certification: farm.certification,
            public_ips,
            dedicated_farm: farm.dedicated_farm,
            farming_policy_limits: farm.farming_policy_limits,
        };

        migrated_count += 1;

        Some(new_farm)
    });

    info!(
        " <<< Farm storage updated! Migrated {} Farms ✅",
        migrated_count
    );

    // Update pallet storage version
    PalletVersion::<T>::set(types::StorageVersion::V7Struct);
    info!(" <<< Storage version upgraded");

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(migrated_count as Weight + 1, migrated_count as Weight + 1)
}

fn get_public_config<T: Config>(
    config: &deprecated::PublicConfig,
) -> Result<PubConfigOf<T>, Error<T>> {
    let ipv4 = <T as Config>::IP4::try_from(config.ipv4.clone())?;
    let gw4 = <T as Config>::GW4::try_from(config.gw4.clone())?;

    let mut pub_config = PublicConfig {
        ip4: IP { ip: ipv4, gw: gw4 },
        ip6: None,
        domain: None,
    };

    if !config.ipv6.is_empty() && !config.gw6.is_empty() {
        let ipv6 = <T as Config>::IP6::try_from(config.ipv6.clone())?;
        let gw6 = <T as Config>::GW6::try_from(config.gw6.clone())?;

        pub_config.ip6 = Some(IP { ip: ipv6, gw: gw6 });
    }

    if !config.domain.is_empty() {
        let domain = <T as Config>::Domain::try_from(config.domain.clone())?;
        pub_config.domain = Some(domain)
    }

    Ok(pub_config)
}

fn get_interfaces<T: Config>(node: &deprecated::NodeV4) -> Result<Vec<InterfaceOf<T>>, Error<T>> {
    let mut parsed_interfaces = Vec::new();

    for intf in &node.interfaces {
        let intf_name = <T as Config>::InterfaceName::try_from(intf.name.clone())?;
        let intf_mac = <T as Config>::InterfaceMac::try_from(intf.mac.clone())?;

        let mut parsed_interfaces_ips: BoundedVec<
            InterfaceIp<T>,
            <T as Config>::MaxInterfaceIpsLength,
        > = bounded_vec![];

        for ip in &intf.ips {
            let intf_ip = <T as Config>::InterfaceIP::try_from(ip.clone())?;
            let _ = parsed_interfaces_ips.try_push(intf_ip);
        }

        parsed_interfaces.push(Interface {
            name: intf_name,
            mac: intf_mac,
            ips: parsed_interfaces_ips,
        });
    }

    Ok(parsed_interfaces)
}

fn get_public_ips<T: Config>(
    farm: &deprecated::FarmV3,
) -> Result<BoundedVec<PublicIpOf<T>, ConstU32<256>>, Error<T>> {
    let mut parsed_public_ips: BoundedVec<PublicIpOf<T>, ConstU32<256>> = bounded_vec![];

    for pub_ip in &farm.public_ips {
        let ip = <T as Config>::PublicIP::try_from(pub_ip.ip.clone())?;
        let gateway = <T as Config>::GatewayIP::try_from(pub_ip.gateway.clone())?;

        let _ = parsed_public_ips.try_push(PublicIP {
            ip,
            gateway,
            contract_id: pub_ip.contract_id,
        });
    }

    Ok(parsed_public_ips)
}
