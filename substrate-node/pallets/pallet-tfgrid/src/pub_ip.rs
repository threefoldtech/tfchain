use crate::{Config, Error, PublicIpGatewayInput, PublicIpIpInput};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    ensure, sp_runtime::SaturatedConversion, traits::ConstU32, BoundedVec, RuntimeDebug,
};
use scale_info::TypeInfo;
use sp_std::marker::PhantomData;
use valip::ip4::{Ip, CIDR as IPv4Cidr};

/// A Public IP.
/// Needs to be valid format (ipv4 with cidr and in public range)
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct PublicIP<T: Config>(
    pub BoundedVec<u8, ConstU32<MAX_IP_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_IP_LENGTH>)>,
);

pub const MIN_IP_LENGTH: u32 = 9;
pub const MAX_IP_LENGTH: u32 = 18;

impl<T: Config> TryFrom<PublicIpIpInput> for PublicIP<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: PublicIpIpInput) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_IP_LENGTH.saturated_into(),
            Self::Error::PublicIPTooShort
        );
        ensure!(
            value.len() <= MAX_IP_LENGTH.saturated_into(),
            Self::Error::PublicIPTooLong
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
impl<T: Config> PartialEq for PublicIP<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for PublicIP<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}

/// A Public IP Gateway.
/// Needs to be valid format (ipv4 without cidr)
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct GatewayIP<T: Config>(
    pub BoundedVec<u8, ConstU32<MAX_GATEWAY_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_GATEWAY_LENGTH>)>,
);

pub const MIN_GATEWAY_LENGTH: u32 = 7;
pub const MAX_GATEWAY_LENGTH: u32 = 15;

impl<T: Config> TryFrom<PublicIpGatewayInput> for GatewayIP<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: PublicIpGatewayInput) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_GATEWAY_LENGTH.saturated_into(),
            Self::Error::GatewayIPTooShort
        );
        ensure!(
            value.len() <= MAX_GATEWAY_LENGTH.saturated_into(),
            Self::Error::GatewayIPTooLong
        );

        if let Ok(ip) = Ip::parse(&value) {
            ensure!(ip.is_public() && ip.is_unicast(), Self::Error::InvalidIP4);
            Ok(Self(value, PhantomData))
        } else {
            Err(Self::Error::InvalidIP4)
        }
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for GatewayIP<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for GatewayIP<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}
