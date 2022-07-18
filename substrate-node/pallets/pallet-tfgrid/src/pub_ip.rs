use sp_std::{marker::PhantomData, vec::Vec};

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{ensure, sp_runtime::SaturatedConversion, BoundedVec, RuntimeDebug, traits::ConstU32};
use scale_info::TypeInfo;

use crate::ipv4;
use crate::{Config, Error};

/// A Public IP.
/// Needs to be valid format (ipv4 with cidr and in public range)
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct PublicIP<T: Config>(
    pub BoundedVec<u8, ConstU32<18>>,
    PhantomData<(T, ConstU32<18>)>,
);

pub const MIN_IP_LENGHT: u32 = 9;
pub const MAX_IP_LENGTH: u32 = 18;

impl<T: Config> TryFrom<Vec<u8>> for PublicIP<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_IP_LENGHT.saturated_into(),
            Self::Error::PublicIPToShort
        );
        let bounded_vec: BoundedVec<u8, ConstU32<MAX_IP_LENGTH>> =
            BoundedVec::try_from(value).map_err(|_| Self::Error::PublicIPToLong)?;
        ensure!(ipv4::parse_ip_cidr(&bounded_vec).is_ok(), Self::Error::InvalidPublicIP);
        Ok(Self(bounded_vec, PhantomData))
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
    pub BoundedVec<u8, ConstU32<18>>,
    PhantomData<(T, ConstU32<18>)>,
);

pub const MIN_GATEWAY_LENGTH: u32 = 7;

impl<T: Config> TryFrom<Vec<u8>> for GatewayIP<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_GATEWAY_LENGTH.saturated_into(),
            Self::Error::GatewayIPToShort
        );
        let bounded_vec: BoundedVec<u8, ConstU32<18>> =
            BoundedVec::try_from(value).map_err(|_| Self::Error::GatewayIPToLong)?;
        ensure!(ipv4::parse_ipv4(&bounded_vec).is_ok(), Self::Error::InvalidPublicIP);
        Ok(Self(bounded_vec, PhantomData))
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
