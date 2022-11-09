use super::*;
use frame_support::{traits::Get, weights::Weight};
use log::info;
use tfchain_support::types::{ConsumableResources, Power, PowerState, PowerTarget, Resources};

pub mod deprecated {
    use crate::Config;
    use codec::{Decode, Encode};
    use frame_support::decl_module;

    use scale_info::TypeInfo;
    use sp_std::prelude::*;

    use tfchain_support::types::{Location, NodeCertification, Resources};

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
    pub struct NodeV9Struct<PubConfig, If> {
        pub version: u32,
        pub id: u32,
        pub farm_id: u32,
        pub twin_id: u32,
        pub resources: Resources,
        pub location: Location,
        pub country: Vec<u8>,
        pub city: Vec<u8>,
        // optional public config
        pub public_config: Option<PubConfig>,
        pub created: u64,
        pub farming_policy_id: u32,
        pub interfaces: Vec<If>,
        pub certification: NodeCertification,
        pub secure_boot: bool,
        pub virtualized: bool,
        pub serial_number: Vec<u8>,
        pub connection_price: u32,
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::Origin { }
    }
}

pub mod v9 {
    use super::*;
    use crate::Config;

    use frame_support::{pallet_prelude::Weight, traits::OnRuntimeUpgrade};
    use sp_std::marker::PhantomData;
    pub struct NodeMigration<T: Config>(PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for NodeMigration<T> {
        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<(), &'static str> {
            info!("current pallet version: {:?}", PalletVersion::<T>::get());
            assert!(PalletVersion::<T>::get() == types::StorageVersion::V9Struct);

            info!("👥  Tfgrid pallet to V10 passes PRE migrate checks ✅");
            Ok(())
        }

        fn on_runtime_upgrade() -> Weight {
            migrate_to_version_10::<T>()
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade() -> Result<(), &'static str> {
            info!("current pallet version: {:?}", PalletVersion::<T>::get());
            assert!(PalletVersion::<T>::get() == types::StorageVersion::V10Struct);

            info!(
                "👥  Tfgrid pallet to {:?} passes POST migrate checks ✅",
                PalletVersion::<T>::get()
            );

            Ok(())
        }
    }
}

pub fn migrate_to_version_10<T: Config>() -> frame_support::weights::Weight {
    if PalletVersion::<T>::get() == types::StorageVersion::V9Struct {
        info!(
            " >>> Starting tfgrid pallet migration, pallet version: {:?}",
            PalletVersion::<T>::get()
        );

        let mut migrated_count = 0;

        Nodes::<T>::translate::<deprecated::NodeV9Struct<PubConfigOf<T>, InterfaceOf<T>>, _>(
            |k, n| {
                migrated_count += 1;
                let migrated_contract = Node {
                    version: TFGRID_NODE_VERSION,
                    id: n.id,
                    farm_id: n.farm_id,
                    twin_id: n.twin_id,
                    resources: ConsumableResources {
                        total_resources: n.resources,
                        used_resources: Resources::empty(),
                    },
                    location: n.location,
                    country: n.country,
                    city: n.city,
                    power: Power {
                        target: PowerTarget::Up,
                        state: PowerState::Up,
                        last_uptime: 0,
                    },
                    // optional public config
                    public_config: n.public_config,
                    created: n.created,
                    farming_policy_id: n.farming_policy_id,
                    interfaces: n.interfaces,
                    certification: n.certification,
                    secure_boot: n.secure_boot,
                    virtualized: n.virtualized,
                    serial_number: n.serial_number,
                    connection_price: n.connection_price,
                };
                info!("Node: {:?} succesfully migrated", k);
                Some(migrated_contract)
            }
        );

        info!(
            " <<< Node storage updated! Migrated {} Nodes ✅",
            migrated_count
        );

        // Update pallet storage version
        PalletVersion::<T>::set(types::StorageVersion::V10Struct);
        info!(" <<< Storage version upgraded");

        // Return the weight consumed by the migration.
        T::DbWeight::get().reads_writes(migrated_count as Weight + 1, migrated_count as Weight + 1)
    } else {
        info!(" >>> Unused migration");
        return 0;
    }
}
