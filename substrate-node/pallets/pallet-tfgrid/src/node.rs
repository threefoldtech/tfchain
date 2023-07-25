use crate::*;
use frame_support::{
    dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo, Pays},
    ensure,
    sp_runtime::SaturatedConversion,
    traits::ConstU32,
    BoundedVec, RuntimeDebug,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::Get;
use sp_std::marker::PhantomData;
use sp_std::{vec, vec::Vec};
use tfchain_support::{
    resources::Resources,
    traits::ChangeNode,
    types::{Interface, Node, NodeCertification, Power, PowerState, PublicConfig},
};

impl<T: Config> Pallet<T> {
    pub fn _create_node(
        account_id: &T::AccountId,
        farm_id: u32,
        resources: ResourcesInput,
        location: LocationInput,
        interfaces: InterfaceInput<T>,
        secure_boot: bool,
        virtualized: bool,
        serial_number: Option<SerialNumberInput>,
    ) -> DispatchResultWithPostInfo {
        ensure!(Farms::<T>::contains_key(farm_id), Error::<T>::FarmNotExists);
        ensure!(
            TwinIdByAccountID::<T>::contains_key(account_id),
            Error::<T>::TwinNotExists
        );
        let twin_id = TwinIdByAccountID::<T>::get(account_id).ok_or(Error::<T>::TwinNotExists)?;

        ensure!(
            !NodeIdByTwinID::<T>::contains_key(twin_id),
            Error::<T>::NodeWithTwinIdExists
        );

        let mut id = NodeID::<T>::get();
        id = id + 1;

        let node_resources = Self::get_resources(resources)?;
        let node_location = Self::get_location(location)?;
        let node_interfaces = Self::get_interfaces(&interfaces)?;

        let node_serial_number = if let Some(serial_input) = serial_number {
            Some(Self::get_serial_number(serial_input)?)
        } else {
            None
        };

        let created = <pallet_timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;

        let mut new_node = Node {
            version: TFGRID_NODE_VERSION,
            id,
            farm_id,
            twin_id,
            resources: node_resources,
            location: node_location,
            public_config: None,
            created,
            farming_policy_id: 0,
            interfaces: node_interfaces,
            certification: NodeCertification::default(),
            secure_boot,
            virtualized,
            serial_number: node_serial_number,
            connection_price: ConnectionPrice::<T>::get(),
        };

        let farming_policy = Self::get_farming_policy(&new_node)?;
        new_node.farming_policy_id = farming_policy.id;
        new_node.certification = farming_policy.node_certification;

        Nodes::<T>::insert(id, &new_node);
        NodeID::<T>::put(id);
        NodeIdByTwinID::<T>::insert(twin_id, new_node.id);

        let mut nodes_by_farm = NodesByFarmID::<T>::get(farm_id);
        nodes_by_farm.push(id);
        NodesByFarmID::<T>::insert(farm_id, nodes_by_farm);

        T::NodeChanged::node_changed(None, &new_node);

        Self::deposit_event(Event::NodeStored(new_node));

        Ok(().into())
    }

    pub fn _update_node(
        account_id: &T::AccountId,
        node_id: u32,
        farm_id: u32,
        resources: ResourcesInput,
        location: LocationInput,
        interfaces: InterfaceInput<T>,
        secure_boot: bool,
        virtualized: bool,
        serial_number: Option<SerialNumberInput>,
    ) -> DispatchResultWithPostInfo {
        let mut node = Nodes::<T>::get(&node_id).ok_or(Error::<T>::NodeNotExists)?;
        ensure!(
            TwinIdByAccountID::<T>::contains_key(account_id),
            Error::<T>::TwinNotExists
        );

        let twin_id = TwinIdByAccountID::<T>::get(account_id).ok_or(Error::<T>::TwinNotExists)?;
        ensure!(node.twin_id == twin_id, Error::<T>::NodeUpdateNotAuthorized);

        ensure!(Farms::<T>::contains_key(farm_id), Error::<T>::FarmNotExists);

        let old_node = Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;

        // If the farm ID changed on the node,
        // remove the node from the old map from the farm and insert into the correct one
        if old_node.farm_id != farm_id {
            let mut old_nodes_by_farm = NodesByFarmID::<T>::get(old_node.farm_id);
            old_nodes_by_farm.retain(|&id| id != node_id);
            NodesByFarmID::<T>::insert(old_node.farm_id, old_nodes_by_farm);

            let mut nodes_by_farm = NodesByFarmID::<T>::get(farm_id);
            nodes_by_farm.push(node_id);
            NodesByFarmID::<T>::insert(farm_id, nodes_by_farm);
        };

        let node_resources = Self::get_resources(resources)?;
        let node_location = Self::get_location(location)?;
        let node_interfaces = Self::get_interfaces(&interfaces)?;

        let node_serial_number = if let Some(serial_input) = serial_number {
            Some(Self::get_serial_number(serial_input)?)
        } else {
            None
        };

        // If the resources on a certified node changed, reset the certification level to DIY
        if Resources::has_changed(&node.resources, &node_resources, 1)
            && node.certification == NodeCertification::Certified
        {
            node.certification = NodeCertification::Diy;
            Self::deposit_event(Event::NodeCertificationSet(node_id, node.certification));
        }

        node.farm_id = farm_id;
        node.resources = node_resources;
        node.location = node_location;
        node.interfaces = node_interfaces;
        node.secure_boot = secure_boot;
        node.virtualized = virtualized;
        node.serial_number = node_serial_number;

        // override node in storage
        Nodes::<T>::insert(node.id, &node);

        T::NodeChanged::node_changed(Some(&old_node), &node);

        Self::deposit_event(Event::NodeUpdated(node));

        Ok(Pays::No.into())
    }

