use crate::{CityNameInput, Config, CountryNameInput, Error, LocationInput, SerialNumberInput};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    ensure, sp_runtime::SaturatedConversion, traits::ConstU32, BoundedVec, RuntimeDebug,
};
use scale_info::TypeInfo;
use sp_std::marker::PhantomData;

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
        let city_str = core::str::from_utf8(&value);
        ensure!(
            validate_city_name(&value) && city_str.is_ok(),
            Self::Error::InvalidCityName
        );

        Ok(Self(value, PhantomData))
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

pub fn validate_city_name(input: &[u8]) -> bool {
    input == DEFAULT_CITY_NAME
        || input
            .iter()
            .all(|c| c.is_ascii_alphabetic() || matches!(c, b'-' | b'.' | b' '))
}

// 4: Cuba, Fiji, Iran, ..
pub const MIN_COUNTRY_NAME_LENGTH: u32 = 4;
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
        let country_str = core::str::from_utf8(&value);
        ensure!(
            validate_country_name(&value) && country_str.is_ok(),
            Self::Error::InvalidCountryName
        );

        Ok(Self(value, PhantomData))
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

pub fn validate_country_name(input: &[u8]) -> bool {
    input == DEFAULT_COUNTRY_NAME
        || input
            .iter()
            .all(|c| c.is_ascii_alphabetic() || matches!(c, b'-' | b'.' | b' '))
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

pub const MIN_SERIAL_NUMBER_LENGTH: u32 = 10;
pub const MAX_SERIAL_NUMBER_LENGTH: u32 = 50;
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

    /// TODO: Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or ...
    fn try_from(value: SerialNumberInput) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_SERIAL_NUMBER_LENGTH.saturated_into(),
            Self::Error::SerialNumberTooShort
        );
        ensure!(
            value.len() <= MAX_SERIAL_NUMBER_LENGTH.saturated_into(),
            Self::Error::SerialNumberTooLong
        );
        ensure!(
            validate_serial_number(&value),
            Self::Error::InvalidSerialNumber
        );

        Ok(Self(value, PhantomData))
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

pub fn validate_serial_number(input: &[u8]) -> bool {
    input == DEFAULT_SERIAL_NUMBER
        || input
            .iter()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.'))
}
