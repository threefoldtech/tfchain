use sp_std::{marker::PhantomData, vec::Vec};

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    ensure, sp_runtime::SaturatedConversion, traits::ConstU32, BoundedVec, RuntimeDebug,
};
use scale_info::TypeInfo;
use valip::{ip4::ip::IPv4, ip6::Ip as IPv6, mac::Mac};

use crate::{Config, Error};

/// An Interface Name.
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct InterfaceName<T: Config>(
    pub BoundedVec<u8, ConstU32<20>>,
    PhantomData<(T, ConstU32<20>)>,
);

pub const MIN_INTF_NAME_LENGHT: u32 = 3;
pub const MAX_INTF_NAME_LENGTH: u32 = 20;

impl<T: Config> TryFrom<Vec<u8>> for InterfaceName<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_INTF_NAME_LENGHT.saturated_into(),
            Self::Error::InterfaceNameToShort
        );
        let bounded_vec: BoundedVec<u8, ConstU32<MAX_INTF_NAME_LENGTH>> =
            BoundedVec::try_from(value.clone()).map_err(|_| Self::Error::InterfaceNameToLong)?;
        ensure!(
            validate_interface_name(&value),
            Self::Error::InvalidInterfaceName
        );
        Ok(Self(bounded_vec, PhantomData))
    }
}

fn validate_interface_name(input: &[u8]) -> bool {
    input
        .iter()
        .all(|c| matches!(c, b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_'))
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for InterfaceName<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for InterfaceName<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}

/// A Interface MAC address.
/// Needs to be valid format (mac address format)
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct InterfaceMac<T: Config>(
    pub BoundedVec<u8, ConstU32<11>>,
    PhantomData<(T, ConstU32<11>)>,
);

pub const INTERFACE_MAC_LENGTH: u32 = 11;

impl<T: Config> TryFrom<Vec<u8>> for InterfaceMac<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= INTERFACE_MAC_LENGTH.saturated_into(),
            Self::Error::InterfaceMacToShort
        );
        let bounded_vec: BoundedVec<u8, ConstU32<INTERFACE_MAC_LENGTH>> =
            BoundedVec::try_from(value).map_err(|_| Self::Error::InterfaceMacToLong)?;
        ensure!(
            Mac::parse(&bounded_vec).is_ok(),
            Self::Error::InvalidMacAddress
        );
        Ok(Self(bounded_vec, PhantomData))
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for InterfaceMac<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for InterfaceMac<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}

/// A Interface IP address.
/// Needs to be a valid IP4 or IP6
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct InterfaceIp<T: Config>(
    pub BoundedVec<u8, ConstU32<39>>,
    PhantomData<(T, ConstU32<39>)>,
);

pub const MAX_INTERFACE_IP_LENGTH: u32 = 39;
pub const MIN_INTERFACE_IP_LENGTH: u32 = 9;

impl<T: Config> TryFrom<Vec<u8>> for InterfaceIp<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_INTERFACE_IP_LENGTH.saturated_into(),
            Self::Error::InterfaceIpToShort
        );
        let bounded_vec: BoundedVec<u8, ConstU32<MAX_INTERFACE_IP_LENGTH>> =
            BoundedVec::try_from(value).map_err(|_| Self::Error::InterfaceIpToLong)?;
        ensure!(
            IPv4::parse(&bounded_vec).is_ok() || IPv6::parse(&bounded_vec).is_ok(),
            Self::Error::InvalidInterfaceIP
        );
        Ok(Self(bounded_vec, PhantomData))
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for InterfaceIp<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for InterfaceIp<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}
