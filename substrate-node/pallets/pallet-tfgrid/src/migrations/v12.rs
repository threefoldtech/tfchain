use crate::Config;
use crate::*;
use crate::{InterfaceOf, LocationOf, Pallet, SerialNumberOf};
use frame_support::{
    pallet_prelude::OptionQuery, pallet_prelude::Weight, storage_alias, traits::Get,
    traits::OnRuntimeUpgrade, Blake2_128Concat, BoundedVec,
};
use log::{debug, info};
use sp_std::marker::PhantomData;

#[cfg(feature = "try-runtime")]
use frame_support::traits::OnRuntimeUpgradeHelpersExt;
#[cfg(feature = "try-runtime")]
use sp_runtime::SaturatedConversion;

// Storage alias from Node v12
#[storage_alias]
pub type Nodes<T: Config> = StorageMap<
    Pallet<T>,
    Blake2_128Concat,
    u32,
    super::types::v12::Node<LocationOf<T>, InterfaceOf<T>, SerialNumberOf<T>>,
    OptionQuery,
>;

// Storage alias from Entity v12
#[storage_alias]
pub type Entities<T: Config> = StorageMap<
    Pallet<T>,
    Blake2_128Concat,
    u32,
    super::types::v12::Entity<AccountIdOf<T>, CityNameOf<T>, CountryNameOf<T>>,
    OptionQuery,
>;

pub struct InputValidation<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for InputValidation<T> {
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V11Struct);

        // Store number of nodes in temp storage
        let nodes_count: u64 = Nodes::<T>::iter_keys().count().saturated_into();
        Self::set_temp_storage(nodes_count, "pre_nodes_count");
        log::info!(
            "🔎 NodeMigrationV12 pre migration: Number of existing nodes {:?}",
            nodes_count
        );

        info!("👥  TFGrid pallet to V12 passes PRE migrate checks ✅",);
        Ok(())
    }

    fn on_runtime_upgrade() -> Weight {
        if PalletVersion::<T>::get() == types::StorageVersion::V11Struct {
            migrate_entities::<T>() + migrate_nodes::<T>() + update_pallet_storage_version::<T>()
        } else {
            info!(" >>> Unused TFGrid pallet V12 migration");
            Weight::zero()
        }
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        assert!(PalletVersion::<T>::get() >= types::StorageVersion::V12Struct);

        // Check number of nodes against pre-check result
        let pre_nodes_count = Self::get_temp_storage("pre_nodes_count").unwrap_or(0u64);
        assert_eq!(
            Nodes::<T>::iter_keys().count().saturated_into::<u64>(),
            pre_nodes_count,
            "Number of nodes migrated does not match"
        );

        info!(
            "👥  TFGrid pallet to {:?} passes POST migrate checks ✅",
            Pallet::<T>::pallet_version()
        );

        Ok(())
    }
}

fn migrate_entities<T: Config>() -> frame_support::weights::Weight {
    info!(" >>> Migrating entities storage...");

    let mut migrated_count = 0;

    // We transform the storage values from the old into the new format.
    Entities::<T>::translate::<super::types::v11::Entity<AccountIdOf<T>>, _>(|k, entity| {
        let country = match get_country_name::<T>(&entity) {
            Ok(country_name) => country_name,
            Err(e) => {
                info!(
                    "failed to parse country name for entity: {:?}, error: {:?}",
                    k, e
                );
                info!("set default country name for entity");
                <T as Config>::CountryName::default()
            }
        };

        let city = match get_city_name::<T>(&entity) {
            Ok(city_name) => city_name,
            Err(e) => {
                info!(
                    "failed to parse city name for entity: {:?}, error: {:?}",
                    k, e
                );
                info!("set default city name for entity");
                <T as Config>::CityName::default()
            }
        };

        let new_entity =
            super::types::v12::Entity::<AccountIdOf<T>, CityNameOf<T>, CountryNameOf<T>> {
                version: TFGRID_ENTITY_VERSION,
                id: entity.id,
                name: entity.name,
                account_id: entity.account_id,
                country,
                city,
            };

        migrated_count += 1;

        debug!("Entity: {:?} succesfully migrated", k);
        Some(new_entity)
    });

    info!(
        " <<< Entity storage updated! Migrated {} Entities ✅",
        migrated_count
    );

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(migrated_count + 1, migrated_count + 1)
}

