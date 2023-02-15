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
        ensure!(validate_city_name(&value), Self::Error::InvalidCityName);

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

pub fn validate_city_name(input: &[u8]) -> bool {
    input == DEFAULT_CITY_NAME
        || match core::str::from_utf8(input) {
            Ok(val) => val
                .chars()
                .all(|c| c.is_alphabetic() || matches!(c, '-' | '.' | ' ')),
            Err(_) => false,
        }
    // we convert to &str and then to chars to handle
    // special alphabetic characters thanks to is_alphabetic()
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
        ensure!(
            validate_country_name(&value),
            Self::Error::InvalidCountryName
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

pub fn validate_country_name(input: &[u8]) -> bool {
    input == DEFAULT_COUNTRY_NAME
        || match core::str::from_utf8(input) {
            Ok(val) => val
                .chars()
                .all(|c| c.is_alphabetic() || matches!(c, '-' | '.' | ' ')),
            Err(_) => false,
        }
    // we convert to &str and then to chars to handle
    // special alphabetic characters thanks to is_alphabetic()
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
        ensure!(
            validate_serial_number(&value),
            Self::Error::InvalidSerialNumber
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

pub fn validate_serial_number(input: &[u8]) -> bool {
    input == DEFAULT_SERIAL_NUMBER
        || input.iter().all(|c| {
            c.is_ascii_alphanumeric() || c.is_ascii_whitespace() || matches!(c, b'-' | b'_' | b'.')
        })
}

#[test]
fn test_validate_city_name_works() {
    assert_eq!(validate_city_name(b"Rio de Janeiro"), true);
    assert_eq!(validate_city_name(b"Ghent"), true);
    assert_eq!(validate_city_name(b"Cairo"), true);
    assert_eq!(
        validate_city_name(&vec![76, 105, 195, 168, 103, 101]), // b"Liège"
        true
    );

    assert_eq!(validate_city_name(b"Los_Angeles"), false);
}

#[test]
fn test_validate_country_name_works() {
    assert_eq!(validate_country_name(b"Brazil"), true);
    assert_eq!(validate_country_name(b"Belgium"), true);
    assert_eq!(validate_country_name(b"Egypt"), true);
    assert_eq!(validate_country_name(b"U.S.A"), true);

    assert_eq!(validate_country_name(b"Costa_Rica"), false);
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
