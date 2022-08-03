use super::Config;
use super::*;
use frame_support::{traits::Get, weights::Weight};
use log::info;
use tfchain_support::types::{Interface, PublicConfig};

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
    pub struct Farm {
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
        pub public_config: PublicConfig,
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

pub mod v4 {
    use super::*;
    use crate::Config;

    // #[cfg(feature = "try-runtime")]
    // use frame_support::traits::GetStorageVersion;

    use frame_support::{pallet_prelude::Weight, traits::OnRuntimeUpgrade};
    use log::info;
    use sp_std::marker::PhantomData;

    pub struct ContractMigrationV4<T: Config>(PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for ContractMigrationV4<T> {
        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<(), &'static str> {
            assert!(PalletVersion::<T>::get() == types::StorageVersion::V5Struct);

            info!("👥  TFGrid pallet to v4 passes PRE migrate checks ✅",);
            Ok(())
        }

        fn on_runtime_upgrade() -> Weight {
            migrate_nodes::<T>()
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade() -> Result<(), &'static str> {
            assert!(PalletVersion::<T>::get() == types::StorageVersion::V6Struct);

            info!(
                "👥  TFGrid pallet migration to {:?} passes POST migrate checks ✅",
                Pallet::<T>::pallet_storage_version()
            );
            Ok(())
        }
    }
}

// use super::pallet::PubConfigOf;
pub fn migrate_nodes<T: Config>() -> frame_support::weights::Weight {
    info!(" >>> Starting migration, pallet version",);
    let count = Nodes::<T>::iter().count();
    info!(" >>> Updating Nodes storage. Migrating {} nodes...", count);

    let mut migrated_count = 0;
    // We transform the storage values from the old into the new format.
    Nodes::<T>::translate::<deprecated::NodeV4, _>(|k, node| {
        info!("     Migrated node for {:?}...", k);

        // By default initialize the public config to None
        let mut public_config = None;
        // If the node has a valid public config we can assign it
        if let Ok(config) = get_public_config::<T>(&node) {
            public_config = Some(config);
        };

        let mut interfaces = Vec::new();
        if let Ok(intfs) = get_interfaces::<T>(&node) {
            interfaces = intfs;
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

        migrated_count += 1;

        Some(new_node)
    });
    info!(
        " <<< Node storage updated! Migrated {} nodes ✅",
        migrated_count
    );

    // Update pallet storage version
    PalletVersion::<T>::set(types::StorageVersion::V6Struct);
    info!(" <<< Storage version upgraded");

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(migrated_count as Weight + 1, migrated_count as Weight + 1)
}

use super::PubConfigOf;
fn get_public_config<T: Config>(node: &deprecated::NodeV4) -> Result<PubConfigOf<T>, Error<T>> {
    let ipv4 = <T as Config>::IP4::try_from(node.public_config.ipv4.clone())?;
    let gw4 = <T as Config>::GW4::try_from(node.public_config.gw4.clone())?;
    let ipv6 = <T as Config>::IP6::try_from(node.public_config.ipv6.clone())?;
    let gw6 = <T as Config>::GW6::try_from(node.public_config.gw6.clone())?;
    let domain = <T as Config>::Domain::try_from(node.public_config.domain.clone())?;

    Ok(PublicConfig {
        ipv4,
        gw4,
        ipv6,
        gw6,
        domain,
    })
}

use super::{InterfaceIp, InterfaceOf};
use frame_support::BoundedVec;
fn get_interfaces<T: Config>(node: &deprecated::NodeV4) -> Result<Vec<InterfaceOf<T>>, Error<T>> {
    let mut parsed_interfaces = Vec::new();

    for intf in &node.interfaces {
        let intf_name = <T as Config>::InterfaceName::try_from(intf.name.clone())?;
        let intf_mac = <T as Config>::InterfaceMac::try_from(intf.mac.clone())?;

        let mut parsed_interfaces_ips: BoundedVec<
            InterfaceIp<T>,
            <T as Config>::MaxInterfaceIpsLength,
        > = vec![].try_into().unwrap();

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
