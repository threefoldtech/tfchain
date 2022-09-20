use crate::{geo, Config, Error};
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
    pub city: BoundedVec<u8, ConstU32<MAX_CITY_NAME_LENGTH>>,
    pub country: BoundedVec<u8, ConstU32<MAX_COUNTRY_NAME_LENGTH>>,
    pub latitude: BoundedVec<u8, ConstU32<MAX_LATITUDE_LENGTH>>,
    pub longitude: BoundedVec<u8, ConstU32<MAX_LONGITUDE_LENGTH>>,
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
            Self::Error::CityNameTooShort
        );
        let city: BoundedVec<u8, ConstU32<MAX_CITY_NAME_LENGTH>> =
            BoundedVec::try_from(value.0).map_err(|_| Self::Error::CityNameTooLong)?;
        let city_str = core::str::from_utf8(&city);
        ensure!(
            validate_city_name(&city) && city_str.is_ok(),
            Self::Error::InvalidCityName
        );

        // 2. country name
        ensure!(
            value.1.len() >= MIN_COUNTRY_NAME_LENGTH.saturated_into(),
            Self::Error::CountryNameTooShort
        );
        let country: BoundedVec<u8, ConstU32<MAX_COUNTRY_NAME_LENGTH>> =
            BoundedVec::try_from(value.1).map_err(|_| Self::Error::CountryNameTooLong)?;
        let country_str = core::str::from_utf8(&country);
        ensure!(
            validate_country_name(&country) && country_str.is_ok(),
            Self::Error::InvalidCountryName
        );

        // Check if [country][city] pair exists in data base
        ensure!(
            geo::validate_country_city(country_str.unwrap(), city_str.unwrap()),
            Self::Error::InvalidCountryCityPair
        );

        // 3. latitude
        ensure!(
            value.2.len() >= MIN_LATITUDE_LENGTH.saturated_into(),
            Self::Error::LatitudeInputToShort
        );
        let latitude: BoundedVec<u8, ConstU32<MAX_LATITUDE_LENGTH>> =
            BoundedVec::try_from(value.2.clone()).map_err(|_| Self::Error::LatitudeInputToLong)?;
        ensure!(
            validate_latitude_input(&latitude),
            Self::Error::InvalidLatitudeInput
        );

        // 4. latitude
        ensure!(
            value.3.len() >= MIN_LONGITUDE_LENGTH.saturated_into(),
            Self::Error::LongitudeInputToShort
        );
        let longitude: BoundedVec<u8, ConstU32<MAX_LONGITUDE_LENGTH>> =
            BoundedVec::try_from(value.3.clone()).map_err(|_| Self::Error::LongitudeInputToLong)?;
        ensure!(
            validate_longitude_input(&longitude),
            Self::Error::InvalidLongitudeInput
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
        .all(|c| c.is_ascii_alphabetic() || c.is_ascii_whitespace() || matches!(c, b'-'))
}

fn validate_country_name(input: &[u8]) -> bool {
    // TODO: find better alternative
    input
        .iter()
        .all(|c| c.is_ascii_alphabetic() || c.is_ascii_whitespace() || matches!(c, b'-'))
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

pub const MIN_SERIAL_NUMBER_LENGTH: u32 = 10;
pub const MAX_SERIAL_NUMBER_LENGTH: u32 = 50;

/// A serial number in ASCI Characters.
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct SerialNumber<T: Config>(
    pub BoundedVec<u8, ConstU32<MAX_SERIAL_NUMBER_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_SERIAL_NUMBER_LENGTH>)>,
);

impl<T: Config> TryFrom<Vec<u8>> for SerialNumber<T> {
    type Error = Error<T>;

    /// TODO: Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or ...
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_SERIAL_NUMBER_LENGTH.saturated_into(),
            Self::Error::SerialNumberTooShort
        );
        let bounded_vec: BoundedVec<u8, ConstU32<MAX_SERIAL_NUMBER_LENGTH>> =
            BoundedVec::try_from(value).map_err(|_| Self::Error::SerialNumberTooLong)?;
        ensure!(
            validate_serial_number(&bounded_vec),
            Self::Error::InvalidSerialNumber
        );
        Ok(Self(bounded_vec, PhantomData))
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

