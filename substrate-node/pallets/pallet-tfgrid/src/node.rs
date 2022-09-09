use crate::{Config, Error};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    ensure, sp_runtime::SaturatedConversion, traits::ConstU32, BoundedVec, RuntimeDebug,
};
use scale_info::TypeInfo;
use sp_std::marker::PhantomData;
use sp_std::vec::Vec;

pub const MIN_LATITUDE_LENGTH: u32 = 1;
pub const MAX_LATITUDE_LENGTH: u32 = 50;

pub const MIN_LONGITUDE_LENGTH: u32 = 1;
pub const MAX_LONGITUDE_LENGTH: u32 = 50;

// 4: Cuba, Fiji, Iran, ..
pub const MIN_COUNTRY_NAME_LENGTH: u32 = 4;
// 56: The United Kingdom of Great Britain and Northern Ireland
pub const MAX_COUNTRY_NAME_LENGTH: u32 = 56;

// 1: Y
pub const MIN_CITY_NAME_LENGTH: u32 = 1;
// 85: Llanfairpwllgwyngyllgogerychwyrndrobwllllantysiliogogogoch
pub const MAX_CITY_NAME_LENGTH: u32 = 58;

/// A location that countains city, country and lat/long informations in ASCI Characters.
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct Location<T: Config> {
    pub(crate) city: BoundedVec<u8, ConstU32<MAX_CITY_NAME_LENGTH>>,
    pub(crate) country: BoundedVec<u8, ConstU32<MAX_COUNTRY_NAME_LENGTH>>,
    pub(crate) latitude: BoundedVec<u8, ConstU32<MAX_LATITUDE_LENGTH>>,
    pub(crate) longitude: BoundedVec<u8, ConstU32<MAX_LONGITUDE_LENGTH>>,
    _marker: PhantomData<T>,
}

impl<T: Config> TryFrom<(Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)> for Location<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided tuple of byte vector
    /// (city, country, latitude and longitude) if one is below the
    /// minimum or exceeds the maximum allowed length.
    /// For city and country check if byte vector contains invalid
    /// ASCII characters. For lat/long check if byte vector can be
    /// converted to float and is inside [-90; 90] range (for latitude)
    /// or inside [-180; 180] range (for longitude)
    fn try_from(value: (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)) -> Result<Self, Self::Error> {
        // 1. city name
        ensure!(
            value.0.len() >= MIN_CITY_NAME_LENGTH.saturated_into(),
            Self::Error::NodeCityNameTooShort
        );
        let city: BoundedVec<u8, ConstU32<MAX_CITY_NAME_LENGTH>> =
            BoundedVec::try_from(value.0).map_err(|_| Self::Error::NodeCityNameTooLong)?;
        ensure!(validate_city_name(&city), Self::Error::InvalidNodeCityName);

        // 2. country name
        ensure!(
            value.1.len() >= MIN_COUNTRY_NAME_LENGTH.saturated_into(),
            Self::Error::NodeCountryNameTooShort
        );
        let country: BoundedVec<u8, ConstU32<MAX_COUNTRY_NAME_LENGTH>> =
            BoundedVec::try_from(value.1).map_err(|_| Self::Error::NodeCountryNameTooLong)?;
        ensure!(
            validate_country_name(&country),
            Self::Error::InvalidNodeCountryName
        );

        // 3. latitude
        ensure!(
            value.2.len() >= MIN_LATITUDE_LENGTH.saturated_into(),
            Self::Error::NodeLatitudeInputToShort
        );
        let latitude: BoundedVec<u8, ConstU32<MAX_LATITUDE_LENGTH>> =
            BoundedVec::try_from(value.2.clone())
                .map_err(|_| Self::Error::NodeLatitudeInputToLong)?;
        ensure!(
            validate_latitude_input(&latitude),
            Self::Error::InvalidNodeLatitudeInput
        );

        // 4. latitude
        ensure!(
            value.3.len() >= MIN_LONGITUDE_LENGTH.saturated_into(),
            Self::Error::NodeLongitudeInputToShort
        );
        let longitude: BoundedVec<u8, ConstU32<MAX_LONGITUDE_LENGTH>> =
            BoundedVec::try_from(value.3.clone())
                .map_err(|_| Self::Error::NodeLongitudeInputToLong)?;
        ensure!(
            validate_longitude_input(&longitude),
            Self::Error::InvalidNodeLongitudeInput
        );

        Ok(Self {
            city,
            country,
            latitude,
            longitude,
            _marker: PhantomData,
        })
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

fn validate_city_name(input: &[u8]) -> bool {
    // TODO: find better alternative
    input
        .iter()
        .all(|c| c.is_ascii_alphabetic() || c.is_ascii_whitespace())
}

fn validate_country_name(input: &[u8]) -> bool {
    // TODO: find better alternative
    input
        .iter()
        .all(|c| c.is_ascii_alphabetic() || c.is_ascii_whitespace())
}

fn validate_latitude_input(input: &[u8]) -> bool {
    match core::str::from_utf8(input) {
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

fn validate_longitude_input(input: &[u8]) -> bool {
    match core::str::from_utf8(input) {
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