    pub fn _set_node_certification(
        node_id: u32,
        node_certification: NodeCertification,
    ) -> DispatchResultWithPostInfo {
        let mut node = Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;

        node.certification = node_certification;

        let current_node_policy = FarmingPoliciesMap::<T>::get(node.farming_policy_id);
        if current_node_policy.default {
            // Refetch farming policy and save it on the node
            let farming_policy = Self::get_farming_policy(&node)?;
            node.farming_policy_id = farming_policy.id;
        }

        // override node in storage
        Nodes::<T>::insert(node.id, &node);

        Self::deposit_event(Event::NodeUpdated(node));
        Self::deposit_event(Event::NodeCertificationSet(node_id, node_certification));

        Ok(().into())
    }

    pub fn _report_uptime(
        account_id: &T::AccountId,
        uptime: u64,
        timestamp_hint: u64,
    ) -> DispatchResultWithPostInfo {
        let twin_id = TwinIdByAccountID::<T>::get(account_id).ok_or(Error::<T>::TwinNotExists)?;

        ensure!(
            NodeIdByTwinID::<T>::contains_key(twin_id),
            Error::<T>::NodeNotExists
        );
        let node_id = NodeIdByTwinID::<T>::get(twin_id);

        ensure!(Nodes::<T>::contains_key(node_id), Error::<T>::NodeNotExists);

        let now = <pallet_timestamp::Pallet<T>>::get().saturated_into::<u64>() / 1000;
        // check if timestamp hint is within the acceptable range of the current timestamp (now) and the drift value
        ensure!(
            timestamp_hint
                >= now
                    .checked_sub(<T as Config>::TimestampHintDrift::get())
                    .unwrap_or(0)
                && timestamp_hint <= now + <T as Config>::TimestampHintDrift::get(),
            Error::<T>::InvalidTimestampHint
        );

        Self::deposit_event(Event::NodeUptimeReported(node_id, now, uptime));

        Ok(Pays::No.into())
    }

    pub fn _add_node_public_config(
        account_id: T::AccountId,
        farm_id: u32,
        node_id: u32,
        public_config: Option<PublicConfig>,
    ) -> DispatchResultWithPostInfo {
        // check if this twin can update the farm with id passed
        let farm = Farms::<T>::get(farm_id).ok_or(Error::<T>::FarmNotExists)?;

        ensure!(
            Twins::<T>::contains_key(farm.twin_id),
            Error::<T>::TwinNotExists
        );
        let farm_twin = Twins::<T>::get(farm.twin_id).ok_or(Error::<T>::TwinNotExists)?;
        ensure!(
            farm_twin.account_id == account_id,
            Error::<T>::CannotUpdateFarmWrongTwin
        );

        // check if the node belong to the farm
        let mut node = Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;
        ensure!(node.farm_id == farm_id, Error::<T>::NodeUpdateNotAuthorized);

        if let Some(config) = public_config {
            config
                .is_valid()
                .map_err(|_| Error::<T>::InvalidPublicConfig)?;
            // update the public config and save
            node.public_config = Some(config);
        } else {
            node.public_config = None;
        }

        Nodes::<T>::insert(node_id, &node);
        Self::deposit_event(Event::NodePublicConfigStored(node_id, node.public_config));

        Ok(().into())
    }