fn validate_serial_number(input: &[u8]) -> bool {
    // TODO: check serial specifications
    input
        .iter()
        .all(|c| matches!(c, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_'))
}

pub const ONE_THOUSAND: u128 = 1_000;
pub const GIB: u128 = 1024 * 1024 * 1024;

/// A resources capacity that countains HRU, SRU, CRU and MRU in integer values.
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct Resources<T: Config> {
    pub hru: u64,
    pub sru: u64,
    pub cru: u64,
    pub mru: u64,
    _marker: PhantomData<T>,
}

pub trait Capacity {
    type Rsrc;
    fn get_resources(&self) -> Self::Rsrc;
    fn get_hru(&self) -> u64;
    fn get_sru(&self) -> u64;
    fn get_cru(&self) -> u64;
    fn get_mru(&self) -> u64;
    fn get_cu(&self) -> u64;
    fn calc_cu(&self) -> u64;
    fn get_su(&self) -> u64;
    fn get_node_weight(&self) -> u64;
}

impl<T: Config> Capacity for Resources<T> {
    type Rsrc = Resources<T>;

    fn get_resources(&self) -> Self::Rsrc {
        Resources {
            hru: self.hru,
            sru: self.sru,
            cru: self.cru,
            mru: self.mru,
            _marker: PhantomData,
        }
    }

    fn get_hru(&self) -> u64 {
        self.hru
    }
    fn get_sru(&self) -> u64 {
        self.sru
    }
    fn get_cru(&self) -> u64 {
        self.cru
    }
    fn get_mru(&self) -> u64 {
        self.mru
    }

    fn get_cu(&self) -> u64 {
        let cu = self.calc_cu();
        let calculated_cu = 2 * (cu as u128 / GIB / ONE_THOUSAND);
        calculated_cu as u64
    }

    fn calc_cu(&self) -> u64 {
        let cru_min = self.cru as u128 * 2 * GIB * ONE_THOUSAND;
        let mru_min = ((self.mru as u128).checked_sub(1).unwrap_or(0) * GIB) * ONE_THOUSAND / 4;
        let sru_min = self.sru as u128 * ONE_THOUSAND / 50;

        if cru_min < mru_min && cru_min < sru_min {
            cru_min as u64
        } else if mru_min < cru_min && mru_min < sru_min {
            mru_min as u64
        } else if sru_min < cru_min && sru_min < mru_min {
            sru_min as u64
        } else {
            0
        }
    }

    fn get_su(&self) -> u64 {
        let su = self.hru as u128 * ONE_THOUSAND / 1200 + self.sru as u128 * ONE_THOUSAND / 250;
        let calculated_su = su / GIB;
        let result = calculated_su as u128 / ONE_THOUSAND;
        result as u64
    }

    fn get_node_weight(&self) -> u64 {
        let cu = self.get_cu();
        let su = self.get_su();
        cu * 2 + su
    }
}

impl<T: Config> TryFrom<(u64, u64, u64, u64)> for Resources<T> {
    type Error = Error<T>;

    /// TODO
    fn try_from(value: (u64, u64, u64, u64)) -> Result<Self, Self::Error> {
        let hru = value.0;
        let sru = value.1;
        let cru = value.2;
        let mru = value.3;

        ensure!(validate_hru_input(hru), Self::Error::InvalidHRUInput);
        ensure!(validate_sru_input(sru), Self::Error::InvalidSRUInput);
        ensure!(validate_cru_input(cru), Self::Error::InvalidCRUInput);
        ensure!(validate_mru_input(mru), Self::Error::InvalidMRUInput);

        Ok(Self {
            hru,
            sru,
            cru,
            mru,
            _marker: PhantomData,
        })
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for Resources<T> {
    fn eq(&self, other: &Self) -> bool {
        self.hru == other.hru
            && self.sru == other.sru
            && self.cru == other.cru
            && self.mru == other.mru
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for Resources<T> {
    fn clone(&self) -> Self {
        Self {
            hru: self.hru.clone(),
            sru: self.sru.clone(),
            cru: self.cru.clone(),
            mru: self.mru.clone(),
            _marker: PhantomData,
        }
    }
}

fn validate_hru_input(_input: u64) -> bool {
    // TODO
    true
}

fn validate_sru_input(_input: u64) -> bool {
    // TODO
    true
}

fn validate_cru_input(_input: u64) -> bool {
    // TODO
    true
}

fn validate_mru_input(_input: u64) -> bool {
    // TODO
    true
}
