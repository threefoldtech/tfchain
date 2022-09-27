use crate::{Config, DomainInput, Error, GW4Input, GW6Input, IP4Input, IP6Input};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    ensure, sp_runtime::SaturatedConversion, traits::ConstU32, BoundedVec, RuntimeDebug,
};
use scale_info::TypeInfo;
use sp_std::marker::PhantomData;
use valip::{
    ip4::{Ip as IPv4, CIDR as IPv4Cidr},
    ip6::{Ip as IPv6, CIDR as IPv6Cidr},
};

/// A Public IP.
/// Needs to be valid format (ipv4 with cidr and in public range)
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct IP4<T: Config>(
    pub BoundedVec<u8, ConstU32<MAX_IP4_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_IP4_LENGTH>)>,
);

pub const MIN_IP4_LENGTH: u32 = 9;
pub const MAX_IP4_LENGTH: u32 = 18;

impl<T: Config> TryFrom<IP4Input> for IP4<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: IP4Input) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_IP4_LENGTH.saturated_into(),
            Self::Error::IP4TooShort
        );
        ensure!(
            value.len() <= MAX_IP4_LENGTH.saturated_into(),
            Self::Error::IP4TooLong
        );
        ensure!(IPv4Cidr::parse(&value).is_ok(), Self::Error::InvalidIP4);

        Ok(Self(value, PhantomData))
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
    pub BoundedVec<u8, ConstU32<MAX_GW4_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_GW4_LENGTH>)>,
);

pub const MIN_GW4_LENGTH: u32 = 7;
pub const MAX_GW4_LENGTH: u32 = 15;

impl<T: Config> TryFrom<GW4Input> for GW4<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: GW4Input) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_GW4_LENGTH.saturated_into(),
            Self::Error::GW4TooShort
        );
        ensure!(
            value.len() <= MAX_GW4_LENGTH.saturated_into(),
            Self::Error::GW4TooLong
        );
        ensure!(IPv4::parse(&value).is_ok(), Self::Error::InvalidGW4);

        Ok(Self(value, PhantomData))
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

pub const MIN_IP6_LENGTH: u32 = 3;
pub const MAX_IP6_LENGTH: u32 = 43;

impl<T: Config> TryFrom<IP6Input> for IP6<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: IP6Input) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_IP6_LENGTH.saturated_into(),
            Self::Error::IP6TooShort
        );
        ensure!(
            value.len() <= MAX_IP6_LENGTH.saturated_into(),
            Self::Error::IP6TooLong
        );
        ensure!(IPv6Cidr::parse(&value).is_ok(), Self::Error::InvalidIP6);

        Ok(Self(value, PhantomData))
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

/// A Public Config IP6 gateway
/// Needs to be valid format (ipv6 without CIDR)
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct GW6<T: Config>(
    pub BoundedVec<u8, ConstU32<MAX_GW6_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_GW6_LENGTH>)>,
);

pub const MIN_GW6_LENGTH: u32 = MIN_IP6_LENGTH;
pub const MAX_GW6_LENGTH: u32 = 39;

impl<T: Config> TryFrom<GW6Input> for GW6<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: GW6Input) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_GW6_LENGTH.saturated_into(),
            Self::Error::GW6TooShort
        );
        ensure!(
            value.len() <= MAX_GW6_LENGTH.saturated_into(),
            Self::Error::GW6TooLong
        );
        ensure!(IPv6::parse(&value).is_ok(), Self::Error::InvalidGW6);

        Ok(Self(value, PhantomData))
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

impl<T: Config> TryFrom<DomainInput> for Domain<T> {
    type Error = Error<T>;

    fn try_from(value: DomainInput) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_DOMAIN_NAME_LENGTH.saturated_into(),
            Self::Error::DomainTooShort
        );
        ensure!(
            value.len() <= MAX_DOMAIN_NAME_LENGTH.saturated_into(),
            Self::Error::DomainTooLong
        );
        ensure!(validate_domain_name(&value), Self::Error::InvalidDomain);

        Ok(Self(value, PhantomData))
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
        .all(|c| matches!(c, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'.'))
}
