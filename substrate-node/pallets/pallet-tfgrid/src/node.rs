use crate::{Config, Error};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    ensure, sp_runtime::SaturatedConversion, traits::ConstU32, BoundedVec, RuntimeDebug,
};
use scale_info::TypeInfo;
use sp_std::marker::PhantomData;

pub const MIN_NODE_LOCATION_LENGTH: u32 = 1;
pub const MAX_NODE_LOCATION_LENGTH: u32 = 50;

/// A Node latitude (ASCI Characters).
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct NodeLatitude<T: Config>(
    pub(crate) BoundedVec<u8, ConstU32<MAX_NODE_LOCATION_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_NODE_LOCATION_LENGTH>)>,
);

impl<T: Config> TryFrom<Vec<u8>> for NodeLatitude<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_NODE_LOCATION_LENGTH.saturated_into(),
            Self::Error::NodeLatitudeInputToShort
        );
        let bounded_vec: BoundedVec<u8, ConstU32<MAX_NODE_LOCATION_LENGTH>> =
            BoundedVec::try_from(value.clone())
                .map_err(|_| Self::Error::NodeLatitudeInputToLong)?;
        ensure!(
            validate_latitude_input(&bounded_vec),
            Self::Error::InvalidNodeLatitude
        );
        Ok(Self(bounded_vec, PhantomData))
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for NodeLatitude<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for NodeLatitude<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}

fn validate_latitude_input(input: &[u8]) -> bool {
    // TODO
    true
}

/// A Node longitude (ASCI Characters).
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct NodeLongitude<T: Config>(
    pub(crate) BoundedVec<u8, ConstU32<MAX_NODE_LOCATION_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_NODE_LOCATION_LENGTH>)>,
);

impl<T: Config> TryFrom<Vec<u8>> for NodeLongitude<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_NODE_LOCATION_LENGTH.saturated_into(),
            Self::Error::NodeLongitudeInputToShort
        );
        let bounded_vec: BoundedVec<u8, ConstU32<MAX_NODE_LOCATION_LENGTH>> =
            BoundedVec::try_from(value.clone())
                .map_err(|_| Self::Error::NodeLongitudeInputToLong)?;
        ensure!(
            validate_longitude_input(&bounded_vec),
            Self::Error::InvalidNodeLongitude
        );
        Ok(Self(bounded_vec, PhantomData))
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for NodeLongitude<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for NodeLongitude<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}

fn validate_longitude_input(input: &[u8]) -> bool {
    // TODO
    true
}
