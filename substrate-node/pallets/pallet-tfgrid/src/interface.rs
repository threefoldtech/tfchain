use crate::{Config, Error, InterfaceIpInput, InterfaceMacInput, InterfaceNameInput};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    ensure, sp_runtime::SaturatedConversion, traits::ConstU32, BoundedVec, RuntimeDebug,
};
use scale_info::TypeInfo;
use sp_std::marker::PhantomData;
use valip::{ip4::Ip as IPv4, ip6::Ip as IPv6, mac::Mac};

/// An Interface Name.
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct InterfaceName<T: Config>(
    pub BoundedVec<u8, ConstU32<MAX_INTF_NAME_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_INTF_NAME_LENGTH>)>,
);

pub const MIN_INTF_NAME_LENGTH: u32 = 3;
pub const MAX_INTF_NAME_LENGTH: u32 = 20;

impl<T: Config> TryFrom<InterfaceNameInput> for InterfaceName<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: InterfaceNameInput) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_INTF_NAME_LENGTH.saturated_into(),
            Self::Error::InterfaceNameTooShort
        );
        ensure!(
            value.len() <= MAX_INTF_NAME_LENGTH.saturated_into(),
            Self::Error::InterfaceNameTooLong
        );
        ensure!(
            validate_interface_name(&value),
            Self::Error::InvalidInterfaceName
        );

        let name: BoundedVec<u8, ConstU32<MAX_INTF_NAME_LENGTH>> =
            value.try_into().unwrap_or_default();

        Ok(Self(name, PhantomData))
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
    pub BoundedVec<u8, ConstU32<INTERFACE_MAC_LENGTH>>,
    PhantomData<(T, ConstU32<INTERFACE_MAC_LENGTH>)>,
);

pub const INTERFACE_MAC_LENGTH: u32 = 17;

impl<T: Config> TryFrom<InterfaceMacInput> for InterfaceMac<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: InterfaceMacInput) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= INTERFACE_MAC_LENGTH.saturated_into(),
            Self::Error::InterfaceMacTooShort
        );
        ensure!(
            value.len() <= INTERFACE_MAC_LENGTH.saturated_into(),
            Self::Error::InterfaceMacTooLong
        );
        ensure!(Mac::parse(&value).is_ok(), Self::Error::InvalidMacAddress);

        let mac: BoundedVec<u8, ConstU32<INTERFACE_MAC_LENGTH>> =
            value.try_into().unwrap_or_default();

        Ok(Self(mac, PhantomData))
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
    pub BoundedVec<u8, ConstU32<MAX_INTERFACE_IP_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_INTERFACE_IP_LENGTH>)>,
);

pub const MIN_INTERFACE_IP_LENGTH: u32 = 7;
pub const MAX_INTERFACE_IP_LENGTH: u32 = 42;

impl<T: Config> TryFrom<InterfaceIpInput> for InterfaceIp<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: InterfaceIpInput) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_INTERFACE_IP_LENGTH.saturated_into(),
            Self::Error::InterfaceIpTooShort
        );
        ensure!(
            value.len() <= MAX_INTERFACE_IP_LENGTH.saturated_into(),
            Self::Error::InterfaceIpTooLong
        );
        ensure!(
            IPv4::parse(&value).is_ok() || IPv6::parse(&value).is_ok(),
            Self::Error::InvalidInterfaceIP
        );

        let ip: BoundedVec<u8, ConstU32<MAX_INTERFACE_IP_LENGTH>> =
            value.try_into().unwrap_or_default();
        Ok(Self(ip, PhantomData))
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