fn migrate_nodes<T: Config>() -> frame_support::weights::Weight {
    info!(" >>> Migrating nodes storage...");

    let mut migrated_count = 0;

    // We transform the storage values from the old into the new format.
    Nodes::<T>::translate::<super::types::v11::Node<InterfaceOf<T>>, _>(|k, node| {
        let location = match get_location::<T>(&node) {
            Ok(loc) => loc,
            Err(e) => {
                info!("failed to parse location for node: {:?}, error: {:?}", k, e);
                info!("set default location for node");
                <T as Config>::Location::default()
            }
        };

        let serial_number = match get_serial_number::<T>(&node) {
            Ok(serial) => Some(serial),
            Err(_) => None,
        };

        let new_node = super::types::v12::Node::<LocationOf<T>, InterfaceOf<T>, SerialNumberOf<T>> {
            version: TFGRID_NODE_VERSION,
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

        debug!("Node: {:?} succesfully migrated", k);
        Some(new_node)
    });
    info!(
        " <<< Node storage updated! Migrated {} nodes ✅",
        migrated_count
    );

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(migrated_count + 1, migrated_count + 1)
}

fn update_pallet_storage_version<T: Config>() -> frame_support::weights::Weight {
    PalletVersion::<T>::set(types::StorageVersion::V12Struct);
    info!(
        " <<< Storage version TFGrid pallet upgraded to {:?}",
        PalletVersion::<T>::get()
    );

    // Return the weight consumed by the migration.
    T::DbWeight::get().writes(1)
}

fn get_country_name<T: Config>(
    node: &super::types::v11::Entity<AccountIdOf<T>>,
) -> Result<CountryNameOf<T>, Error<T>> {
    let country_name_input: CountryNameInput =
        BoundedVec::try_from(node.country.clone()).map_err(|_| Error::<T>::CountryNameTooLong)?;

    <T as Config>::CountryName::try_from(country_name_input)
}

fn get_city_name<T: Config>(
    node: &super::types::v11::Entity<AccountIdOf<T>>,
) -> Result<CityNameOf<T>, Error<T>> {
    let city_name_input: CityNameInput =
        BoundedVec::try_from(node.city.clone()).map_err(|_| Error::<T>::CityNameTooLong)?;

    <T as Config>::CityName::try_from(city_name_input)
}

fn get_location<T: Config>(
    node: &super::types::v11::Node<InterfaceOf<T>>,
) -> Result<LocationOf<T>, Error<T>> {
    let location_input = LocationInput {
        city: BoundedVec::try_from(node.city.clone()).map_err(|_| Error::<T>::CityNameTooLong)?,
        country: BoundedVec::try_from(node.country.clone())
            .map_err(|_| Error::<T>::CountryNameTooLong)?,
        latitude: BoundedVec::try_from(node.location.latitude.clone())
            .map_err(|_| Error::<T>::LatitudeInputTooLong)?,
        longitude: BoundedVec::try_from(node.location.longitude.clone())
            .map_err(|_| Error::<T>::LongitudeInputTooLong)?,
    };

    <T as Config>::Location::try_from(location_input)
}

fn get_serial_number<T: Config>(
    node: &super::types::v11::Node<InterfaceOf<T>>,
) -> Result<SerialNumberOf<T>, Error<T>> {
    let serial_number_input: SerialNumberInput = BoundedVec::try_from(node.serial_number.clone())
        .map_err(|_| Error::<T>::SerialNumberTooLong)?;

    <T as Config>::SerialNumber::try_from(serial_number_input)
}
