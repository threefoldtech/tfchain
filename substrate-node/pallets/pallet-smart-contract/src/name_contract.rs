use frame_support::{ensure, BoundedVec, RuntimeDebugNoBound};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_std::{convert::TryFrom, marker::PhantomData, vec::Vec};

use crate::{Config, Error};

/// A Name Contract Name.
#[derive(Encode, Decode, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct NameContractName<T: Config>(
    pub(crate) BoundedVec<u8, T::MaxNameContractNameLength>,
    PhantomData<(T, T::MaxNameContractNameLength)>,
);

pub const MIN_NAME_LENGTH: u32 = 3;

impl<T: Config> TryFrom<Vec<u8>> for NameContractName<T> {
    type Error = Error<T>;

    /// Fallible initialization from a provided byte vector if it is below the
    /// minimum or exceeds the maximum allowed length or contains invalid ASCII
    /// characters.
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        ensure!(
            value.len() >= MIN_NAME_LENGTH as usize,
            Self::Error::NameContractNameTooShort
        );
        let bounded_vec: BoundedVec<u8, T::MaxNameContractNameLength> =
            BoundedVec::try_from(value).map_err(|_| Self::Error::NameContractNameTooLong)?;
        ensure!(
            is_valid_name_contract_name(&bounded_vec),
            Self::Error::NameNotValid
        );
        Ok(Self(bounded_vec, PhantomData))
    }
}

/// Verify that a given slice can be used as a name contract name.
fn is_valid_name_contract_name(input: &[u8]) -> bool {
    input
        .iter()
        .all(|c| matches!(c, b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_'))
}

impl<T: Config> From<NameContractName<T>> for Vec<u8> {
    fn from(value: NameContractName<T>) -> Self {
        value.0.to_vec()
    }
}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> PartialEq for NameContractName<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Config> Eq for NameContractName<T> {}

// FIXME: did not find a way to automatically implement this.
impl<T: Config> Clone for NameContractName<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}