    pub fn _delete_node(account_id: &T::AccountId, node_id: u32) -> DispatchResultWithPostInfo {
        let node = Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;
        let twin_id = TwinIdByAccountID::<T>::get(account_id).ok_or(Error::<T>::TwinNotExists)?;
        ensure!(node.twin_id == twin_id, Error::<T>::NodeUpdateNotAuthorized);

        let mut nodes_by_farm = NodesByFarmID::<T>::get(node.farm_id);
        let location = nodes_by_farm
            .binary_search(&node_id)
            .or(Err(Error::<T>::NodeNotExists))?;
        nodes_by_farm.remove(location);
        NodesByFarmID::<T>::insert(node.farm_id, nodes_by_farm);

        // Call node deleted
        T::NodeChanged::node_deleted(&node);

        Nodes::<T>::remove(node_id);

        Self::deposit_event(Event::NodeDeleted(node_id));

        Ok(().into())
    }

    pub fn _add_node_certifier(certifier: T::AccountId) -> DispatchResultWithPostInfo {
        match AllowedNodeCertifiers::<T>::get() {
            Some(mut certifiers) => {
                let location = certifiers
                    .binary_search(&certifier)
                    .err()
                    .ok_or(Error::<T>::AlreadyCertifier)?;
                certifiers.insert(location, certifier.clone());
                AllowedNodeCertifiers::<T>::put(certifiers);

                Self::deposit_event(Event::NodeCertifierAdded(certifier));
            }
            None => {
                let certifiers = vec![certifier.clone()];
                AllowedNodeCertifiers::<T>::put(certifiers);
                Self::deposit_event(Event::NodeCertifierAdded(certifier));
            }
        }

        Ok(().into())
    }

    pub fn _remove_node_certifier(certifier: T::AccountId) -> DispatchResultWithPostInfo {
        if let Some(mut certifiers) = AllowedNodeCertifiers::<T>::get() {
            let location = certifiers
                .binary_search(&certifier)
                .ok()
                .ok_or(Error::<T>::NotCertifier)?;
            certifiers.remove(location);
            AllowedNodeCertifiers::<T>::put(&certifiers);

            Self::deposit_event(Event::NodeCertifierRemoved(certifier));
        }
        Ok(().into())
    }

    pub fn _change_power_state(
        account_id: &T::AccountId,
        power_state: Power,
    ) -> DispatchResultWithPostInfo {
        let twin_id = TwinIdByAccountID::<T>::get(account_id).ok_or(Error::<T>::TwinNotExists)?;
        ensure!(
            NodeIdByTwinID::<T>::contains_key(twin_id),
            Error::<T>::NodeNotExists
        );
        let node_id = NodeIdByTwinID::<T>::get(twin_id);
        let node = Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;

        let power_state = match power_state {
            Power::Up => PowerState::Up,
            Power::Down => PowerState::Down(frame_system::Pallet::<T>::block_number()),
        };

        let mut node_power = NodePower::<T>::get(node_id);

        // if the power state is not correct => change it and emit event
        if node_power.state != power_state {
            node_power.state = power_state.clone();

            NodePower::<T>::insert(node_id, node_power);
            Self::deposit_event(Event::PowerStateChanged {
                farm_id: node.farm_id,
                node_id,
                power_state,
            });
        }

        Ok(Pays::No.into())
    }

    pub fn _change_power_target(
        account_id: &T::AccountId,
        node_id: u32,
        power_target: Power,
    ) -> DispatchResultWithPostInfo {
        let twin_id = TwinIdByAccountID::<T>::get(account_id).ok_or(Error::<T>::TwinNotExists)?;
        let node = Nodes::<T>::get(node_id).ok_or(Error::<T>::NodeNotExists)?;
        let farm = Farms::<T>::get(node.farm_id).ok_or(Error::<T>::FarmNotExists)?;
        ensure!(
            twin_id == farm.twin_id,
            Error::<T>::UnauthorizedToChangePowerTarget
        );
        // Make sure only the farmer that owns this node can change the power target
        ensure!(
            node.farm_id == farm.id,
            Error::<T>::UnauthorizedToChangePowerTarget
        );

        Self::_change_power_target_on_node(node.id, node.farm_id, power_target);

        Ok(().into())
    }

