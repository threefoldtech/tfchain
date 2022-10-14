use super::Config;
use super::InterfaceOf;
use super::PubConfigOf;
use super::*;
use frame_support::{traits::Get, weights::Weight, BoundedVec};
use log::info;

pub mod deprecated {
    use codec::{Decode, Encode};
    use core::cmp::{Ord, PartialOrd};
    use scale_info::TypeInfo;
    use sp_std::prelude::*;
    use sp_std::vec::Vec;
    use tfchain_support::{resources::Resources, types::NodeCertification};

    #[derive(Encode, Decode, Debug, Default, PartialEq, Eq, Clone, TypeInfo)]
    pub struct Entity<AccountId> {
        pub version: u32,
        pub id: u32,
        pub name: Vec<u8>,
        pub account_id: AccountId,
        pub country: Vec<u8>,
        pub city: Vec<u8>,
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
    pub struct Node<PubConfig, If> {
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

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, TypeInfo)]
    pub struct Location {
        pub longitude: Vec<u8>,
        pub latitude: Vec<u8>,
    }
}

pub mod v10 {
    use super::*;
    use crate::Config;

    use frame_support::{pallet_prelude::Weight, traits::OnRuntimeUpgrade};
    use sp_std::marker::PhantomData;
    pub struct GridMigration<T: Config>(PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for GridMigration<T> {
        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<(), &'static str> {
            assert!(PalletVersion::<T>::get() == types::StorageVersion::V9Struct);

            info!("👥  TFGrid pallet to v10 passes PRE migrate checks ✅",);
            Ok(())
        }

        fn on_runtime_upgrade() -> Weight {
            migrate::<T>()
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade() -> Result<(), &'static str> {
            assert!(PalletVersion::<T>::get() == types::StorageVersion::V10Struct);

            info!(
                "👥  TFGrid pallet migration to {:?} passes POST migrate checks ✅",
                Pallet::<T>::pallet_version()
            );

            Ok(())
        }
    }
}

fn migrate<T: Config>() -> frame_support::weights::Weight {
    if PalletVersion::<T>::get() == types::StorageVersion::V9Struct {
        migrate_entities::<T>() + migrate_nodes::<T>() + update_pallet_storage_version::<T>()
    } else {
        info!(" >>> Unused migration");
        0
    }
}

fn migrate_entities<T: Config>() -> frame_support::weights::Weight {
    info!(" >>> Migrating entities storage...");

    let mut migrated_count = 0;

    // We transform the storage values from the old into the new format.
    Entities::<T>::translate::<deprecated::Entity<AccountIdOf<T>>, _>(|k, entity| {
        info!("     Migrated entity for {:?}...", k);

        let country_name_input: CountryNameInput = BoundedVec::try_from(entity.country).unwrap();
        let country = match <T as Config>::CountryName::try_from(country_name_input) {
            Ok(country_name) => country_name,
            Err(e) => {
                info!(
                    "failed to parse country name for entity: {:?}, error: {:?}",
                    k, e
                );
                let default_country_name_input: CountryNameInput =
                    BoundedVec::try_from(node::DEFAULT_COUNTRY_NAME.to_vec()).unwrap();
                info!("set default country name for entity");
                <T as Config>::CountryName::try_from(default_country_name_input).unwrap()
            }
        };

        let city_name_input: CityNameInput = BoundedVec::try_from(entity.city).unwrap();
        let city = match <T as Config>::CityName::try_from(city_name_input) {
            Ok(city_name) => city_name,
            Err(e) => {
                info!(
                    "failed to parse city name for entity: {:?}, error: {:?}",
                    k, e
                );
                let default_city_name_input: CityNameInput =
                    BoundedVec::try_from(node::DEFAULT_CITY_NAME.to_vec()).unwrap();

                info!("set default city name for entity");
                <T as Config>::CityName::try_from(default_city_name_input).unwrap()
            }
        };

        let new_entity = TfgridEntity::<T> {
            version: 2, // ??
            id: entity.id,
            name: entity.name,
            account_id: entity.account_id,
            country,
            city,
        };

        migrated_count += 1;

        Some(new_entity)
    });

    info!(
        " <<< Entity storage updated! Migrated {} Entities ✅",
        migrated_count
    );

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(migrated_count as Weight + 1, migrated_count as Weight + 1)
}

