use sp_std::{marker::PhantomData, vec::Vec};

use crate::{Config, Error};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    ensure, sp_runtime::SaturatedConversion, traits::ConstU32, BoundedVec, RuntimeDebug,
};
use scale_info::TypeInfo;
use valip::{
    ip4::{cidr::CIDR, ip::IPv4},
    ip6::{Ip as IPv6, CIDR as IPv6Cidr},
};

/// A Public IP.
/// Needs to be valid format (ipv4 with cidr and in public range)
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct IP4<T: Config>(
    pub BoundedVec<u8, ConstU32<MAX_IP_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_IP_LENGTH>)>,
);

pub const MIN_IP_LENGHT: u32 = 9;
pub const MAX_IP_LENGTH: u32 = 18;

impl<T: Config> TryFrom<Vec<u8>> for IP4<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_IP_LENGHT.saturated_into(),
            Self::Error::IP4ToShort
        );
        let bounded_vec: BoundedVec<u8, ConstU32<MAX_IP_LENGTH>> =
            BoundedVec::try_from(value).map_err(|_| Self::Error::IP4ToLong)?;
        ensure!(CIDR::parse(&bounded_vec).is_ok(), Self::Error::InvalidIP4);
        Ok(Self(bounded_vec, PhantomData))
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for IP4<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for IP4<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}

/// A Public IP Gateway.
/// Needs to be valid format (ipv4 without cidr)
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct GW4<T: Config>(
    pub BoundedVec<u8, ConstU32<MAX_GATEWAY_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_GATEWAY_LENGTH>)>,
);

pub const MIN_GATEWAY_LENGTH: u32 = 7;
pub const MAX_GATEWAY_LENGTH: u32 = 15;

impl<T: Config> TryFrom<Vec<u8>> for GW4<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_GATEWAY_LENGTH.saturated_into(),
            Self::Error::GW4ToShort
        );
        let bounded_vec: BoundedVec<u8, ConstU32<MAX_GATEWAY_LENGTH>> =
            BoundedVec::try_from(value).map_err(|_| Self::Error::GW4ToLong)?;
        ensure!(IPv4::parse(&bounded_vec).is_ok(), Self::Error::InvalidGW4);
        Ok(Self(bounded_vec, PhantomData))
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for GW4<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for GW4<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}

/// Public Config IP6.
/// Needs to be valid format (ipv6 with cidr and in public range)
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct IP6<T: Config>(
    pub BoundedVec<u8, ConstU32<MAX_IP6_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_IP6_LENGTH>)>,
);

pub const MIN_IP6_LENGHT: u32 = 3;
pub const MAX_IP6_LENGTH: u32 = 42;

impl<T: Config> TryFrom<Vec<u8>> for IP6<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_IP6_LENGHT.saturated_into(),
            Self::Error::IP6ToShort
        );
        let bounded_vec: BoundedVec<u8, ConstU32<MAX_IP6_LENGTH>> =
            BoundedVec::try_from(value).map_err(|_| Self::Error::IP6ToLong)?;
        ensure!(
            IPv6Cidr::parse(&bounded_vec).is_ok(),
            Self::Error::InvalidIP6
        );
        Ok(Self(bounded_vec, PhantomData))
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for IP6<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for IP6<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}

/// A Public Config IP6
/// Needs to be valid format (ipv6 without cidr)
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct GW6<T: Config>(
    pub BoundedVec<u8, ConstU32<MAX_IP6_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_IP6_LENGTH>)>,
);

impl<T: Config> TryFrom<Vec<u8>> for GW6<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_IP6_LENGHT.saturated_into(),
            Self::Error::GW6ToShort
        );
        let bounded_vec: BoundedVec<u8, ConstU32<MAX_IP6_LENGTH>> =
            BoundedVec::try_from(value).map_err(|_| Self::Error::GW6ToLong)?;
        ensure!(IPv6::parse(&bounded_vec).is_ok(), Self::Error::InvalidGW6);
        Ok(Self(bounded_vec, PhantomData))
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for GW6<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for GW6<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}

/// A Public Config Domain
/// Needs to be valid format (ASCI Characters or numbers and dash).
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct Domain<T: Config>(
    pub BoundedVec<u8, ConstU32<MAX_DOMAIN_NAME_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_DOMAIN_NAME_LENGTH>)>,
);

pub const MIN_DOMAIN_NAME_LENGTH: u32 = 4;
pub const MAX_DOMAIN_NAME_LENGTH: u32 = 128;

impl<T: Config> TryFrom<Vec<u8>> for Domain<T> {
    type Error = Error<T>;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_DOMAIN_NAME_LENGTH.saturated_into(),
            Self::Error::DomainToShort
        );
        let bounded_vec: BoundedVec<u8, ConstU32<MAX_DOMAIN_NAME_LENGTH>> =
            BoundedVec::try_from(value).map_err(|_| Self::Error::DomainToLong)?;
        ensure!(
            validate_domain_name(&bounded_vec),
            Self::Error::InvalidDomain
        );
        Ok(Self(bounded_vec, PhantomData))
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for Domain<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for Domain<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}

fn validate_domain_name(input: &[u8]) -> bool {
    input
        .iter()
        .all(|c| matches!(c, b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.'))
}