    fn _change_power_target_on_node(node_id: u32, farm_id: u32, power_target: Power) {
        let mut node_power = NodePower::<T>::get(node_id);
        node_power.target = power_target.clone();
        NodePower::<T>::insert(node_id, &node_power);

        Self::deposit_event(Event::PowerTargetChanged {
            farm_id,
            node_id,
            power_target,
        });
    }

    fn get_resources(
        resources: pallet::ResourcesInput,
    ) -> Result<Resources, DispatchErrorWithPostInfo> {
        ensure!(resources.validate_hru(), Error::<T>::InvalidHRUInput);
        ensure!(resources.validate_sru(), Error::<T>::InvalidSRUInput);
        ensure!(resources.validate_cru(), Error::<T>::InvalidCRUInput);
        ensure!(resources.validate_mru(), Error::<T>::InvalidMRUInput);

        Ok(resources)
    }

    fn get_location(
        location: pallet::LocationInput,
    ) -> Result<LocationOf<T>, DispatchErrorWithPostInfo> {
        let parsed_location = <T as Config>::Location::try_from(location)?;
        Ok(parsed_location)
    }

    fn get_interfaces(
        interfaces: &InterfaceInput<T>,
    ) -> Result<Vec<InterfaceOf<T>>, DispatchErrorWithPostInfo> {
        let mut parsed_interfaces = Vec::new();
        if interfaces.len() == 0 {
            return Ok(parsed_interfaces);
        }

        for intf in interfaces.iter() {
            let intf_name = Self::get_interface_name(intf.name.clone())?;
            let intf_mac = Self::get_interface_mac(intf.mac.clone())?;

            let mut parsed_interfaces_ips: BoundedVec<
                InterfaceIpOf<T>,
                <T as Config>::MaxInterfaceIpsLength,
            > = vec![]
                .try_into()
                .map_err(|_| Error::<T>::InvalidInterfaceIP)?;

            for ip in intf.ips.iter() {
                let intf_ip = Self::get_interface_ip(ip.clone())?;
                parsed_interfaces_ips
                    .try_push(intf_ip)
                    .map_err(|_| Error::<T>::InvalidInterfaceIP)?;
            }

            parsed_interfaces.push(Interface {
                name: intf_name,
                mac: intf_mac,
                ips: parsed_interfaces_ips,
            });
        }

        Ok(parsed_interfaces)
    }

    fn get_interface_name(
        if_name: InterfaceNameInput,
    ) -> Result<InterfaceNameOf<T>, DispatchErrorWithPostInfo> {
        let if_name_parsed = <T as Config>::InterfaceName::try_from(if_name)?;
        Ok(if_name_parsed)
    }

    fn get_interface_mac(
        if_mac: InterfaceMacInput,
    ) -> Result<InterfaceMacOf<T>, DispatchErrorWithPostInfo> {
        let if_mac_parsed = <T as Config>::InterfaceMac::try_from(if_mac)?;
        Ok(if_mac_parsed)
    }

    fn get_interface_ip(
        if_ip: InterfaceIpInput,
    ) -> Result<InterfaceIpOf<T>, DispatchErrorWithPostInfo> {
        let if_ip_parsed = <T as Config>::InterfaceIP::try_from(if_ip)?;
        Ok(if_ip_parsed)
    }

    fn get_serial_number(
        serial_number: pallet::SerialNumberInput,
    ) -> Result<SerialNumberOf<T>, DispatchErrorWithPostInfo> {
        let parsed_serial_number = <T as Config>::SerialNumber::try_from(serial_number)?;
        Ok(parsed_serial_number)
    }
}

// 1: Y
pub const MIN_CITY_NAME_LENGTH: u32 = 1;
// 85: Llanfairpwllgwyngyllgogerychwyrndrobwllllantysiliogogogoch
pub const MAX_CITY_NAME_LENGTH: u32 = 58;
pub const DEFAULT_CITY_NAME: &[u8] = b"Unknown";

/// A city name in ASCI Characters.
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct CityName<T: Config>(
    pub BoundedVec<u8, ConstU32<MAX_CITY_NAME_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_CITY_NAME_LENGTH>)>,
);

impl<T: Config> TryFrom<CityNameInput> for CityName<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: CityNameInput) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_CITY_NAME_LENGTH.saturated_into(),
            Self::Error::CityNameTooShort
        );
        ensure!(
            value.len() <= MAX_CITY_NAME_LENGTH.saturated_into(),
            Self::Error::CityNameTooLong
        );

        Ok(Self(value, PhantomData))
    }
}

