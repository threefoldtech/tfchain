use crate::{Config, DomainInput, Error, FullIp6Input, FullIp4Input, Gw4Input, Gw6Input, Ip4Input, Ip6Input};
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

impl<T: Config> TryFrom<Ip4Input> for IP4<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: Ip4Input) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_IP4_LENGTH.saturated_into(),
            Self::Error::IP4TooShort
        );
        ensure!(
            value.len() <= MAX_IP4_LENGTH.saturated_into(),
            Self::Error::IP4TooLong
        );

        if let Ok(ip) = IPv4Cidr::parse(&value) {
            ensure!(ip.is_public() && ip.is_unicast(), Self::Error::InvalidIP4);
            Ok(Self(value, PhantomData))
        } else {
            Err(Self::Error::InvalidIP4)
        }
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

impl<T: Config> TryFrom<Gw4Input> for GW4<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: Gw4Input) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_GW4_LENGTH.saturated_into(),
            Self::Error::GW4TooShort
        );
        ensure!(
            value.len() <= MAX_GW4_LENGTH.saturated_into(),
            Self::Error::GW4TooLong
        );

        if let Ok(ip) = IPv4::parse(&value) {
            ensure!(ip.is_public() && ip.is_unicast(), Self::Error::InvalidIP4);
            Ok(Self(value, PhantomData))
        } else {
            Err(Self::Error::InvalidIP4)
        }
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

impl<T: Config> TryFrom<Ip6Input> for IP6<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: Ip6Input) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_IP6_LENGTH.saturated_into(),
            Self::Error::IP6TooShort
        );
        ensure!(
            value.len() <= MAX_IP6_LENGTH.saturated_into(),
            Self::Error::IP6TooLong
        );

        if let Ok(ip) = IPv6Cidr::parse(&value) {
            ensure!(ip.is_public() && ip.is_unicast(), Self::Error::InvalidIP6);
            Ok(Self(value, PhantomData))
        } else {
            Err(Self::Error::InvalidIP6)
        }
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

impl<T: Config> TryFrom<Gw6Input> for GW6<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: Gw6Input) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_GW6_LENGTH.saturated_into(),
            Self::Error::GW6TooShort
        );
        ensure!(
            value.len() <= MAX_GW6_LENGTH.saturated_into(),
            Self::Error::GW6TooLong
        );

        if let Ok(ip) = IPv6::parse(&value) {
            // Todo, validate if unicast
            ensure!(ip.is_public() && ip.is_unicast(), Self::Error::InvalidIP6);
            Ok(Self(value, PhantomData))
        } else {
            Err(Self::Error::InvalidIP6)
        }
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


/// A Full IP4.
/// Needs to be valid IP and Gateway, IP must also fall in range of gateway
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct FullPublicIp4<T: Config> {
    pub ip: BoundedVec<u8, ConstU32<MAX_IP4_LENGTH>>,
    pub gateway: BoundedVec<u8, ConstU32<MAX_GW4_LENGTH>>,
    _marker: PhantomData<(T, ConstU32<MAX_GW4_LENGTH>)>,
}

impl<T: Config> TryFrom<FullIp4Input> for FullPublicIp4<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: FullIp4Input) -> Result<Self, Self::Error> {
        ensure!(
            value.gw.len() >= MIN_GW4_LENGTH.saturated_into(),
            Self::Error::GatewayIPTooShort
        );
        ensure!(
            value.gw.len() <= MAX_GW4_LENGTH.saturated_into(),
            Self::Error::GatewayIPTooLong
        );

        ensure!(
            value.ip.len() >= MIN_IP4_LENGTH.saturated_into(),
            Self::Error::PublicIPTooShort
        );
        ensure!(
            value.ip.len() <= MAX_IP4_LENGTH.saturated_into(),
            Self::Error::PublicIPTooLong
        );

        if let Ok(gw) = IPv4::parse(&value.gw) {
            ensure!(gw.is_public() && gw.is_unicast(), Self::Error::InvalidGW4);
            if let Ok(ip) = IPv4Cidr::parse(&value.ip) {
                ensure!(
                    ip.is_public() && ip.is_unicast() && ip.contains(gw),
                    Self::Error::InvalidIP4
                );
            } else {
                return Err(Self::Error::InvalidIP4);
            }
            Ok(Self {
                ip: value.ip,
                gateway: value.gw,
                _marker: PhantomData,
            })
        } else {
            Err(Self::Error::InvalidGW4)
        }
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for FullPublicIp4<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ip == other.ip && self.gateway == other.gateway
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for FullPublicIp4<T> {
    fn clone(&self) -> Self {
        Self {
            ip: self.ip.clone(),
            gateway: self.gateway.clone(),
            _marker: PhantomData,
        }
    }
}

/// A Full IP4.
/// Needs to be valid IP and Gateway, IP must also fall in range of gateway
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct FullPublicIp6<T: Config> {
    pub ip: BoundedVec<u8, ConstU32<MAX_IP6_LENGTH>>,
    pub gateway: BoundedVec<u8, ConstU32<MAX_GW6_LENGTH>>,
    _marker: PhantomData<(T, ConstU32<MAX_GW6_LENGTH>)>,
}

impl<T: Config> TryFrom<FullIp6Input> for FullPublicIp6<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector
    fn try_from(value: FullIp6Input) -> Result<Self, Self::Error> {
        ensure!(
            value.gw.len() >= MIN_GW6_LENGTH.saturated_into(),
            Self::Error::GatewayIPTooShort
        );
        ensure!(
            value.gw.len() <= MAX_GW6_LENGTH.saturated_into(),
            Self::Error::GatewayIPTooLong
        );

        ensure!(
            value.ip.len() >= MIN_IP6_LENGTH.saturated_into(),
            Self::Error::PublicIPTooShort
        );
        ensure!(
            value.ip.len() <= MAX_IP6_LENGTH.saturated_into(),
            Self::Error::PublicIPTooLong
        );

        if let Ok(gw) = IPv6::parse(&value.gw) {
            ensure!(gw.is_public() && gw.is_unicast(), Self::Error::InvalidGW4);
            if let Ok(ip) = IPv6Cidr::parse(&value.ip) {
                ensure!(
                    ip.is_public() && ip.is_unicast() && ip.contains(gw),
                    Self::Error::InvalidIP6
                );
            } else {
                return Err(Self::Error::InvalidIP6);
            }
            Ok(Self {
                ip: value.ip,
                gateway: value.gw,
                _marker: PhantomData,
            })
        } else {
            Err(Self::Error::InvalidGW4)
        }
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for FullPublicIp6<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ip == other.ip && self.gateway == other.gateway
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for FullPublicIp6<T> {
    fn clone(&self) -> Self {
        Self {
            ip: self.ip.clone(),
            gateway: self.gateway.clone(),
            _marker: PhantomData,
        }
    }
}
