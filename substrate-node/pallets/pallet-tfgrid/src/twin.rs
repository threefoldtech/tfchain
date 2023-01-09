use crate::{Config, Error, RelayInput};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    ensure, sp_runtime::SaturatedConversion, traits::ConstU32, BoundedVec, RuntimeDebug,
};
use scale_info::TypeInfo;
use sp_std::marker::PhantomData;

/// A Twin Relay address.
/// Can be a dns name, ip, ..
///
/// It is bounded in size (inclusive range [MinLength, MaxLength]) and must be a valid ipv6
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct Relay<T: Config>(
    pub BoundedVec<u8, ConstU32<MAX_RELAY_LENGTH>>,
    PhantomData<(T, ConstU32<MAX_RELAY_LENGTH>)>,
);

pub const MIN_RELAY_LENGTH: u32 = 10;
// Max relay length is half of max URL length
pub const MAX_RELAY_LENGTH: u32 = 1024;

impl<T: Config> TryFrom<RelayInput> for Relay<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: RelayInput) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_RELAY_LENGTH.saturated_into(),
            Self::Error::RelayTooShort
        );
        ensure!(
            value.len() <= MAX_RELAY_LENGTH.saturated_into(),
            Self::Error::RelayTooLong
        );
        Ok(Self(value, PhantomData))
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for Relay<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for Relay<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}