impl<T: Config> Default for CityName<T> {
    fn default() -> Self {
        let city: BoundedVec<u8, ConstU32<MAX_CITY_NAME_LENGTH>> =
            DEFAULT_CITY_NAME.to_vec().try_into().unwrap_or_default();

        Self(city, PhantomData)
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for CityName<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for CityName<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}

// 2: Allow country code like BE, FR, BR, ...
pub const MIN_COUNTRY_NAME_LENGTH: u32 = 2;
// 56: The United Kingdom of Great Britain and Northern Ireland
pub const MAX_COUNTRY_NAME_LENGTH: u32 = 56;
pub const DEFAULT_COUNTRY_NAME: &[u8] = b"Unknown";

/// A city name in ASCI Characters.
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct CountryName<T: Config>(
    pub BoundedVec<u8, ConstU32<MAX_COUNTRY_NAME_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_COUNTRY_NAME_LENGTH>)>,
);

impl<T: Config> TryFrom<CountryNameInput> for CountryName<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: CountryNameInput) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_COUNTRY_NAME_LENGTH.saturated_into(),
            Self::Error::CountryNameTooShort
        );
        ensure!(
            value.len() <= MAX_COUNTRY_NAME_LENGTH.saturated_into(),
            Self::Error::CountryNameTooLong
        );

        Ok(Self(value, PhantomData))
    }
}

impl<T: Config> Default for CountryName<T> {
    fn default() -> Self {
        let country: BoundedVec<u8, ConstU32<MAX_COUNTRY_NAME_LENGTH>> =
            DEFAULT_COUNTRY_NAME.to_vec().try_into().unwrap_or_default();

        Self(country, PhantomData)
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for CountryName<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for CountryName<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}

pub const MIN_LATITUDE_LENGTH: u32 = 1;
pub const MAX_LATITUDE_LENGTH: u32 = 50;
pub const DEFAULT_LATITUDE: &[u8] = b"Unknown";

pub const MIN_LONGITUDE_LENGTH: u32 = 1;
pub const MAX_LONGITUDE_LENGTH: u32 = 50;
pub const DEFAULT_LONGITUDE: &[u8] = b"Unknown";

/// A location that countains city, country and lat/long informations in ASCI Characters.
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct Location<T: Config> {
    pub city: CityName<T>,
    pub country: CountryName<T>,
    pub latitude: BoundedVec<u8, ConstU32<MAX_LATITUDE_LENGTH>>,
    pub longitude: BoundedVec<u8, ConstU32<MAX_LONGITUDE_LENGTH>>,
    _marker: PhantomData<T>,
}

impl<T: Config> TryFrom<LocationInput> for Location<T> {
    type Error = Error<T>;