fn migrate_nodes<T: Config>() -> frame_support::weights::Weight {
    info!(" >>> Migrating nodes storage...");

    let mut migrated_count = 0;

    // We transform the storage values from the old into the new format.
    Nodes::<T>::translate::<deprecated::Node<PubConfigOf<T>, InterfaceOf<T>>, _>(|k, node| {
        info!("     Migrated node for {:?}...", k);

        let location = match get_location::<T>(&node) {
            Ok(loc) => loc,
            Err(e) => {
                info!(
                    "location: [city: {:?}, country: {:?}, latitude: {:?}, longitude: {:?}]",
                    core::str::from_utf8(&node.city).unwrap(),
                    core::str::from_utf8(&node.country).unwrap(),
                    core::str::from_utf8(&node.location.latitude).unwrap(),
                    core::str::from_utf8(&node.location.longitude).unwrap()
                );
                info!("failed to parse location for node: {:?}, error: {:?}", k, e);
                let default_location_input = LocationInput {
                    city: BoundedVec::try_from(node::DEFAULT_CITY_NAME.to_vec()).unwrap(),
                    country: BoundedVec::try_from(node::DEFAULT_COUNTRY_NAME.to_vec()).unwrap(),
                    latitude: BoundedVec::try_from(node::DEFAULT_LATITUDE.to_vec()).unwrap(),
                    longitude: BoundedVec::try_from(node::DEFAULT_LONGITUDE.to_vec()).unwrap(),
                };
                info!("set default location for node");
                <T as Config>::Location::try_from(default_location_input).unwrap()
            }
        };

        let serial_number_input: SerialNumberInput =
            BoundedVec::try_from(node.serial_number.clone()).unwrap();
        let serial_number = match <T as Config>::SerialNumber::try_from(serial_number_input) {
            Ok(serial) => serial,
            Err(e) => {
                info!(
                    "serial number: {:?}",
                    core::str::from_utf8(&node.serial_number).unwrap()
                );
                info!(
                    "failed to parse serial number for node: {:?}, error: {:?}",
                    k, e
                );
                let default_serial_number_input: SerialNumberInput =
                    BoundedVec::try_from(node::DEFAULT_SERIAL_NUMBER.to_vec()).unwrap();
                info!("set default serial number for node");
                <T as Config>::SerialNumber::try_from(default_serial_number_input).unwrap()
            }
        };

        let new_node = TfgridNode::<T> {
            version: 6, // ??
            id: node.id,
            farm_id: node.farm_id,
            twin_id: node.twin_id,
            resources: node.resources,
            location,
            public_config: node.public_config,
            created: node.created,
            farming_policy_id: node.farming_policy_id,
            interfaces: node.interfaces,
            certification: node.certification,
            secure_boot: node.secure_boot,
            virtualized: node.virtualized,
            serial_number,
            connection_price: node.connection_price,
        };

        migrated_count += 1;

        Some(new_node)
    });
    info!(
        " <<< Node storage updated! Migrated {} nodes ✅",
        migrated_count
    );

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(migrated_count as Weight + 1, migrated_count as Weight + 1)
}

fn update_pallet_storage_version<T: Config>() -> frame_support::weights::Weight {
    PalletVersion::<T>::set(types::StorageVersion::V10Struct);
    info!(" <<< Storage version upgraded");

    // Return the weight consumed by the migration.
    T::DbWeight::get().writes(1)
}

fn get_location<T: Config>(
    node: &deprecated::Node<PubConfigOf<T>, InterfaceOf<T>>,
) -> Result<LocationOf<T>, Error<T>> {
    let location_input = LocationInput {
        city: BoundedVec::try_from(node.city.clone()).unwrap(),
        country: BoundedVec::try_from(node.country.clone()).unwrap(),
        latitude: BoundedVec::try_from(node.location.latitude.clone()).unwrap(),
        longitude: BoundedVec::try_from(node.location.longitude.clone()).unwrap(),
    };

    <T as Config>::Location::try_from(location_input)
}
