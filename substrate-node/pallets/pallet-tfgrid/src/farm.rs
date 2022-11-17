use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::Get;
use frame_support::{ensure, sp_runtime::SaturatedConversion, BoundedVec, RuntimeDebug};
use scale_info::TypeInfo;
use sp_std::{marker::PhantomData, vec::Vec};

use crate::{Config, Error, FarmNameInput};

/// A Farm name (ASCI Characters).
///
/// It is bounded in size (inclusive range [MinLength, MaxLength]) and must be a valid ipv6
#[derive(Encode, Decode, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct FarmName<T: Config>(
    pub(crate) BoundedVec<u8, T::MaxFarmNameLength>,
    PhantomData<(T, T::MaxFarmNameLength)>,
);

pub const MIN_FARM_NAME_LENGTH: u32 = 3;

impl<T: Config> TryFrom<FarmNameInput<T>> for FarmName<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: FarmNameInput<T>) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_FARM_NAME_LENGTH.saturated_into(),
            Self::Error::FarmNameTooShort
        );
        ensure!(
            value.len() <= T::MaxFarmNameLength::get() as usize,
            Self::Error::FarmNameTooLong
        );
        ensure!(validate_farm_name(&value), Self::Error::InvalidFarmName);
        Ok(Self(value, PhantomData))
    }
}

impl<T: Config> From<FarmName<T>> for Vec<u8> {
    fn from(value: FarmName<T>) -> Self {
        value.0.to_vec()
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for FarmName<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for FarmName<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}

pub fn replace_farm_name_invalid_characters(input: &[u8]) -> Vec<u8> {
    input
        .iter()
        .map(|c| match c {
            b' ' => b'_',
            b'\'' => b'-',
            b';' => b'_',
            _ => *c,
        })
        .collect()
}

fn validate_farm_name(input: &[u8]) -> bool {
    input
        .iter()
        .all(|c| matches!(c, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_'))
}