    /// Fallible initialization from provided byte vectors
    /// (city, country, latitude and longitude) if one is below the
    /// minimum or exceeds the maximum allowed length.
    /// For city and country check if byte vector contains invalid
    /// ASCII characters. For lat/long check if byte vector can be
    /// converted to float and is inside [-90; 90] range (for latitude)
    /// or inside [-180; 180] range (for longitude)
    fn try_from(value: LocationInput) -> Result<Self, Self::Error> {
        // Check if [country][city] pair exists in data base
        let city = CityName::<T>::try_from(value.city)?;
        let country = CountryName::<T>::try_from(value.country)?;

        // latitude
        ensure!(
            value.latitude.len() >= MIN_LATITUDE_LENGTH.saturated_into(),
            Self::Error::LatitudeInputTooShort
        );
        ensure!(
            value.latitude.len() <= MAX_LATITUDE_LENGTH.saturated_into(),
            Self::Error::LatitudeInputTooLong
        );
        ensure!(
            validate_latitude_input(&value.latitude.to_vec()),
            Self::Error::InvalidLatitudeInput
        );

        // longitude
        ensure!(
            value.longitude.len() >= MIN_LONGITUDE_LENGTH.saturated_into(),
            Self::Error::LongitudeInputTooShort
        );
        ensure!(
            value.longitude.len() <= MAX_LONGITUDE_LENGTH.saturated_into(),
            Self::Error::LongitudeInputTooLong
        );
        ensure!(
            validate_longitude_input(&value.longitude.to_vec()),
            Self::Error::InvalidLongitudeInput
        );

        Ok(Self {
            city,
            country,
            latitude: value.latitude,
            longitude: value.longitude,
            _marker: PhantomData,
        })
    }
}

impl<T: Config> Default for Location<T> {
    fn default() -> Self {
        let city = CityName::default();
        let country = CountryName::default();
        let latitude: BoundedVec<u8, ConstU32<MAX_LATITUDE_LENGTH>> =
            DEFAULT_LATITUDE.to_vec().try_into().unwrap_or_default();
        let longitude: BoundedVec<u8, ConstU32<MAX_LONGITUDE_LENGTH>> =
            DEFAULT_LONGITUDE.to_vec().try_into().unwrap_or_default();

        Self {
            city,
            country,
            latitude,
            longitude,
            _marker: PhantomData,
        }
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for Location<T> {
    fn eq(&self, other: &Self) -> bool {
        self.city == other.city
            && self.country == other.country
            && self.latitude == other.latitude
            && self.longitude == other.longitude
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for Location<T> {
    fn clone(&self) -> Self {
        Self {
            city: self.city.clone(),
            country: self.country.clone(),
            latitude: self.latitude.clone(),
            longitude: self.longitude.clone(),
            _marker: PhantomData,
        }
    }
}

pub fn validate_latitude_input(input: &[u8]) -> bool {
    input == DEFAULT_LATITUDE
        || match core::str::from_utf8(input) {
            Ok(val) => {
                if let Some(lat) = val.parse::<f32>().ok() {
                    lat >= -90.0 && lat <= 90.0
                } else {
                    false
                }
            }
            Err(_) => false,
        }
}

pub fn validate_longitude_input(input: &[u8]) -> bool {
    input == DEFAULT_LONGITUDE
        || match core::str::from_utf8(input) {
            Ok(val) => {
                if let Some(long) = val.parse::<f32>().ok() {
                    long >= -180.0 && long <= 180.0
                } else {
                    false
                }
            }
            Err(_) => false,
        }
}

pub const MAX_SERIAL_NUMBER_LENGTH: u32 = 128;
pub const DEFAULT_SERIAL_NUMBER: &[u8] = b"Not Specified";

/// A serial number in ASCI Characters.
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct SerialNumber<T: Config>(
    pub BoundedVec<u8, ConstU32<MAX_SERIAL_NUMBER_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_SERIAL_NUMBER_LENGTH>)>,
);

impl<T: Config> TryFrom<SerialNumberInput> for SerialNumber<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: SerialNumberInput) -> Result<Self, Self::Error> {
        ensure!(
            value.len() <= MAX_SERIAL_NUMBER_LENGTH.saturated_into(),
            Self::Error::SerialNumberTooLong
        );

        Ok(Self(value, PhantomData))
    }
}

impl<T: Config> Default for SerialNumber<T> {
    fn default() -> Self {
        let serial: BoundedVec<u8, ConstU32<MAX_SERIAL_NUMBER_LENGTH>> = DEFAULT_SERIAL_NUMBER
            .to_vec()
            .try_into()
            .unwrap_or_default();

        Self(serial, PhantomData)
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for SerialNumber<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for SerialNumber<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}

#[test]
fn test_validate_latitude_input_works() {
    assert_eq!(validate_latitude_input(b"90.0"), true);
    assert_eq!(validate_latitude_input(b"-90.0"), true);
    assert_eq!(validate_latitude_input(b"0.0"), true);

    assert_eq!(validate_latitude_input(b"90.00001"), false); // 10e-5 sensitive
    assert_eq!(validate_latitude_input(b"-90.00001"), false); // 10e-5 sensitive
    assert_eq!(validate_longitude_input(b"30,35465"), false);
    assert_eq!(validate_latitude_input(b"garbage data"), false);
}

#[test]
fn test_validate_longitude_input_works() {
    assert_eq!(validate_longitude_input(b"180.0"), true);
    assert_eq!(validate_longitude_input(b"-180.0"), true);
    assert_eq!(validate_longitude_input(b"0.0"), true);

    assert_eq!(validate_longitude_input(b"180.00001"), false); // 10e-5 sensitive
    assert_eq!(validate_longitude_input(b"-180.00001"), false); // 10e-5 sensitive
    assert_eq!(validate_longitude_input(b"30,35465"), false);
    assert_eq!(validate_longitude_input(b"garbage data"), false);
}
