use crate::{Config, Error};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    ensure, sp_runtime::SaturatedConversion, traits::ConstU32, BoundedVec, RuntimeDebug,
};
use scale_info::TypeInfo;
use sp_std::marker::PhantomData;

pub const MIN_LOCATION_LENGTH: u32 = 1;
pub const MAX_LOCATION_LENGTH: u32 = 50;

// 4: Cuba, Fiji, Iran, ..
pub const MIN_COUNTRY_NAME_LENGTH: u32 = 4;
// 56: The United Kingdom of Great Britain and Northern Ireland
pub const MAX_COUNTRY_NAME_LENGTH: u32 = 56;

// 1: Y
pub const MIN_CITY_NAME_LENGTH: u32 = 1;
// 85: Llanfairpwllgwyngyllgogerychwyrndrobwllllantysiliogogogoch
pub const MAX_CITY_NAME_LENGTH: u32 = 58;

/// A location lat/long (ASCI Characters).
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct Location<T: Config>(
    // Latitude
    pub(crate) BoundedVec<u8, ConstU32<MAX_LOCATION_LENGTH>>,
    // Longitude
    pub(crate) BoundedVec<u8, ConstU32<MAX_LOCATION_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_LOCATION_LENGTH>)>,
);

impl<T: Config> TryFrom<(Vec<u8>, Vec<u8>)> for Location<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided tuple of byte vector
    /// (latitude and longitude) if one is below the minimum or exceeds
    /// the maximum allowed length or can not be converted to float or
    /// is out of [-90; 90] range (for latitude) or is out of [-180; 180]
    /// range (for longitude)
    fn try_from(value: (Vec<u8>, Vec<u8>)) -> Result<Self, Self::Error> {
        ensure!(
            value.0.len() >= MIN_LOCATION_LENGTH.saturated_into(),
            Self::Error::NodeLatitudeInputToShort
        );
        let latitute_bv: BoundedVec<u8, ConstU32<MAX_LOCATION_LENGTH>> =
            BoundedVec::try_from(value.0.clone())
                .map_err(|_| Self::Error::NodeLatitudeInputToLong)?;
        ensure!(
            validate_latitude_input(&latitute_bv),
            Self::Error::InvalidNodeLatitudeInput
        );

        ensure!(
            value.1.len() >= MIN_LOCATION_LENGTH.saturated_into(),
            Self::Error::NodeLongitudeInputToShort
        );
        let longitute_bv: BoundedVec<u8, ConstU32<MAX_LOCATION_LENGTH>> =
            BoundedVec::try_from(value.1.clone())
                .map_err(|_| Self::Error::NodeLongitudeInputToLong)?;
        ensure!(
            validate_longitude_input(&longitute_bv),
            Self::Error::InvalidNodeLongitudeInput
        );

        Ok(Self(latitute_bv, longitute_bv, PhantomData))
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for Location<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for Location<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone(), self.2)
    }
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

/// A country name (ASCI Characters).
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct CountryName<T: Config>(
    pub(crate) BoundedVec<u8, ConstU32<MAX_COUNTRY_NAME_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_COUNTRY_NAME_LENGTH>)>,
);

impl<T: Config> TryFrom<Vec<u8>> for CountryName<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_COUNTRY_NAME_LENGTH.saturated_into(),
            Self::Error::NodeCountryNameTooShort
        );
        let bounded_vec: BoundedVec<u8, ConstU32<MAX_COUNTRY_NAME_LENGTH>> =
            BoundedVec::try_from(value).map_err(|_| Self::Error::NodeCountryNameTooLong)?;
        ensure!(
            validate_country_name(&bounded_vec),
            Self::Error::InvalidNodeCountryName
        );
        Ok(Self(bounded_vec, PhantomData))
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

fn validate_country_name(input: &[u8]) -> bool {
    // TODO: find better alternative
    input
        .iter()
        .all(|c| c.is_ascii_alphabetic() || c.is_ascii_whitespace())
}

/// A city name (ASCI Characters).
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct CityName<T: Config>(
    pub(crate) BoundedVec<u8, ConstU32<MAX_CITY_NAME_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_CITY_NAME_LENGTH>)>,
);

impl<T: Config> TryFrom<Vec<u8>> for CityName<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_CITY_NAME_LENGTH.saturated_into(),
            Self::Error::NodeCityNameTooShort
        );
        let bounded_vec: BoundedVec<u8, ConstU32<MAX_CITY_NAME_LENGTH>> =
            BoundedVec::try_from(value).map_err(|_| Self::Error::NodeCityNameTooLong)?;
        ensure!(
            validate_city_name(&bounded_vec),
            Self::Error::InvalidNodeCityName
        );
        Ok(Self(bounded_vec, PhantomData))
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

fn validate_city_name(input: &[u8]) -> bool {
    // TODO: find better alternative
    input
        .iter()
        .all(|c| c.is_ascii_alphabetic() || c.is_ascii_whitespace())
}
